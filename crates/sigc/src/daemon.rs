//! Async daemon server for sigc RPC
//!
//! Provides a REQ/REP server using nng (sync) wrapped in Tokio for async behavior.
//! This maintains the brokerless architecture while providing non-blocking operations.

use anyhow::Result;
use nng::{options::Options, Protocol, Socket};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// RPC request message
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    /// Ping to check if server is alive
    Ping,

    /// Compile source code
    Compile { source: String },

    /// Run a backtest
    Run { source: String },

    /// Get server status
    Status,

    /// Shutdown the server
    Shutdown,
}

/// RPC response message
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    /// Pong response
    Pong,

    /// Compilation result
    CompileResult {
        success: bool,
        nodes: usize,
        error: Option<String>,
    },

    /// Backtest result
    RunResult {
        success: bool,
        total_return: f64,
        sharpe_ratio: f64,
        max_drawdown: f64,
        error: Option<String>,
    },

    /// Server status
    Status {
        version: String,
        uptime_secs: u64,
        requests_handled: u64,
    },

    /// Server shutting down
    Shutdown,

    /// Error response
    Error { message: String },
}

/// Daemon server state with runtime pool for concurrent execution
pub struct Daemon {
    socket: Arc<Socket>,
    compiler: Arc<sig_compiler::Compiler>,
    runtime_pool: Arc<Vec<Mutex<sig_runtime::Runtime>>>,
    pool_size: usize,
    #[allow(dead_code)]
    cache: Option<sig_cache::Cache>,
    start_time: std::time::Instant,
    requests_handled: Arc<std::sync::atomic::AtomicU64>,
}

impl Daemon {
    /// Create a new daemon server with default pool size
    ///
    /// Uses a conservative default to avoid thread oversubscription:
    /// - Each Runtime uses Rayon internally (defaults to num_cpus threads)
    /// - Pool size = max(1, num_cpus / 2) to leave headroom for Rayon
    /// - Example: 8 cores → 4 workers, each using ~2 cores via Rayon
    pub fn new(address: &str) -> Result<Self> {
        let num_cpus = num_cpus::get();
        // Conservative default: half the cores to avoid oversubscription
        // Each worker's Rayon pool will use multiple threads internally
        let pool_size = std::cmp::max(1, num_cpus / 2);
        tracing::info!(
            "Auto-configured {} runtime workers ({} CPUs detected)",
            pool_size, num_cpus
        );
        Self::with_pool_size(address, pool_size)
    }

    /// Create a new daemon server with specified runtime pool size
    pub fn with_pool_size(address: &str, pool_size: usize) -> Result<Self> {
        let socket = Socket::new(Protocol::Rep0)?;
        socket.listen(address)?;

        // Set receive timeout for non-blocking behavior
        socket.set_opt::<nng::options::RecvTimeout>(Some(Duration::from_millis(100)))?;

        tracing::info!("Daemon listening on {} (async mode, {} runtime workers)", address, pool_size);

        // Daemon owns the cache exclusively
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("sigc");

        let cache = match sig_cache::Cache::open(&cache_dir) {
            Ok(c) => {
                tracing::info!("Cache opened successfully (daemon owns it)");
                Some(c)
            }
            Err(e) => {
                tracing::warn!("Failed to open cache: {}, continuing without cache", e);
                None
            }
        };

        // Create compiler with cache if available
        let compiler = if let Some(c) = cache.clone() {
            sig_compiler::Compiler::with_cache(c)
        } else {
            sig_compiler::Compiler::new()
        };

        // Calculate optimal Rayon threads per worker to avoid oversubscription
        // If RAYON_NUM_THREADS is set, respect it; otherwise auto-configure
        let rayon_threads_per_worker = std::env::var("RAYON_NUM_THREADS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or_else(|| {
                // Auto-configure: give each worker fair share of CPUs
                let num_cpus = num_cpus::get();
                std::cmp::max(1, num_cpus / pool_size)
            });

        tracing::info!(
            "Each worker will use up to {} Rayon threads",
            rayon_threads_per_worker
        );

        // Set Rayon's global thread pool (affects all workers)
        // Note: This is a global setting, but we're being conservative
        if std::env::var("RAYON_NUM_THREADS").is_err() {
            std::env::set_var("RAYON_NUM_THREADS", rayon_threads_per_worker.to_string());
        }

        // Create pool of runtime instances for concurrent execution
        let mut runtime_pool = Vec::with_capacity(pool_size);
        for i in 0..pool_size {
            let runtime = if let Some(c) = cache.clone() {
                sig_runtime::Runtime::with_cache(c)
            } else {
                sig_runtime::Runtime::new()
            };
            runtime_pool.push(Mutex::new(runtime));
            tracing::debug!("Initialized runtime worker {}/{}", i + 1, pool_size);
        }

        Ok(Daemon {
            socket: Arc::new(socket),
            compiler: Arc::new(compiler),
            runtime_pool: Arc::new(runtime_pool),
            pool_size,
            cache,
            start_time: std::time::Instant::now(),
            requests_handled: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
    }

    /// Run the daemon server loop (async via spawn_blocking with concurrent request handling)
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Daemon server started (async mode with {} concurrent workers)", self.pool_size);

        let socket = Arc::clone(&self.socket);
        let compiler = Arc::clone(&self.compiler);
        let runtime_pool = Arc::clone(&self.runtime_pool);
        let pool_size = self.pool_size;
        let start_time = self.start_time;
        let requests_handled = Arc::clone(&self.requests_handled);
        let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));

        loop {
            // Check if shutdown was requested
            if shutdown.load(std::sync::atomic::Ordering::SeqCst) {
                tracing::info!("Daemon shutting down");
                break;
            }

            // Spawn blocking task for recv (avoids blocking the tokio runtime)
            let socket_clone = Arc::clone(&socket);
            let msg = match tokio::task::spawn_blocking(move || socket_clone.recv()).await? {
                Ok(msg) => msg,
                Err(nng::Error::TimedOut) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    continue;
                }
                Err(e) => {
                    tracing::error!("Receive error: {}", e);
                    continue;
                }
            };

            // Parse request
            let request: Request = match serde_json::from_slice(&msg) {
                Ok(req) => req,
                Err(e) => {
                    let response = Response::Error {
                        message: format!("Invalid request: {}", e),
                    };
                    self.send_response_async(&response).await?;
                    continue;
                }
            };

            tracing::debug!("Received request: {:?}", request);

            // Check for shutdown request
            if matches!(request, Request::Shutdown) {
                let response = Response::Shutdown;
                self.send_response_async(&response).await?;
                shutdown.store(true, std::sync::atomic::Ordering::SeqCst);
                continue;
            }

            let request_id = requests_handled.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            // Spawn a concurrent task for each request (non-blocking)
            let socket_clone = Arc::clone(&socket);
            let compiler_clone = Arc::clone(&compiler);
            let runtime_pool_clone = Arc::clone(&runtime_pool);

            tokio::spawn(async move {
                // Select runtime from pool using round-robin
                let runtime_idx = (request_id as usize) % pool_size;

                // Handle request (potentially CPU-intensive, so spawn_blocking)
                let response = tokio::task::spawn_blocking(move || {
                    Self::handle_request_sync(
                        &compiler_clone,
                        &runtime_pool_clone,
                        runtime_idx,
                        start_time,
                        request,
                    )
                })
                .await
                .unwrap_or_else(|e| Response::Error {
                    message: format!("Request handling failed: {}", e),
                });

                // Send response
                if let Err(e) = Self::send_response_sync(&socket_clone, &response) {
                    tracing::error!("Failed to send response: {}", e);
                }
            });

            // Immediately loop back to receive next request (concurrent handling)
        }

        Ok(())
    }

    fn send_response_sync(socket: &Socket, response: &Response) -> Result<()> {
        let data = serde_json::to_vec(response)?;
        socket.send(&data).map_err(|(_, e)| e)?;
        Ok(())
    }

    async fn send_response_async(&self, response: &Response) -> Result<()> {
        let data = serde_json::to_vec(response)?;
        let socket = Arc::clone(&self.socket);
        tokio::task::spawn_blocking(move || socket.send(&data).map_err(|(_, e)| e)).await??;
        Ok(())
    }

    fn handle_request_sync(
        compiler: &sig_compiler::Compiler,
        runtime_pool: &[Mutex<sig_runtime::Runtime>],
        runtime_idx: usize,
        start_time: std::time::Instant,
        request: Request,
    ) -> Response {
        match request {
            Request::Ping => Response::Pong,

            Request::Compile { source } => match compiler.compile(&source) {
                Ok(ir) => Response::CompileResult {
                    success: true,
                    nodes: ir.nodes.len(),
                    error: None,
                },
                Err(e) => Response::CompileResult {
                    success: false,
                    nodes: 0,
                    error: Some(e.to_string()),
                },
            },

            Request::Run { source } => {
                // First compile
                let ir = match compiler.compile(&source) {
                    Ok(ir) => ir,
                    Err(e) => {
                        return Response::RunResult {
                            success: false,
                            total_return: 0.0,
                            sharpe_ratio: 0.0,
                            max_drawdown: 0.0,
                            error: Some(e.to_string()),
                        };
                    }
                };

                // Select runtime from pool (each runtime can run independently)
                let runtime = &runtime_pool[runtime_idx];
                let mut runtime_guard = runtime.lock().unwrap();

                tracing::debug!("Running backtest on worker {}", runtime_idx);

                match runtime_guard.run_ir(&ir) {
                    Ok(report) => Response::RunResult {
                        success: true,
                        total_return: report.metrics.total_return,
                        sharpe_ratio: report.metrics.sharpe_ratio,
                        max_drawdown: report.metrics.max_drawdown,
                        error: None,
                    },
                    Err(e) => Response::RunResult {
                        success: false,
                        total_return: 0.0,
                        sharpe_ratio: 0.0,
                        max_drawdown: 0.0,
                        error: Some(e.to_string()),
                    },
                }
            }

            Request::Status => Response::Status {
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_secs: start_time.elapsed().as_secs(),
                requests_handled: 0, // Would need to pass counter
            },

            Request::Shutdown => Response::Shutdown,
        }
    }
}

/// Client for connecting to the daemon (async via spawn_blocking)
pub struct Client {
    socket: Arc<Socket>,
}

impl Client {
    /// Connect to a daemon server
    pub fn new(address: &str) -> Result<Self> {
        let socket = Socket::new(Protocol::Req0)?;
        socket.dial(address)?;

        // Set timeouts
        socket.set_opt::<nng::options::SendTimeout>(Some(Duration::from_secs(5)))?;
        socket.set_opt::<nng::options::RecvTimeout>(Some(Duration::from_secs(30)))?;

        Ok(Client {
            socket: Arc::new(socket),
        })
    }

    /// Send a request and receive response (async via spawn_blocking)
    pub async fn request(&self, request: &Request) -> Result<Response> {
        let data = serde_json::to_vec(request)?;

        // Send (blocking operation wrapped in spawn_blocking)
        let socket = Arc::clone(&self.socket);
        let data_clone = data.clone();
        tokio::task::spawn_blocking(move || socket.send(&data_clone).map_err(|(_, e)| e)).await??;

        // Receive (blocking operation wrapped in spawn_blocking)
        let socket = Arc::clone(&self.socket);
        let msg = tokio::task::spawn_blocking(move || socket.recv()).await??;

        let response: Response = serde_json::from_slice(&msg)?;
        Ok(response)
    }

    /// Ping the server
    pub async fn ping(&self) -> Result<bool> {
        match self.request(&Request::Ping).await? {
            Response::Pong => Ok(true),
            _ => Ok(false),
        }
    }

    /// Compile source code
    pub async fn compile(&self, source: &str) -> Result<Response> {
        self.request(&Request::Compile {
            source: source.to_string(),
        })
        .await
    }

    /// Run a backtest
    pub async fn run(&self, source: &str) -> Result<Response> {
        self.request(&Request::Run {
            source: source.to_string(),
        })
        .await
    }

    /// Get server status
    pub async fn status(&self) -> Result<Response> {
        self.request(&Request::Status).await
    }

    /// Shutdown the server
    pub async fn shutdown(&self) -> Result<Response> {
        self.request(&Request::Shutdown).await
    }
}

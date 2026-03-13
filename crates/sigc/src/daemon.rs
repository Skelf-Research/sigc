//! Daemon server for sigc RPC
//!
//! Provides a REQ/REP server using nng for remote compilation and execution.

use anyhow::Result;
use nng::{options::Options, Protocol, Socket};
use serde::{Deserialize, Serialize};
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

/// Daemon server state
pub struct Daemon {
    socket: Socket,
    compiler: sig_compiler::Compiler,
    runtime: sig_runtime::Runtime,
    start_time: std::time::Instant,
    requests_handled: u64,
}

impl Daemon {
    /// Create a new daemon server
    pub fn new(address: &str) -> Result<Self> {
        let socket = Socket::new(Protocol::Rep0)?;
        socket.listen(address)?;

        // Set receive timeout
        socket.set_opt::<nng::options::RecvTimeout>(Some(Duration::from_millis(100)))?;

        tracing::info!("Daemon listening on {}", address);

        Ok(Daemon {
            socket,
            compiler: sig_compiler::Compiler::new(),
            runtime: sig_runtime::Runtime::new(),
            start_time: std::time::Instant::now(),
            requests_handled: 0,
        })
    }

    /// Run the daemon server loop
    pub fn run(&mut self) -> Result<()> {
        tracing::info!("Daemon server started");

        loop {
            // Try to receive a message
            let msg = match self.socket.recv() {
                Ok(msg) => msg,
                Err(nng::Error::TimedOut) => continue,
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
                    self.send_response(&response)?;
                    continue;
                }
            };

            tracing::debug!("Received request: {:?}", request);
            self.requests_handled += 1;

            // Handle request
            let response = self.handle_request(request);

            // Check for shutdown
            if matches!(response, Response::Shutdown) {
                self.send_response(&response)?;
                tracing::info!("Daemon shutting down");
                break;
            }

            self.send_response(&response)?;
        }

        Ok(())
    }

    fn send_response(&self, response: &Response) -> Result<()> {
        let data = serde_json::to_vec(response)?;
        self.socket.send(&data).map_err(|(_, e)| e)?;
        Ok(())
    }

    fn handle_request(&mut self, request: Request) -> Response {
        match request {
            Request::Ping => Response::Pong,

            Request::Compile { source } => match self.compiler.compile(&source) {
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
                let ir = match self.compiler.compile(&source) {
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

                // Then run
                match self.runtime.run_ir(&ir) {
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
                uptime_secs: self.start_time.elapsed().as_secs(),
                requests_handled: self.requests_handled,
            },

            Request::Shutdown => Response::Shutdown,
        }
    }
}

/// Client for connecting to the daemon
pub struct Client {
    socket: Socket,
}

impl Client {
    /// Connect to a daemon server
    pub fn connect(address: &str) -> Result<Self> {
        let socket = Socket::new(Protocol::Req0)?;
        socket.dial(address)?;

        // Set timeouts
        socket.set_opt::<nng::options::SendTimeout>(Some(Duration::from_secs(5)))?;
        socket.set_opt::<nng::options::RecvTimeout>(Some(Duration::from_secs(30)))?;

        Ok(Client { socket })
    }

    /// Send a request and receive response
    pub fn request(&self, request: &Request) -> Result<Response> {
        let data = serde_json::to_vec(request)?;
        self.socket.send(&data).map_err(|(_, e)| e)?;

        let msg = self.socket.recv()?;
        let response: Response = serde_json::from_slice(&msg)?;
        Ok(response)
    }

    /// Ping the server
    pub fn ping(&self) -> Result<bool> {
        match self.request(&Request::Ping)? {
            Response::Pong => Ok(true),
            _ => Ok(false),
        }
    }

    /// Compile source code
    pub fn compile(&self, source: &str) -> Result<Response> {
        self.request(&Request::Compile {
            source: source.to_string(),
        })
    }

    /// Run a backtest
    pub fn run(&self, source: &str) -> Result<Response> {
        self.request(&Request::Run {
            source: source.to_string(),
        })
    }

    /// Get server status
    pub fn status(&self) -> Result<Response> {
        self.request(&Request::Status)
    }

    /// Shutdown the server
    pub fn shutdown(&self) -> Result<Response> {
        self.request(&Request::Shutdown)
    }
}

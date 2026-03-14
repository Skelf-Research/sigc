//! sigc - Signal Compiler CLI
//!
//! A single binary that exposes subcommands for compiling and running signals.

mod daemon;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// ANSI color codes for better error display
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

fn print_error(msg: &str) {
    eprintln!("{}{}error:{} {}", BOLD, RED, RESET, msg);
}

fn print_success(msg: &str) {
    println!("{}{}✓{} {}", BOLD, GREEN, RESET, msg);
}

fn print_info(msg: &str) {
    println!("{}{}info:{} {}", BOLD, BLUE, RESET, msg);
}

fn print_warning(msg: &str) {
    eprintln!("{}{}warning:{} {}", BOLD, YELLOW, RESET, msg);
}

/// Try to open cache with retries (brokerless concurrency handling)
fn try_open_cache_with_retry(cache_dir: &PathBuf, max_attempts: u32) -> Result<sig_cache::Cache> {
    use std::thread;
    use std::time::Duration;

    for attempt in 1..=max_attempts {
        match sig_cache::Cache::open(cache_dir) {
            Ok(cache) => {
                if attempt > 1 {
                    tracing::debug!("Cache opened successfully on attempt {}", attempt);
                }
                return Ok(cache);
            }
            Err(e) => {
                let is_lock_error = e.to_string().contains("lock") || e.to_string().contains("WouldBlock");

                if is_lock_error && attempt < max_attempts {
                    // Exponential backoff: 100ms, 200ms, 400ms...
                    let wait_ms = 100 * 2_u64.pow(attempt - 1);
                    tracing::debug!(
                        "Cache lock conflict (attempt {}/{}), retrying in {}ms...",
                        attempt, max_attempts, wait_ms
                    );
                    thread::sleep(Duration::from_millis(wait_ms));
                } else {
                    return Err(e.into());
                }
            }
        }
    }

    Err(anyhow::anyhow!("Failed to open cache after {} attempts", max_attempts))
}

#[derive(Parser)]
#[command(name = "sigc")]
#[command(version, about = "Signal compiler and backtester", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a .sig file to IR
    Compile {
        /// Input .sig file
        input: PathBuf,

        /// Output file for serialized IR
        #[arg(short, long)]
        emit: Option<PathBuf>,
    },

    /// Run a signal (compile + execute)
    Run {
        /// Input .sig file
        input: PathBuf,

        /// Output report file (json or csv)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Start the daemon server
    Daemon {
        /// Listen address (tcp://host:port)
        #[arg(long, default_value = "tcp://127.0.0.1:7240")]
        listen: String,

        /// Number of concurrent runtime workers (defaults to CPU count)
        #[arg(long)]
        workers: Option<usize>,
    },

    /// Send request to daemon
    Request {
        /// Daemon address
        #[arg(long, default_value = "tcp://127.0.0.1:7240")]
        addr: String,

        #[command(subcommand)]
        action: RequestAction,
    },

    /// Explain a compiled signal's IR
    Explain {
        /// Input .sig file or IR artifact
        input: PathBuf,
    },

    /// Show differences between two runs
    Diff {
        /// First artifact
        a: PathBuf,
        /// Second artifact
        b: PathBuf,
    },

    /// Cache management
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
}

#[derive(Subcommand)]
enum CacheAction {
    /// Show cache statistics
    Stats,
    /// Verify cache integrity
    Verify,
    /// Clear all cached artifacts
    Clear,
}

#[derive(Subcommand)]
enum RequestAction {
    /// Ping the daemon
    Ping,
    /// Compile a file via daemon
    Compile { input: PathBuf },
    /// Run a backtest via daemon
    Run { input: PathBuf },
    /// Get daemon status
    Status,
    /// Shutdown the daemon
    Shutdown,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    tracing::info!("sigc v{}", env!("CARGO_PKG_VERSION"));

    // Handle Request commands without cache (they connect to daemon via async)
    if let Some(Commands::Request { addr, action }) = cli.command {
        let runtime = tokio::runtime::Runtime::new()?;
        return runtime.block_on(async {
            let client = daemon::Client::new(&addr)?;

            match action {
                RequestAction::Ping => {
                    if client.ping().await? {
                        println!("Pong!");
                    } else {
                        println!("No response");
                    }
                }

                RequestAction::Compile { input } => {
                    let source = std::fs::read_to_string(&input)?;
                    match client.compile(&source).await? {
                        daemon::Response::CompileResult { success, nodes, error } => {
                            if success {
                                println!("Compiled successfully: {} nodes", nodes);
                            } else {
                                println!("Compilation failed: {}", error.unwrap_or_default());
                            }
                        }
                        _ => println!("Unexpected response"),
                    }
                }

                RequestAction::Run { input } => {
                    let source = std::fs::read_to_string(&input)?;
                    match client.run(&source).await? {
                        daemon::Response::RunResult { success, total_return, sharpe_ratio, max_drawdown, error } => {
                            if success {
                                println!();
                                println!("=== Backtest Results (via daemon) ===");
                                println!("Total Return:      {:>8.2}%", total_return * 100.0);
                                println!("Sharpe Ratio:      {:>8.2}", sharpe_ratio);
                                println!("Max Drawdown:      {:>8.2}%", max_drawdown * 100.0);
                                println!();
                            } else {
                                println!("Run failed: {}", error.unwrap_or_default());
                            }
                        }
                        _ => println!("Unexpected response"),
                    }
                }

                RequestAction::Status => {
                    match client.status().await? {
                        daemon::Response::Status { version, uptime_secs, requests_handled } => {
                            println!("Daemon Status:");
                            println!("  Version:  {}", version);
                            println!("  Uptime:   {}s", uptime_secs);
                            println!("  Requests: {}", requests_handled);
                        }
                        _ => println!("Unexpected response"),
                    }
                }

                RequestAction::Shutdown => {
                    client.shutdown().await?;
                    println!("Daemon shutdown requested");
                }
            }
            Ok(())
        });
    }

    // Initialize cache for other commands (only in standalone mode)
    // In brokerless architecture, cache should only be opened when daemon is NOT running
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sigc");

    // Try to open cache with retry/timeout to avoid lock conflicts
    let cache = match try_open_cache_with_retry(&cache_dir, 3) {
        Ok(c) => {
            tracing::debug!("Cache directory: {}", cache_dir.display());
            c
        }
        Err(e) => {
            // If cache is locked, suggest using daemon mode
            if e.to_string().contains("lock") || e.to_string().contains("WouldBlock") {
                print_error("Cache is locked (daemon may be running). Use 'sigc request' commands to communicate with daemon.");
                print_info("Example: sigc request compile <file>");
                std::process::exit(1);
            }
            return Err(e);
        }
    };

    match cli.command {
        Some(Commands::Compile { input, emit }) => {
            print_info(&format!("Compiling: {}", input.display()));

            let source = match std::fs::read_to_string(&input) {
                Ok(s) => s,
                Err(e) => {
                    print_error(&format!("Failed to read file '{}': {}", input.display(), e));
                    std::process::exit(1);
                }
            };

            let compiler = sig_compiler::Compiler::with_cache(cache);
            let ir = match compiler.compile(&source) {
                Ok(ir) => ir,
                Err(e) => {
                    print_error(&format!("Compilation failed:\n{}", e));
                    std::process::exit(1);
                }
            };

            if let Some(output_path) = emit {
                print_info(&format!("Would emit IR to: {}", output_path.display()));
            }

            print_success(&format!("Compiled {} nodes, {} outputs", ir.nodes.len(), ir.outputs.len()));
        }

        Some(Commands::Run { input, output }) => {
            print_info(&format!("Running: {}", input.display()));

            let source = match std::fs::read_to_string(&input) {
                Ok(s) => s,
                Err(e) => {
                    print_error(&format!("Failed to read file '{}': {}", input.display(), e));
                    std::process::exit(1);
                }
            };

            let compiler = sig_compiler::Compiler::new();
            let ir = match compiler.compile(&source) {
                Ok(ir) => ir,
                Err(e) => {
                    print_error(&format!("Compilation failed:\n{}", e));
                    std::process::exit(1);
                }
            };

            let mut runtime = sig_runtime::Runtime::with_cache(cache);
            let report = match runtime.run_ir(&ir) {
                Ok(r) => r,
                Err(e) => {
                    print_error(&format!("Execution failed: {}", e));
                    std::process::exit(1);
                }
            };

            // Display results with color
            println!();
            println!("{}{}=== Backtest Results ==={}", BOLD, GREEN, RESET);
            println!();

            let ret_color = if report.metrics.total_return >= 0.0 { GREEN } else { RED };
            println!("  Total Return:      {}{:>8.2}%{}",
                ret_color, report.metrics.total_return * 100.0, RESET);
            println!("  Annualized Return: {}{:>8.2}%{}",
                ret_color, report.metrics.annualized_return * 100.0, RESET);

            let sharpe_color = if report.metrics.sharpe_ratio >= 1.0 { GREEN }
                else if report.metrics.sharpe_ratio >= 0.0 { YELLOW }
                else { RED };
            println!("  Sharpe Ratio:      {}{:>8.2}{}",
                sharpe_color, report.metrics.sharpe_ratio, RESET);

            let dd_color = if report.metrics.max_drawdown <= 0.1 { GREEN }
                else if report.metrics.max_drawdown <= 0.2 { YELLOW }
                else { RED };
            println!("  Max Drawdown:      {}{:>8.2}%{}",
                dd_color, report.metrics.max_drawdown * 100.0, RESET);

            println!("  Turnover:          {:>8.2}%", report.metrics.turnover * 100.0);
            println!();

            // Export report if requested
            if let Some(output_path) = output {
                let ext = output_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                match ext {
                    "json" => {
                        let json = serde_json::json!({
                            "source": input.to_string_lossy(),
                            "metrics": {
                                "total_return": report.metrics.total_return,
                                "annualized_return": report.metrics.annualized_return,
                                "sharpe_ratio": report.metrics.sharpe_ratio,
                                "max_drawdown": report.metrics.max_drawdown,
                                "turnover": report.metrics.turnover
                            },
                            "executed_at": report.executed_at
                        });
                        std::fs::write(&output_path, serde_json::to_string_pretty(&json)?)?;
                        print_success(&format!("Report exported to: {}", output_path.display()));
                    }
                    "csv" => {
                        let csv = format!(
                            "metric,value\ntotal_return,{}\nannualized_return,{}\nsharpe_ratio,{}\nmax_drawdown,{}\nturnover,{}\n",
                            report.metrics.total_return,
                            report.metrics.annualized_return,
                            report.metrics.sharpe_ratio,
                            report.metrics.max_drawdown,
                            report.metrics.turnover
                        );
                        std::fs::write(&output_path, csv)?;
                        print_success(&format!("Report exported to: {}", output_path.display()));
                    }
                    _ => {
                        print_warning(&format!("Unknown output format: {}", ext));
                    }
                }
            }
        }

        Some(Commands::Daemon { listen, workers }) => {
            tracing::info!("Starting daemon on {}", listen);
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async {
                let mut daemon = if let Some(pool_size) = workers {
                    tracing::info!("Using {} runtime workers", pool_size);
                    daemon::Daemon::with_pool_size(&listen, pool_size)?
                } else {
                    daemon::Daemon::new(&listen)?
                };
                daemon.run().await
            })?;
        }

        Some(Commands::Request { .. }) => {
            // Already handled above
            unreachable!()
        }

        Some(Commands::Explain { input }) => {
            tracing::info!("Explaining: {}", input.display());
            let source = std::fs::read_to_string(&input)?;
            let compiler = sig_compiler::Compiler::new();
            let ir = compiler.compile(&source)?;

            println!();
            println!("=== IR Explanation ===");
            println!("Source: {}", input.display());
            println!("Nodes:  {}", ir.nodes.len());
            println!("Outputs: {}", ir.outputs.len());
            println!();

            println!("Node Graph:");
            for node in &ir.nodes {
                let inputs_str = node.inputs.iter()
                    .map(|i| format!("#{}", i))
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("  #{}: {:?} [{}] -> {:?}",
                    node.id,
                    node.operator,
                    inputs_str,
                    node.type_info.dtype
                );
            }

            println!();
            println!("Outputs: {:?}", ir.outputs);
            println!();
        }

        Some(Commands::Diff { a, b }) => {
            tracing::info!("Diffing {} vs {}", a.display(), b.display());

            // Check if inputs are JSON reports or .sig files
            let ext_a = a.extension().and_then(|s| s.to_str()).unwrap_or("");
            let ext_b = b.extension().and_then(|s| s.to_str()).unwrap_or("");

            let (metrics_a, metrics_b) = if ext_a == "json" && ext_b == "json" {
                // Load from JSON reports
                let json_a: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&a)?)?;
                let json_b: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&b)?)?;

                let m_a = (
                    json_a["metrics"]["total_return"].as_f64().unwrap_or(0.0),
                    json_a["metrics"]["annualized_return"].as_f64().unwrap_or(0.0),
                    json_a["metrics"]["sharpe_ratio"].as_f64().unwrap_or(0.0),
                    json_a["metrics"]["max_drawdown"].as_f64().unwrap_or(0.0),
                    json_a["metrics"]["turnover"].as_f64().unwrap_or(0.0),
                );
                let m_b = (
                    json_b["metrics"]["total_return"].as_f64().unwrap_or(0.0),
                    json_b["metrics"]["annualized_return"].as_f64().unwrap_or(0.0),
                    json_b["metrics"]["sharpe_ratio"].as_f64().unwrap_or(0.0),
                    json_b["metrics"]["max_drawdown"].as_f64().unwrap_or(0.0),
                    json_b["metrics"]["turnover"].as_f64().unwrap_or(0.0),
                );
                (m_a, m_b)
            } else {
                // Run both .sig files
                let compiler = sig_compiler::Compiler::new();
                let mut runtime = sig_runtime::Runtime::new();

                let source_a = std::fs::read_to_string(&a)?;
                let ir_a = compiler.compile(&source_a)?;
                let report_a = runtime.run_ir(&ir_a)?;

                let source_b = std::fs::read_to_string(&b)?;
                let ir_b = compiler.compile(&source_b)?;
                let report_b = runtime.run_ir(&ir_b)?;

                let m_a = (
                    report_a.metrics.total_return,
                    report_a.metrics.annualized_return,
                    report_a.metrics.sharpe_ratio,
                    report_a.metrics.max_drawdown,
                    report_a.metrics.turnover,
                );
                let m_b = (
                    report_b.metrics.total_return,
                    report_b.metrics.annualized_return,
                    report_b.metrics.sharpe_ratio,
                    report_b.metrics.max_drawdown,
                    report_b.metrics.turnover,
                );
                (m_a, m_b)
            };

            // Display comparison
            println!();
            println!("=== Backtest Comparison ===");
            println!("A: {}", a.display());
            println!("B: {}", b.display());
            println!();
            println!("{:<20} {:>12} {:>12} {:>12}", "Metric", "A", "B", "Delta");
            println!("{}", "-".repeat(58));

            let metrics = [
                ("Total Return", metrics_a.0 * 100.0, metrics_b.0 * 100.0, "%"),
                ("Ann. Return", metrics_a.1 * 100.0, metrics_b.1 * 100.0, "%"),
                ("Sharpe Ratio", metrics_a.2, metrics_b.2, ""),
                ("Max Drawdown", metrics_a.3 * 100.0, metrics_b.3 * 100.0, "%"),
                ("Turnover", metrics_a.4 * 100.0, metrics_b.4 * 100.0, "%"),
            ];

            for (name, val_a, val_b, suffix) in metrics {
                let delta = val_b - val_a;
                let delta_str = if delta >= 0.0 {
                    format!("+{:.2}{}", delta, suffix)
                } else {
                    format!("{:.2}{}", delta, suffix)
                };
                println!("{:<20} {:>10.2}{} {:>10.2}{} {:>12}",
                    name, val_a, suffix, val_b, suffix, delta_str);
            }
            println!();
        }

        Some(Commands::Cache { action }) => match action {
            CacheAction::Stats => {
                let size = std::fs::read_dir(&cache_dir)
                    .map(|entries| entries.count())
                    .unwrap_or(0);
                println!("Cache Statistics:");
                println!("  Location: {}", cache_dir.display());
                println!("  Entries:  {}", size);
            }
            CacheAction::Verify => {
                println!("Cache verification: OK");
                println!("  Location: {}", cache_dir.display());
            }
            CacheAction::Clear => {
                if cache_dir.exists() {
                    std::fs::remove_dir_all(&cache_dir)?;
                    std::fs::create_dir_all(&cache_dir)?;
                    println!("Cache cleared");
                } else {
                    println!("Cache directory does not exist");
                }
            }
        },

        None => {
            println!("sigc v{}", env!("CARGO_PKG_VERSION"));
            println!("Signal compiler and backtester");
            println!();
            println!("Run 'sigc --help' for usage information");
        }
    }

    Ok(())
}


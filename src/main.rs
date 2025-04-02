// Change the use statement to match the crate name from the build error
use test_rust_mcp_sdk::server::run; // Use the crate name 'test_rust_mcp_sdk'

// Keep standard library/external crate imports needed for main
use anyhow::{Context, Result};
use tracing::{error, info, Level}; // Keep Level
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use home; // Keep home

#[tokio::main]
async fn main() -> Result<()> {
    // --- Tracing setup ---
    let home_dir = home::home_dir().context("failed to get user home directory")?;
    let log_directory = home_dir.join(".screenpipe").join("logs").join("rust_stdio_refactored_logs"); // Log dir name change
    std::fs::create_dir_all(&log_directory)
        .with_context(|| format!("failed to create log directory: {:?}", log_directory))?;
    let log_file_appender = tracing_appender::rolling::daily(log_directory, "rust_stdio_refactored.log"); // Log file name change
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(log_file_appender);

    // Set default log level more dynamically, e.g., via RUST_LOG env var
    let default_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info")); // Default to info if RUST_LOG not set

    tracing_subscriber::registry()
        .with(default_filter.add_directive(Level::DEBUG.into())) // Keep DEBUG hardcoded for now, or adjust as needed
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false) // No ANSI colors in log files
                .with_span_events(FmtSpan::NONE) // Don't include span events
                .with_writer(non_blocking_writer), // Write to the non-blocking file appender
        )
        // Optionally, add another layer for console logging if desired during development
        // .with(
        //     tracing_subscriber::fmt::layer()
        //         .with_ansi(true) // Enable ANSI colors for the console
        // )
        .init(); // Initialize the subscriber
    // --- End Tracing setup ---

    info!("starting mcp rust stdio server process...");

    // Call the imported run function directly
    if let Err(e) = run().await {
        error!("server exited with error: {:?}", e);
        // Consider exiting with a non-zero status code on error
        std::process::exit(1);
    } else {
         info!("server exited successfully.");
    }

    Ok(())
}

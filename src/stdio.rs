use anyhow::Result;
use serde::Serialize;
use tokio::io::{AsyncWriteExt, Stdout};
use tracing::debug;

/// Writes a JSON message to the writer (stdout), followed by a newline.
pub async fn write_message_newline(writer: &mut Stdout, message: &impl Serialize) -> Result<()> {
    let message_str = serde_json::to_string(message)?;
    debug!("sending raw json: {}", message_str); // Log the JSON being sent

    // Write the JSON string and then a newline character
    writer.write_all(message_str.as_bytes()).await?;
    writer.write_all(b"\n").await?; // Add newline delimiter
    writer.flush().await?; // Ensure data is sent immediately

    Ok(())
}

// Potentially add a read_message_newline function here later if needed

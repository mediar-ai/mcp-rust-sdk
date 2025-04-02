# Rust MCP Stdio Server Test

This repository contains a simple example of a [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server implemented in Rust using a newline-delimited JSON stdio transport.

https://github.com/user-attachments/assets/428319c2-fce2-4654-ab34-cb6987ee4164

## Status

This is currently a basic test implementation created for learning and debugging purposes. It handles the MCP initialization handshake and basic list commands (`tools/list`, `resources/list`, `prompts/list`) as well as a dummy tool call (`tools/call`).

## Usage

1.  Build the server:
    ```bash
    cargo build
    ```
2.  Configure an MCP client (like Claude Desktop) to launch the executable found at `target/debug/mcp-rust-sdk`. Ensure the client uses newline-delimited JSON for stdio communication.
    
    For example, in `claude_desktop_config.json` (or similar client configuration), you might add an entry like this, replacing the `command` path with the **absolute path** to your built executable:
    ```json
    {
        "mcpServers": {
            "rust_stdio_test": {
                "command": "/path/to/your/mcp-rust-sdk/target/debug/mcp-rust-sdk"
            }
            // ... other servers ...
        }
    }
    ```
    *(Remember to use the correct absolute path for your system)*

3.  Set the `RUST_LOG` environment variable (e.g., `RUST_LOG=debug` or `RUST_LOG=trace`) to control logging verbosity. Logs are written to `$HOME/.screenpipe/logs/rust_stdio_test_logs/`.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

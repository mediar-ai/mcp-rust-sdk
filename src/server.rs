use crate::{handlers, stdio, types}; // Use crate:: for sibling modules
use anyhow::Result;
use futures::StreamExt;
use serde_json::Value;
use tokio::io::{self, BufReader, Stdout};
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{debug, error, info, trace, warn};
use types::{GenericErrorResponse, GenericNotification, GenericRequest, GenericResponse, ServerCapabilities, Implementation}; // Bring specific types into scope

// Server state (could be expanded later)
struct ServerState {
    server_info: Implementation,
    server_capabilities: ServerCapabilities,
    // Add other stateful data here, e.g., initialized status, client capabilities
}

/// Runs the main server loop, handling MCP messages over stdio.
pub async fn run() -> Result<()> {
    let server_state = ServerState {
        server_info: Implementation {
            name: "rust-mcp-stdio-refactored".to_string(),
            version: "0.1.1".to_string(), // Updated version example
        },
        server_capabilities: ServerCapabilities {
            tools: Some(serde_json::json!({})),     // Indicate capability
            resources: Some(serde_json::json!({})), // Indicate capability
            prompts: Some(serde_json::json!({})),   // Indicate capability
        },
    };

    info!("rust stdio server starting...");
    info!("server info: {:?}", server_state.server_info);
    info!("server capabilities: {:?}", server_state.server_capabilities);

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut framed_reader = FramedRead::new(BufReader::new(stdin), LinesCodec::new());

    // Main message loop
    while let Some(line_result) = framed_reader.next().await {
        match line_result {
            Ok(line) => {
                trace!("received raw line: {}", line);
                if line.trim().is_empty() {
                    trace!("skipping empty line");
                    continue;
                }

                // Try parsing as a generic structure first to get id/method
                // Using if-let chain for clarity
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    if value.get("id").is_some() {
                        // Likely a Request
                        match serde_json::from_value::<GenericRequest>(value) {
                            Ok(request) => {
                                handle_request(&request, &server_state, &mut stdout).await?;
                            }
                            Err(e) => {
                                error!("failed to parse request: {}. line: '{}'", e, line);
                                // Try to get ID for error response, even if parsing failed partially
                                let id = serde_json::from_str::<Value>(&line).ok().and_then(|v| v.get("id").cloned()).unwrap_or(Value::Null);
                                let err_resp = handlers::parse_error(Some(id), &e.to_string());
                                if let Err(write_e) = stdio::write_message_newline(&mut stdout, &err_resp).await {
                                    error!("failed to write parse error response: {:?}", write_e);
                                    break; // Exit on write error
                                }
                            }
                        }
                    } else if value.get("method").is_some() {
                         // Likely a Notification (no ID)
                         match serde_json::from_value::<GenericNotification>(value) {
                             Ok(notification) => {
                                 handle_notification(&notification, &server_state, &mut stdout).await?;
                             }
                             Err(e) => {
                                 // Less critical to respond to notification parse errors, but log it.
                                 error!("failed to parse notification: {}. line: '{}'", e, line);
                                 // Optionally send a generic error if the protocol demands it, but often notifications are fire-and-forget.
                                 // For now, just log.
                            }
                         }
                    } else {
                         // Invalid JSON-RPC message (neither request nor notification)
                         error!("received invalid json-rpc message (no id or method): {}", line);
                         // Cannot respond meaningfully without an ID.
                    }
                } else {
                    // Totally invalid JSON
                    error!("failed to parse incoming line as json: '{}'", line);
                    let err_resp = handlers::parse_error(None, "Invalid JSON received"); // No ID possible
                    if let Err(write_e) = stdio::write_message_newline(&mut stdout, &err_resp).await {
                        error!("failed to write json parse error response: {:?}", write_e);
                        break; // Exit on write error
                    }
                }
            }
            Err(e) => {
                error!("error reading line from stdin: {:?}", e);
                break; // Exit loop on read error
            }
        }
    }

    info!("rust stdio server shutting down.");
    Ok(())
}


/// Handles dispatching of incoming requests based on method.
async fn handle_request(request: &GenericRequest, server_state: &ServerState, stdout: &mut Stdout) -> Result<()> {
    info!("received request: id={}, method={}", request.id, request.method);
    debug!("request details: {:?}", request);

    let response_result: Result<Value, GenericErrorResponse> = match request.method.as_str() {
        "initialize" => {
            match request.params.clone() { // Clone params for deserialization
                Some(params_value) => {
                    match serde_json::from_value::<types::InitializeRequestParams>(params_value) {
                        Ok(params) => handlers::handle_initialize(params, &server_state.server_capabilities, &server_state.server_info)
                            .map(|result| serde_json::to_value(result).unwrap()) // Convert result to Value
                            .map_err(|e| handlers::invalid_params_error(request.id.clone(), "initialize", &e.to_string())), // Handler error -> RPC error
                        Err(e) => Err(handlers::invalid_params_error(request.id.clone(), "initialize", &e.to_string())),
                    }
                }
                None => Err(handlers::invalid_params_error(request.id.clone(), "initialize", "missing params field")),
            }
        }

        "tools/list" => {
             handlers::handle_list_tools()
                 .map(|result| serde_json::to_value(result).unwrap())
                 .map_err(|e| handlers::create_error_response(request.id.clone(), -32603, format!("Internal error during tools/list: {}", e))) // Generic internal error
        }

        "resources/list" => {
             handlers::handle_list_resources()
                 .map(|result| serde_json::to_value(result).unwrap())
                 .map_err(|e| handlers::create_error_response(request.id.clone(), -32603, format!("Internal error during resources/list: {}", e)))
        }

        "prompts/list" => {
             handlers::handle_list_prompts()
                 .map(|result| serde_json::to_value(result).unwrap())
                 .map_err(|e| handlers::create_error_response(request.id.clone(), -32603, format!("Internal error during prompts/list: {}", e)))
        }

        "tools/call" => {
             match request.params.clone() {
                Some(params_value) => {
                    match serde_json::from_value::<types::CallToolRequestParams>(params_value) {
                        Ok(params) => handlers::handle_call_tool(params)
                            .map(|result| serde_json::to_value(result).unwrap()) // Convert result to Value
                            .map_err(|e| handlers::create_error_response(request.id.clone(), -32603, format!("Internal error during tools/call: {}", e))), // Handler error -> RPC error
                        Err(e) => Err(handlers::invalid_params_error(request.id.clone(), "tools/call", &e.to_string())),
                    }
                }
                None => Err(handlers::invalid_params_error(request.id.clone(), "tools/call", "missing params field")),
             }
        }

        _ => {
            warn!("received unhandled request method: {}", request.method);
            Err(handlers::method_not_found_error(request.id.clone(), &request.method))
        }
    };

    // Send the response (either success or error)
    match response_result {
        Ok(result_value) => {
            let response = GenericResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: result_value,
            };
             if let Err(e) = stdio::write_message_newline(stdout, &response).await {
                 error!("failed to write success response for id {}: {:?}", request.id, e);
                 return Err(e.into()); // Propagate write error
             }
             info!("sent success response for id: {}", request.id);
        }
        Err(error_response) => {
             if let Err(e) = stdio::write_message_newline(stdout, &error_response).await {
                 error!("failed to write error response for id {}: {:?}", request.id, e);
                  return Err(e.into()); // Propagate write error
             }
             info!("sent error response for id: {}", request.id);
        }
    }

    Ok(())
}

/// Handles dispatching of incoming notifications based on method.
async fn handle_notification(notification: &GenericNotification, _server_state: &ServerState, _stdout: &mut Stdout) -> Result<()> {
    info!("received notification: method={}", notification.method);
    debug!("notification details: {:?}", notification);

    match notification.method.as_str() {
         "initialized" => {
              match notification.params.clone() {
                  Some(params_value) => {
                      match serde_json::from_value::<types::InitializedNotificationParams>(params_value) {
                           Ok(params) => {
                               if let Err(e) = handlers::handle_initialized(params) {
                                    error!("error handling 'initialized' notification: {:?}", e);
                                    // Decide if an error here is critical enough to stop the server. Usually not for notifications.
                               }
                           },
                           Err(e) => {
                               error!("failed to parse 'initialized' params: {}. value: {:?}", e, notification.params);
                               // Cannot send JSON-RPC error response for notification parse error
                           }
                      }
                  },
                   None => {
                       // If params are expected but missing
                       warn!("'initialized' notification received without expected params (though none currently defined)");
                       // Handle as if params were empty/default if possible
                        if let Err(e) = handlers::handle_initialized(Default::default()) {
                            error!("error handling 'initialized' notification with default params: {:?}", e);
                        }
                   }
              }
         }
         // Add other notification handlers here like $/cancelRequest, etc.
         "$/cancelRequest" => {
            warn!("received '$/cancelRequest' notification, but cancellation is not implemented yet.");
            // TODO: Implement request cancellation logic if needed
         }
         _ => {
              warn!("received unhandled notification method: {}", notification.method);
         }
    }
    // Notifications typically don't have responses
    Ok(())
}

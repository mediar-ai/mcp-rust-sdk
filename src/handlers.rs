use crate::constants::SUPPORTED_PROTOCOL_VERSION;
use crate::types::{
    CallToolRequestParams, CallToolResult, ContentPart, InitializeRequestParams, InitializeResult,
    ListPromptsResult, ListResourcesResult, ListToolsResult, Prompt, Resource, ServerCapabilities,
    Implementation, Tool, ErrorData, InitializedNotificationParams,
};
use anyhow::Result; // Keep Result
use serde_json::Value;
use tracing::{debug, info, warn};

// --- Initialization Handler ---

pub fn handle_initialize(
    params: InitializeRequestParams,
    server_capabilities: &ServerCapabilities, // Pass capabilities
    server_info: &Implementation,         // Pass server info
) -> Result<InitializeResult> {
    info!(
        "handling initialize request: client={:?}, version={}",
        params.client_info, params.protocol_version
    );

    // Basic version check (could be more sophisticated)
    if params.protocol_version != SUPPORTED_PROTOCOL_VERSION {
        warn!(
            "client requested protocol version {}, but server uses {}",
            params.protocol_version, SUPPORTED_PROTOCOL_VERSION
        );
        // Respond with server's version regardless for now
    }

    // TODO: Store/use client capabilities (params.capabilities) if needed

    let result = InitializeResult {
        protocol_version: SUPPORTED_PROTOCOL_VERSION.to_string(),
        capabilities: server_capabilities.clone(), // Use passed capabilities
        server_info: server_info.clone(),         // Use passed server info
        instructions: None, // No specific instructions for now
    };

    Ok(result)
}

// --- Initialized Notification Handler ---
// This is a notification, so it doesn't return a result to send back.
// It might trigger internal state changes.
pub fn handle_initialized(_params: InitializedNotificationParams) -> Result<()> {
     info!("received 'initialized' notification from client. connection ready.");
     // Add any logic needed after initialization confirmation here
     Ok(())
}


// --- List Handlers ---

pub fn handle_list_tools() -> Result<ListToolsResult> {
    info!("handling tools/list request");

    // --- Create a dummy tool ---
    let dummy_tool = Tool {
        name: "dummy_tool_from_rust".to_string(),
        description: Some("A simple test tool.".to_string()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {} // No specific input properties for this dummy tool
        }),
    };
    // --- End dummy tool ---

    let result = ListToolsResult {
        tools: vec![dummy_tool], // Send the list with the dummy tool
    };
    Ok(result)
}

pub fn handle_list_resources() -> Result<ListResourcesResult> {
    info!("handling resources/list request");
    let dummy_resource = Resource {
        uri: "mcp://dummy/resource/1".to_string(),
        name: "Dummy Resource".to_string(),
        description: Some("A test resource from Rust".to_string()),
    };
    let result = ListResourcesResult {
        resources: vec![dummy_resource], // Send dummy
    };
    Ok(result)
}

pub fn handle_list_prompts() -> Result<ListPromptsResult> {
    info!("handling prompts/list request");
    let dummy_prompt = Prompt {
        name: "dummy_prompt".to_string(),
        description: Some("A test prompt from Rust".to_string()),
        arguments: None, // No args for simplicity
    };
    let result = ListPromptsResult {
        prompts: vec![dummy_prompt], // Send dummy
    };
    Ok(result)
}

// --- Tool Call Handler ---

pub fn handle_call_tool(params: CallToolRequestParams) -> Result<CallToolResult> {
    info!("handling tools/call request for tool: {}", params.name);
    debug!("tool call arguments: {:?}", params.arguments);

    // Check which tool is being called
    if params.name == "dummy_tool_from_rust" {
        // --- Execute Dummy Tool Logic ---
        info!(
            "executing dummy_tool_from_rust with args: {:?}",
            params.arguments
        );

        // Create a simple success result
        let result_content = ContentPart {
            type_: "text".to_string(),
            text: Some(format!(
                "dummy_tool_from_rust executed successfully by Rust! Received args: {}",
                params.arguments
            )),
        };
        let tool_result = CallToolResult {
            content: vec![result_content],
            is_error: None, // Indicate success
        };
        Ok(tool_result)
        // --- End Dummy Tool Logic ---
    } else {
        // Handle calls to unknown tools by returning an error *within* the result structure
        warn!("received call for unknown tool: {}", params.name);
        let error_content = ContentPart {
            type_: "text".to_string(),
            text: Some(format!(
                "Error: Tool '{}' not implemented by this server.",
                params.name
            )),
        };
        let tool_result = CallToolResult {
            content: vec![error_content],
            is_error: Some(true), // Indicate tool execution error
        };
        Ok(tool_result) // Still Ok from the handler's perspective, error is in the result
    }
}

// --- Generic Error Creation ---
// Helper to create standard JSON-RPC error responses

pub fn create_error_response(id: Value, code: i32, message: String) -> crate::types::GenericErrorResponse {
    crate::types::GenericErrorResponse {
        jsonrpc: "2.0".to_string(),
        id,
        error: ErrorData { code, message },
    }
}

pub fn method_not_found_error(id: Value, method_name: &str) -> crate::types::GenericErrorResponse {
    create_error_response(id, -32601, format!("Method not found: {}", method_name))
}

pub fn invalid_params_error(id: Value, method_name: &str, details: &str) -> crate::types::GenericErrorResponse {
     create_error_response(id, -32602, format!("Invalid params for {}: {}", method_name, details))
}

pub fn parse_error(id: Option<Value>, details: &str) -> crate::types::GenericErrorResponse {
     create_error_response(id.unwrap_or(Value::Null), -32700, format!("Parse error: {}", details))
}

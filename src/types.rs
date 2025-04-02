use serde::{Deserialize, Serialize};
use serde_json::Value;

// --- MCP Type Definitions ---

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub tools: Option<Value>,
    pub resources: Option<Value>,
    pub prompts: Option<Value>,
    // Add other capabilities like logging, sampling as needed
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    // Define client capabilities if needed for logic later
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequestParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: Implementation,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: Implementation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GenericRequest {
    pub jsonrpc: String,
    pub id: Value, // Use Value for flexibility (can be number or string)
    pub method: String,
    // We'll deserialize params separately based on method
    pub params: Option<Value>,
}

#[derive(Serialize, Debug)]
pub struct GenericResponse<T> {
    pub jsonrpc: String,
    pub id: Value,
    pub result: T,
}

#[derive(Serialize, Debug)]
pub struct ErrorData {
    pub code: i32,
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct GenericErrorResponse {
    pub jsonrpc: String,
    pub id: Value,
    pub error: ErrorData,
}

// --- MCP Data Structures ---

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub uri: String, // Resource URI
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PromptArgument {
     pub name: String,
     #[serde(skip_serializing_if = "Option::is_none")]
     pub description: Option<String>,
     pub required: bool,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
     #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

// --- MCP Response Types for Lists ---

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListToolsResult {
    pub tools: Vec<Tool>, // Use the specific Tool struct
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesResult {
    pub resources: Vec<Resource>, // Use the specific Resource struct
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListPromptsResult {
    pub prompts: Vec<Prompt>, // Use the specific Prompt struct
}

// --- Tool Call Specific Structs ---

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CallToolRequestParams {
    pub name: String,      // Name of the tool being called
    pub arguments: Value,  // Arguments for the tool (use Value for flexibility)
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentPart {
    #[serde(rename = "type")] // Need to rename the field 'type'
    pub type_: String, // e.g., "text", "image", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    // ... other potential fields like uri, language, etc.
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CallToolResult {
    pub content: Vec<ContentPart>, // Result content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,    // Optional flag for tool errors
}

// --- Notification Structs (Example: Initialized) ---
// While "initialized" doesn't have specific params in the current spec,
// defining a struct can be useful for consistency if params are added later.
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitializedNotificationParams {
    // Currently empty, but could hold info in future protocol versions
}

// Generic Notification struct (similar to GenericRequest but no ID expected in response)
#[derive(Deserialize, Debug)]
pub struct GenericNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

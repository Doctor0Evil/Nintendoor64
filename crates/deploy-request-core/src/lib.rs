use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeployRequest {
    pub id: String,
    pub workflow: String,            // GitHub Actions workflow file name
    pub environment: String,        // "staging", "prod", ...
    pub refspec: String,            // branch or tag
    pub requester: String,          // "sonia-ai"
    pub reason: String,
    pub parameters: serde_json::Value,
}

use std::io::{Read, Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use schemars::JsonSchema;

use n64_build_maintenance::{
    MaintenanceConfirmRequest,
    MaintenanceConfirmResponse,
    MaintenancePreviewRequest,
    MaintenancePreviewResponse,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct SoniaEnvelope<T> {
    pub version: u32,
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<SoniaError>,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct SoniaError {
    pub code: String,
    pub message: String,
}

fn main() -> Result<()> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    let v: Value = serde_json::from_str(&buf)?;
    let cmd = v
        .get("command")
        .and_then(|c| c.as_str())
        .unwrap_or_default()
        .to_string();
    let version = v
        .get("version")
        .and_then(|x| x.as_u64())
        .unwrap_or(1) as u32;

    match cmd.as_str() {
        "maintenance.preview" => {
            let params: MaintenancePreviewRequest =
                serde_json::from_value(v.get("params").cloned().unwrap_or(Value::Null))?;
            match n64_build_maintenance::preview(params) {
                Ok(resp) => write_ok(version, "maintenance.preview", &resp)?,
                Err(e) => write_err(version, "MaintenancePreviewFailed", &e.to_string())?,
            }
        }
        "maintenance.confirm" => {
            let params: MaintenanceConfirmRequest =
                serde_json::from_value(v.get("params").cloned().unwrap_or(Value::Null))?;
            match n64_build_maintenance::confirm(params) {
                Ok(resp) => write_ok(version, "maintenance.confirm", &resp)?,
                Err(e) => write_err(version, "MaintenanceConfirmFailed", &e.to_string())?,
            }
        }
        _ => {
            write_err(
                version,
                "UnknownCommand",
                &format!("Unknown command '{}'", cmd),
            )?;
        }
    }

    Ok(())
}

fn write_ok<T: Serialize>(version: u32, command: &str, data: &T) -> Result<()> {
    let env = SoniaEnvelope {
        version,
        command: command.to_string(),
        params: Option::<Value>::None,
        data: Some(serde_json::to_value(data)?),
        error: None,
        status: "ok".to_string(),
    };
    let mut out = std::io::stdout();
    writeln!(out, "{}", serde_json::to_string_pretty(&env)?)?;
    Ok(())
}

fn write_err(version: u32, code: &str, message: &str) -> Result<()> {
    let env: SoniaEnvelope<Value> = SoniaEnvelope {
        version,
        command: "error".to_string(),
        params: None,
        data: None,
        error: Some(SoniaError {
            code: code.to_string(),
            message: message.to_string(),
        }),
        status: "error".to_string(),
    };
    let mut out = std::io::stdout();
    writeln!(out, "{}", serde_json::to_string_pretty(&env)?)?;
    Ok(())
}

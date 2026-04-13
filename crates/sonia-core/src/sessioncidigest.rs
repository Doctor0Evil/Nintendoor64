// crates/sonia-core/src/sessioncidigest.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CiFailure {
    pub crate_name: String,
    pub kind: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub log_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CiStatus {
    pub failures: Vec<CiFailure>,
    // other fields omitted
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RustcJsonMessage {
    #[serde(default)]
    message: RustcMessageInner,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RustcMessageInner {
    #[serde(default)]
    code: Option<RustcMessageCode>,
    #[serde(default)]
    level: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    spans: Vec<RustcSpan>,
    #[serde(default)]
    rendered: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RustcMessageCode {
    #[serde(default)]
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RustcSpan {
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    line_start: u32,
    #[serde(default)]
    column_start: u32,
    // line_end/column_end elided for brevity
}

/// Detects procedural macro panics from a rustc --error-format=json stream and
/// appends them as CiFailure entries with kind = "ProcMacroPanic".
pub fn ingest_proc_macro_panics_from_rustc_json(
    status: &mut CiStatus,
    crate_name: &str,
    json_stream: &str,
) {
    for line in json_stream.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parsed: Result<RustcJsonMessage, _> = serde_json::from_str(line);
        let Ok(msg) = parsed else {
            continue;
        };

        let inner = msg.message;

        // We treat any "error" that mentions "proc macro panicked" or
        // similar as a ProcMacroPanic. You can refine this by inspecting
        // inner.code or macro_backtrace when available.
        if inner.level != "error" {
            continue;
        }

        let text = inner.message.to_lowercase();
        let is_macro_panic = text.contains("proc macro panicked")
            || text.contains("proc-macro derive panicked")
            || text.contains("custom attribute panicked")
            || text.contains("macro expansion panicked");

        if !is_macro_panic {
            continue;
        }

        let primary_span = inner
            .spans
            .iter()
            .find(|s| !s.file_name.is_empty())
            .cloned();

        let (file, line, column) = if let Some(span) = primary_span {
            (
                Some(span.file_name),
                Some(span.line_start),
                Some(span.column_start),
            )
        } else {
            (None, None, None)
        };

        let mut msg_text = inner.message;
        if let Some(rendered) = inner.rendered {
            if !rendered.is_empty() {
                msg_text.push_str("\n\n");
                msg_text.push_str(&rendered);
            }
        }

        status.failures.push(CiFailure {
            crate_name: crate_name.to_string(),
            kind: "ProcMacroPanic".to_string(),
            message: msg_text,
            file,
            line,
            column,
            log_url: None,
        });
    }
}

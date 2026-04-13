use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlameHunk {
    pub start_line: u32,
    pub end_line: u32,
    pub author_kind: String, // "human" | "ai-bot" | "unknown"
    pub commit_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlameMeta {
    pub path: String,
    pub hunks: Vec<BlameHunk>,
}

pub fn blame_meta(path: &str) -> anyhow::Result<BlameMeta> {
    let output = Command::new("git")
        .arg("blame")
        .arg("--line-porcelain")
        .arg(path)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("git blame failed for {path}");
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut hunks = Vec::new();

    // Extremely simple: treat whole file as one hunk with the last commit seen.
    let mut last_commit = String::new();
    let mut last_author = String::new();
    let mut line_no: u32 = 0;

    for line in text.lines() {
        if line.starts_with(|c: char| c.is_ascii_hexdigit()) {
            let parts: Vec<_> = line.split_whitespace().collect();
            if let Some(hash) = parts.first() {
                last_commit = hash.to_string();
            }
        } else if line.starts_with("author ") {
            last_author = line["author ".len()..].to_string();
        } else if line.starts_with("\t") {
            line_no += 1;
        }
    }

    let author_kind = if last_author.contains("bot") || last_author.contains("ai") {
        "ai-bot"
    } else {
        "human"
    };

    hunks.push(BlameHunk {
        start_line: 1,
        end_line: line_no,
        author_kind: author_kind.to_string(),
        commit_id: last_commit,
    });

    Ok(BlameMeta {
        path: path.to_string(),
        hunks,
    })
}

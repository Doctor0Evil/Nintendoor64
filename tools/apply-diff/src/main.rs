use std::{fs, io::{self, Read}, path::PathBuf};
use anyhow::Result;
use apply_diff::{ApplyDiffRequest, ApplyDiffResult, HunkResult};

fn main() -> Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let req: ApplyDiffRequest = serde_json::from_str(&buf)?;

    let mut results = Vec::new();
    let mut had_error = false;

    for h in &req.hunks {
        let path = PathBuf::from(&h.file);
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                had_error = true;
                results.push(HunkResult {
                    file: h.file.clone(),
                    start_line: h.start_line,
                    end_line: h.end_line,
                    applied: false,
                    error: Some(format!("read error: {e}")),
                });
                continue;
            }
        };

        let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        let len = lines.len() as u32;

        if h.start_line == 0 || h.end_line < h.start_line || h.end_line > len {
            had_error = true;
            results.push(HunkResult {
                file: h.file.clone(),
                start_line: h.start_line,
                end_line: h.end_line,
                applied: false,
                error: Some("line range out of bounds".into()),
            });
            continue;
        }

        let start_idx = (h.start_line - 1) as usize;
        let end_idx = h.end_line as usize;

        let existing: Vec<&str> = lines[start_idx..end_idx].iter().map(|s| s.as_str()).collect();
        if existing != h.expected.iter().map(|s| s.as_str()).collect::<Vec<_>>() {
            had_error = true;
            results.push(HunkResult {
                file: h.file.clone(),
                start_line: h.start_line,
                end_line: h.end_line,
                applied: false,
                error: Some("expected text does not match file; refusing to apply".into()),
            });
            continue;
        }

        let mut out = Vec::new();
        out.extend_from_slice(&lines[..start_idx]);
        out.extend(h.replacement.clone());
        out.extend_from_slice(&lines[end_idx..]);

        fs::write(&path, out.join("\n"))?;

        results.push(HunkResult {
            file: h.file.clone(),
            start_line: h.start_line,
            end_line: h.end_line,
            applied: true,
            error: None,
        });
    }

    let res = ApplyDiffResult {
        status: if had_error { "error".into() } else { "ok".into() },
        hunks: results,
    };
    println!("{}", serde_json::to_string_pretty(&res)?);

    Ok(())
}

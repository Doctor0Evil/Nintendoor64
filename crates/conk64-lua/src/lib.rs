// crates/conk64-lua/src/lib.rs
use anyhow::Result;

pub fn wrap_lua(source: &str) -> String {
    format!(
r#"local user_chunk = nil

function user_logic()
{body}
end

function on_frame_advance()
    if not user_chunk then
        user_chunk = coroutine.create(user_logic)
    end
    if coroutine.status(user_chunk) ~= "dead" then
        local ok, err = coroutine.resume(user_chunk)
        if not ok then
            print("Conk64 script error: " .. tostring(err))
        end
    end
end
"#,
        body = indent(source, 4)
    )
}

fn indent(src: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    src.lines()
        .map(|line| format!("{pad}{line}", pad = pad, line = line))
        .collect::<Vec<_>>()
        .join("\n")
}

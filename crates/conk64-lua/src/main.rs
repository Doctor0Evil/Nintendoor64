// crates/conk64-lua/src/main.rs
use std::fs;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("missing --in");
    let output = args.next().expect("missing --out");

    let src = fs::read_to_string(&input)?;
    let wrapped = conk64_lua::wrap_lua(&src);
    fs::write(&output, wrapped)?;

    Ok(())
}

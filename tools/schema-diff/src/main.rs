use std::{fs, path::PathBuf};
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    old: PathBuf,
    #[arg(long)]
    new: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let old = fs::read_to_string(&args.old)?;
    let new = fs::read_to_string(&args.new)?;

    let diff = similar::TextDiff::from_lines(&old, &new);

    println!("### Schema diff: `{}` -> `{}`", args.old.display(), args.new.display());
    println!();
    println!("```diff");
    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            match change.tag() {
                similar::ChangeTag::Delete => print!("-"),
                similar::ChangeTag::Insert => print!("+"),
                similar::ChangeTag::Equal => print!(" "),
            }
            print!("{}", change);
        }
    }
    println!("```");

    Ok(())
}

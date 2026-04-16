// Destination: crates/conker-npc-check/src/main.rs

mod model;
mod scan;
mod checks;

use crate::model::InputPaths;
use anyhow::Result;
use std::env;
use std::path::PathBuf;

fn usage() {
    eprintln!(
        "conker-npc-check

Usage:
  conker-npc-check --maps <dir> --npcs <dir> --session <file>

Arguments:
  --maps     Directory containing Conker MapRecipe JSON files.
  --npcs     Directory containing NpcContract JSON files.
  --session  Session profile JSON describing active invariants.

Exit codes:
  0 on success (all checks passed).
  1 on validation or invariant failure.
  2 on usage or IO error."
    );
}

fn parse_args() -> Result<InputPaths> {
    let mut args = env::args().skip(1);
    let mut maps_dir: Option<PathBuf> = None;
    let mut npcs_dir: Option<PathBuf> = None;
    let mut session_file: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--maps" => {
                let v = args.next().ok_or_else(|| anyhow::anyhow!("missing value for --maps"))?;
                maps_dir = Some(PathBuf::from(v));
            }
            "--npcs" => {
                let v = args.next().ok_or_else(|| anyhow::anyhow!("missing value for --npcs"))?;
                npcs_dir = Some(PathBuf::from(v));
            }
            "--session" => {
                let v =
                    args.next().ok_or_else(|| anyhow::anyhow!("missing value for --session"))?;
                session_file = Some(PathBuf::from(v));
            }
            "-h" | "--help" => {
                usage();
                std::process::exit(0);
            }
            other => {
                return Err(anyhow::anyhow!("unknown argument: {other}"));
            }
        }
    }

    let maps_dir =
        maps_dir.ok_or_else(|| anyhow::anyhow!("--maps <dir> is required"))?;
    let npcs_dir =
        npcs_dir.ok_or_else(|| anyhow::anyhow!("--npcs <dir> is required"))?;
    let session_file =
        session_file.ok_or_else(|| anyhow::anyhow!("--session <file> is required"))?;

    Ok(InputPaths {
        maps_dir,
        npcs_dir,
        session_file,
    })
}

fn main() {
    let paths = match parse_args() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            usage();
            std::process::exit(2);
        }
    };

    match run(paths) {
        Ok(()) => {
            println!("conker-npc-check: all checks passed");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("conker-npc-check: FAILED\n{e}");
            std::process::exit(1);
        }
    }
}

fn run(paths: InputPaths) -> Result<()> {
    let (maps, npc_contracts, session) = scan::load_all(&paths)?;

    let report = checks::run_all_checks(&maps, &npc_contracts, &session)?;

    if report.errors.is_empty() {
        Ok(())
    } else {
        eprintln!("Found {} NPC contract issues:", report.errors.len());
        for err in &report.errors {
            eprintln!("- {err}");
        }
        Err(anyhow::anyhow!("NPC contract invariants failed"))
    }
}

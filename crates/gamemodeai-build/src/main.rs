use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::Parser;

mod contract;
use contract::{BuildContract, StepKind};

#[derive(Debug, Parser)]
#[command(name = "gamemodeai-build")]
#[command(about = "Top-level build orchestrator for GAMEMODE.ai retro and Nintendoor64 builds.")]
struct Cli {
    /// Path to the build contract JSON file
    #[arg(long)]
    contract: PathBuf,

    /// If set, only print planned commands without executing
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let contract_json =
        std::fs::read_to_string(&cli.contract).with_context(|| "reading build contract")?;
    let contract: BuildContract =
        serde_json::from_str(&contract_json).with_context(|| "parsing build contract JSON")?;

    // TODO: integrate jsonschema validation against gamemodeai.build.contract.schema.json
    // validate_contract_schema(&contract_json)?;

    let ordered_steps = topo_sort_steps(&contract)
        .with_context(|| "resolving build step order (cycle in depends_on?)")?;

    let workspace_root = Path::new(&contract.workspace_root).to_path_buf();
    let mut step_outputs: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for step in ordered_steps {
        if cli.dry_run {
            let cmd = build_command(&workspace_root, &contract, &step, &step_outputs)?;
            println!("# [dry-run] {}", format_command(&cmd));
            continue;
        }

        let mut cmd = build_command(&workspace_root, &contract, &step, &step_outputs)?;
        println!("> {}", format_command(&cmd));

        let status = cmd.status().with_context(|| "spawning tool")?;
        if !status.success() {
            return Err(anyhow!(
                "step {} failed with status {:?}",
                step.id,
                status.code()
            ));
        }

        // Record outputs for subsequent steps
        let mut out_map = BTreeMap::new();
        if let Some(rom) = &step.outputs.rom {
            out_map.insert("rom".to_string(), rom.clone());
        }
        if let Some(dir) = &step.outputs.assets_dir {
            out_map.insert("assets_dir".to_string(), dir.clone());
        }
        if let Some(dir) = &step.outputs.logs_dir {
            out_map.insert("logs_dir".to_string(), dir.clone());
        }
        for (k, v) in &step.outputs.extra {
            out_map.insert(k.clone(), v.clone());
        }
        step_outputs.insert(step.id.clone(), out_map);
    }

    // Final manifest (very small first pass)
    if let Some(manifest_path) = &contract.outputs.artifact_manifest {
        let manifest = serde_json::json!({
            "contract_id": contract.id,
            "target": format!("{:?}", contract.target),
            "rom": contract.outputs.rom,
            "steps": step_outputs,
        });
        let path = workspace_root.join(manifest_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&manifest)?)?;
    }

    println!("Final ROM: {}", contract.outputs.rom);
    Ok(())
}

fn topo_sort_steps(
    contract: &BuildContract,
) -> Result<Vec<contract::BuildStep>> {
    let mut id_to_step = BTreeMap::new();
    for step in &contract.steps {
        if id_to_step.insert(step.id.clone(), step.clone()).is_some() {
            return Err(anyhow!("duplicate step id {}", step.id));
        }
    }

    let mut indegree: BTreeMap<String, usize> = BTreeMap::new();
    let mut adj: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for step in &contract.steps {
        indegree.entry(step.id.clone()).or_insert(0);
        for dep in &step.depends_on {
            adj.entry(dep.clone())
                .or_default()
                .push(step.id.clone());
            *indegree.entry(step.id.clone()).or_insert(0) += 1;
        }
    }

    let mut queue = VecDeque::new();
    for (id, deg) in &indegree {
        if *deg == 0 {
            queue.push_back(id.clone());
        }
    }

    let mut ordered_ids = Vec::new();
    while let Some(id) = queue.pop_front() {
        ordered_ids.push(id.clone());
        if let Some(neighbors) = adj.get(&id) {
            for n in neighbors {
                if let Some(d) = indegree.get_mut(n) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(n.clone());
                    }
                }
            }
        }
    }

    if ordered_ids.len() != id_to_step.len() {
        let all_ids: BTreeSet<_> = id_to_step.keys().cloned().collect();
        let ordered_set: BTreeSet<_> = ordered_ids.iter().cloned().collect();
        let diff: Vec<_> = all_ids.difference(&ordered_set).cloned().collect();
        return Err(anyhow!(
            "cycle detected or unreachable steps: {:?}",
            diff
        ));
    }

    Ok(ordered_ids
        .into_iter()
        .map(|id| id_to_step.remove(&id).unwrap())
        .collect())
}

fn build_command(
    workspace_root: &Path,
    contract: &BuildContract,
    step: &contract::BuildStep,
    prev_outputs: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<Command> {
    let mut cmd = Command::new(&step.tool);
    cmd.current_dir(workspace_root);

    if let Some(env) = &step.env {
        for (k, v) in env {
            cmd.env(k, v);
        }
    }

    // Static args
    cmd.args(&step.args);

    // Tool-specific wiring based on StepKind
    match step.kind {
        StepKind::Retro => {
            // Example: retro-cli build-nes-map --recipe <recipe> --out-dir <dir>
            if let Some(recipe) = step.inputs.recipe.as_ref().or_else(|| Some(&contract.inputs.recipe)) {
                cmd.arg("build-from-recipe");
                cmd.arg("--recipe").arg(recipe);
            } else {
                return Err(anyhow!("retro step {} missing recipe input", step.id));
            }

            if let Some(assets_dir) = &step.outputs.assets_dir {
                cmd.arg("--out-dir").arg(assets_dir);
            }
        }
        StepKind::N64 => {
            // Example: n64-build --recipe <recipe> --out-rom <rom>
            if let Some(recipe) = step.inputs.recipe.as_ref().or_else(|| Some(&contract.inputs.recipe)) {
                cmd.arg("--recipe").arg(recipe);
            } else {
                return Err(anyhow!("n64 step {} missing recipe input", step.id));
            }

            if let Some(out_rom) = &step.outputs.rom {
                cmd.arg("--out-rom").arg(out_rom);
            } else {
                return Err(anyhow!("n64 step {} missing outputs.rom binding", step.id));
            }
        }
        StepKind::Starzip => {
            // Example: starzip-cli patch --layout <layout> --in-rom <rom> --out-rom <rom>
            cmd.arg("patch");

            // Layout
            let layout = step
                .inputs
                .layout
                .as_ref()
                .unwrap_or(&contract.inputs.layout);
            cmd.arg("--layout").arg(layout);

            // Input ROM: from contract, or from another step's outputs
            let in_rom = if let Some(rom) = &step.inputs.rom {
                rom.clone()
            } else {
                // look up last N64 step's rom
                let mut candidate: Option<String> = None;
                for dep in &step.depends_on {
                    if let Some(map) = prev_outputs.get(dep) {
                        if let Some(r) = map.get("rom") {
                            candidate = Some(r.clone());
                        }
                    }
                }
                candidate.ok_or_else(|| {
                    anyhow!("starzip step {} missing rom input", step.id)
                })?
            };
            cmd.arg("--in-rom").arg(in_rom);

            if let Some(out_rom) = &step.outputs.rom {
                cmd.arg("--out-rom").arg(out_rom);
            } else {
                return Err(anyhow!("starzip step {} missing outputs.rom binding", step.id));
            }

            if let Some(patch_spec) = &step.inputs.patch_spec {
                cmd.arg("--spec").arg(patch_spec);
            }
        }
        StepKind::Custom => {
            // Custom steps rely purely on tool, args, and explicit inputs/outputs.extra,
            // so we don't enforce wiring here.
        }
    }

    Ok(cmd)
}

fn format_command(cmd: &Command) -> String {
    let prog = cmd.get_program().to_string_lossy().to_string();
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();
    format!("{} {}", prog, args.join(" "))
}

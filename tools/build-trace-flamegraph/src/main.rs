use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildTrace {
    contract_id: Option<String>,
    workspaceroot: String,
    profile: String,
    started_at: String,
    ended_at: String,
    duration_ms: u64,
    events: Vec<BuildEvent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildEvent {
    contract_id: Option<String>,
    step_id: String,
    step_kind: Option<String>,
    tool: String,
    workdir: String,
    args: Vec<String>,
    env: Vec<EnvPair>,
    started_at: String,
    ended_at: String,
    duration_ms: u64,
    exit_code: Option<i32>,
    success: bool,
    stdout_digest: Option<LogDigest>,
    stderr_digest: Option<LogDigest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnvPair {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogDigest {
    head_lines: Vec<String>,
    total_lines: usize,
}

#[derive(clap::Parser, Debug)]
#[command(
    name = "build-trace-flamegraph",
    about = "Generate collapsed stacks and a flamegraph from gamemodeai-build traces."
)]
struct Cli {
    /// Path to build-trace.json (defaults to artifacts/meta/build-trace.json).
    #[arg(long, value_name = "PATH")]
    trace: Option<PathBuf>,

    /// Output directory for collapsed stacks and SVG.
    #[arg(long, value_name = "DIR", default_value = "artifacts/meta")]
    out_dir: PathBuf,

    /// Optional filter: only include events whose step_id contains this substring.
    #[arg(long, value_name = "FILTER")]
    step_filter: Option<String>,

    /// Optional filter: only include events whose tool name contains this substring.
    #[arg(long, value_name = "FILTER")]
    tool_filter: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let trace_path = cli
        .trace
        .clone()
        .unwrap_or_else(|| PathBuf::from("artifacts/meta/build-trace.json"));

    let trace_json = fs::read_to_string(&trace_path)
        .with_context(|| format!("reading build trace at {:?}", trace_path))?;
    let trace: BuildTrace =
        serde_json::from_str(&trace_json).with_context(|| "parsing build trace JSON")?;

    fs::create_dir_all(&cli.out_dir).with_context(|| "creating output directory")?;

    let collapsed_path = cli.out_dir.join("build-trace.collapsed");
    let svg_path = cli.out_dir.join("build-trace.svg");

    let collapsed = generate_collapsed(&trace, cli.step_filter.as_deref(), cli.tool_filter.as_deref());

    write_collapsed(&collapsed_path, &collapsed)?;
    write_svg(&collapsed_path, &svg_path)?;

    println!(
        "Wrote collapsed stacks to {:?} and flamegraph SVG to {:?}",
        collapsed_path, svg_path
    );

    Ok(())
}

fn generate_collapsed(
    trace: &BuildTrace,
    step_filter: Option<&str>,
    tool_filter: Option<&str>,
) -> Vec<(String, u64)> {
    // Map from stack string -> total samples (we use duration_ms as a proxy).
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();

    for ev in &trace.events {
        if let Some(f) = step_filter {
            if !ev.step_id.contains(f) {
                continue;
            }
        }
        if let Some(f) = tool_filter {
            if !ev.tool.contains(f) {
                continue;
            }
        }

        // Construct a simple stack like:
        //   contract:<id>;step:<step_id>;tool:<tool>  duration_ms
        let mut frames = Vec::new();

        if let Some(cid) = ev.contract_id.as_ref().or(trace.contract_id.as_ref()) {
            frames.push(format!("contract:{}", cid));
        }

        if let Some(kind) = ev.step_kind.as_ref() {
            frames.push(format!("step_kind:{}", kind));
        }

        frames.push(format!("step:{}", ev.step_id));
        frames.push(format!("tool:{}", ev.tool));

        let stack = frames.join(";");
        let entry = counts.entry(stack).or_insert(0);
        *entry = entry.saturating_add(ev.duration_ms.max(1));
    }

    counts.into_iter().collect()
}

fn write_collapsed(path: &Path, stacks: &[(String, u64)]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    for (stack, samples) in stacks {
        writeln!(file, "{} {}", stack, samples)?;
    }
    Ok(())
}

/// Generate an SVG flamegraph using `inferno`.
///
/// This expects `inferno` to be present in the build dependencies:
///   inferno = { version = "0.11", default-features = false, features = ["collapse", "flamegraph"] }
fn write_svg(collapsed_path: &Path, svg_path: &Path) -> anyhow::Result<()> {
    use inferno::flamegraph::{from_reader, Options};

    let mut opts = Options::default();
    opts.count_name = "ms".to_string();
    opts.title = "gamemodeai-build flamegraph".to_string();

    let mut input = fs::File::open(collapsed_path)
        .with_context(|| format!("opening collapsed stacks at {:?}", collapsed_path))?;
    let mut output = fs::File::create(svg_path)
        .with_context(|| format!("creating SVG at {:?}", svg_path))?;

    from_reader(&mut opts, &mut input, &mut output)
        .with_context(|| "generating flamegraph from collapsed stacks")?;

    Ok(())
}

// Small anyhow-style extension for better error messages without bringing in the full crate if you prefer.
trait Context<T> {
    fn with_context<F: FnOnce() -> String>(self, f: F) -> anyhow::Result<T>;
}

impl<T, E> Context<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<F: FnOnce() -> String>(self, f: F) -> anyhow::Result<T> {
        self.map_err(|e| anyhow::anyhow!("{}: {}", f(), e))
    }
}

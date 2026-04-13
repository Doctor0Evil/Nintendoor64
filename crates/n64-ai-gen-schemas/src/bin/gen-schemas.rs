// Nintendoor64/crates/n64-ai-gen-schemas/src/bin/gen-schemas.rs
//! CLI binary for generating JSON Schemas from Nintendoor64 Rust types.
//!
//! Usage:
//! ```bash
//! cargo run -p n64-ai-gen-schemas -- --output ./schemas --watch
//! ```

use clap::Parser;
use n64_ai_gen_schemas::{GeneratorConfig, SchemaGenerator};
use std::path::PathBuf;

#[cfg(feature = "full")]
use n64_ai_gen_schemas::register_schemas;

/// Schema generator CLI for Nintendoor64 contracts.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output directory for generated schema files
    #[arg(short, long, default_value = "schemas")]
    output: PathBuf,

    /// Include example values in generated schemas
    #[arg(long, default_value_t = true)]
    examples: bool,

    /// Emit a combined registry index file
    #[arg(long, default_value_t = true)]
    registry: bool,

    /// Schema ID prefix for remote resolution
    #[arg(long, default_value = "https://nintendoor64.ai/schemas/v1/")]
    id_prefix: String,

    /// Watch mode: regenerate schemas on file changes
    #[arg(short, long, default_value_t = false)]
    watch: bool,

    /// Types to exclude from generation (comma-separated)
    #[arg(long, value_delimiter = ',')]
    exclude: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = GeneratorConfig {
        output_dir: args.output,
        include_examples: args.examples,
        emit_registry: args.registry,
        schema_id_prefix: if args.id_prefix.is_empty() {
            None
        } else {
            Some(args.id_prefix)
        },
        exclude_types: args.exclude,
    };

    let mut generator = SchemaGenerator::with_config(config);

    #[cfg(feature = "full")]
    {
        // Register all Nintendoor64 core types for schema generation
        // Sonia AI Model contracts
        register_schemas!(
            generator,
            // Core artifact contracts
            sonia_core::ArtifactSpec,
            sonia_core::ArtifactType,
            sonia_core::EncodingMode,
            // Session management
            gamemodeai_session::SessionProfile,
            gamemodeai_session::CiFailure,
            gamemodeai_session::CiStatus,
            // Feature layout & knowledge graph
            sonia_featurelayout::FeatureLayout,
            sonia_featurelayout::FeatureEntry,
            gamemodeai_kg::SystemNode,
            // N64-specific types
            n64_layout::RomLayout,
            n64_layout::Segment,
            n64_layout::FileEntry,
            n64_layout::PatchSpec,
            n64_layout::PatchOperation,
            n64_layout::N64Constraints,
            n64_layout::BudgetReport,
            // Invariant and checklist types
            sonia_core::invariants::DeterminismViolation,
            sonia_core::invariants::BudgetViolation,
            sonia_core::ai_checklist::CheckResult,
        );
    }

    #[cfg(not(feature = "full"))]
    {
        eprintln!("Warning: 'full' feature not enabled. Registering minimal schema set.");
        // Minimal fallback: register a dummy type for testing
        #[derive(schemars::JsonSchema)]
        struct Dummy {
            #[schemars(description = "A test field")]
            test: String,
        }
        generator.register_schema::<Dummy>("Dummy");
    }

    if args.watch {
        eprintln!("Watch mode enabled. Monitoring source files for changes...");
        watch_and_generate(&mut generator)?;
    } else {
        generator.emit()?;
        eprintln!("Schema generation complete.");
    }

    Ok(())
}

#[cfg(feature = "full")]
fn watch_and_generate(generator: &mut SchemaGenerator) -> Result<(), Box<dyn std::error::Error>> {
    use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    // Watch the crates directory for Rust source changes
    let watch_path = PathBuf::from("crates");
    if watch_path.exists() {
        watcher.watch(&watch_path, RecursiveMode::Recursive)?;
        eprintln!("Watching: {}", watch_path.display());
    }

    // Initial generation
    generator.emit()?;

    // Event loop
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(event)) => {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    eprintln!("Detected source change, regenerating schemas...");
                    // Re-register to pick up any type changes
                    #[cfg(feature = "full")]
                    {
                        register_schemas!(
                            *generator,
                            sonia_core::ArtifactSpec,
                            gamemodeai_session::SessionProfile,
                            n64_layout::RomLayout,
                            n64_layout::PatchSpec,
                        );
                    }
                    generator.emit()?;
                }
            }
            Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
            Err(_) => {} // Timeout, continue loop
        }
    }
}

#[cfg(not(feature = "full"))]
fn watch_and_generate(generator: &mut SchemaGenerator) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Watch mode requires 'full' feature to be meaningful.");
    generator.emit()?;
    Ok(())
}

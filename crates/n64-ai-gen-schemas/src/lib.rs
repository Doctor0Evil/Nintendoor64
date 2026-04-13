// Nintendoor64/crates/n64-ai-gen-schemas/src/lib.rs
//! Schema generation library for Nintendoor64 AI-contract artifacts.
//!
//! This crate provides utilities to derive and emit JSON Schema definitions
//! from Rust types used throughout the Nintendoor64 Sonia AI Model.
//! Generated schemas serve as the canonical validation contracts for AI-generated
//! ArtifactSpec, SessionProfile, FeatureLayout, and platform-specific types.

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Core schema registry that tracks all generated schemas and their metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRegistry {
    /// Map of schema name to generated JSON Schema
    pub schemas: HashMap<String, schemars::schema::RootSchema>,
    /// Version metadata for change tracking
    pub version: String,
    /// Generation timestamp (ISO 8601)
    pub generated_at: String,
    /// Optional changelog of breaking changes
    pub changelog: Vec<SchemaChange>,
}

/// Represents a detected schema change between versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    pub schema_name: String,
    pub change_type: ChangeType,
    pub description: String,
    pub breaking: bool,
    pub migration_hint: Option<String>,
}

/// Classification of schema modifications.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    FieldAdded,
    FieldRemoved,
    TypeChanged,
    RequiredChanged,
    EnumVariantAdded,
    EnumVariantRemoved,
    DescriptionUpdated,
}

/// Generator configuration for fine-tuned schema emission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Output directory for generated .schema.json files
    pub output_dir: PathBuf,
    /// Whether to include example values in schemas
    pub include_examples: bool,
    /// Whether to emit a combined registry index
    pub emit_registry: bool,
    /// Prefix for schema $id fields (for remote resolution)
    pub schema_id_prefix: Option<String>,
    /// Types to explicitly exclude from generation
    pub exclude_types: Vec<String>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("schemas"),
            include_examples: true,
            emit_registry: true,
            schema_id_prefix: Some("https://nintendoor64.ai/schemas/v1/".to_string()),
            exclude_types: vec![],
        }
    }
}

/// Main schema generator entry point.
pub struct SchemaGenerator {
    config: GeneratorConfig,
    registry: SchemaRegistry,
}

impl SchemaGenerator {
    /// Create a new generator with default configuration.
    pub fn new() -> Self {
        Self {
            config: GeneratorConfig::default(),
            registry: SchemaRegistry {
                schemas: HashMap::new(),
                version: "0.1.0".to_string(),
                generated_at: chrono::Utc::now().to_rfc3339(),
                changelog: vec![],
            },
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(config: GeneratorConfig) -> Self {
        Self {
            config,
            registry: SchemaRegistry {
                schemas: HashMap::new(),
                version: "0.1.0".to_string(),
                generated_at: chrono::Utc::now().to_rfc3339(),
                changelog: vec![],
            },
        }
    }

    /// Register a type for schema generation by name and schema object.
    pub fn register_schema<T: JsonSchema + ?Sized>(&mut self, name: &str) {
        if self.config.exclude_types.contains(&name.to_string()) {
            return;
        }
        let schema = schema_for!(T);
        if let Some(prefix) = &self.config.schema_id_prefix {
            // Inject $id for remote schema resolution
            let mut schema_obj = schema.schema;
            schema_obj.metadata = schema_obj.metadata.or_default();
            schema_obj.metadata.as_mut().unwrap().id = Some(format!("{}{}", prefix, name));
            self.registry.schemas.insert(name.to_string(), schemars::schema::RootSchema {
                meta_schema: schema.meta_schema,
                schema: schema_obj,
                definitions: schema.definitions,
            });
        } else {
            self.registry.schemas.insert(name.to_string(), schema);
        }
    }

    /// Generate all registered schemas to disk.
    pub fn emit(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure output directory exists
        fs::create_dir_all(&self.config.output_dir)?;

        // Emit individual schema files
        for (name, schema) in &self.registry.schemas {
            let filename = format!("{}.schema.json", name);
            let path = self.config.output_dir.join(&filename);
            let content = serde_json::to_string_pretty(schema)?;
            fs::write(&path, content)?;
            eprintln!("Generated: {}", path.display());
        }

        // Emit registry index if enabled
        if self.config.emit_registry {
            let registry_path = self.config.output_dir.join("registry.index.json");
            let content = serde_json::to_string_pretty(&self.registry)?;
            fs::write(&registry_path, content)?;
            eprintln!("Generated registry index: {}", registry_path.display());
        }

        Ok(())
    }

    /// Compare current registry against a previous one to detect changes.
    pub fn detect_changes(&self, previous: &SchemaRegistry) -> Vec<SchemaChange> {
        let mut changes = Vec::new();
        
        for (name, current_schema) in &self.registry.schemas {
            if let Some(prev_schema) = previous.schemas.get(name) {
                // Simple structural diff: compare required fields and property keys
                let curr_props = current_schema.schema.object.as_ref()
                    .and_then(|o| o.properties.as_ref())
                    .map(|p| p.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                let prev_props = prev_schema.schema.object.as_ref()
                    .and_then(|o| o.properties.as_ref())
                    .map(|p| p.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                
                // Detect added fields
                for field in &curr_props {
                    if !prev_props.contains(field) {
                        changes.push(SchemaChange {
                            schema_name: name.clone(),
                            change_type: ChangeType::FieldAdded,
                            description: format!("Added field '{}'", field),
                            breaking: false,
                            migration_hint: None,
                        });
                    }
                }
                // Detect removed fields
                for field in &prev_props {
                    if !curr_props.contains(field) {
                        changes.push(SchemaChange {
                            schema_name: name.clone(),
                            change_type: ChangeType::FieldRemoved,
                            description: format!("Removed field '{}'", field),
                            breaking: true,
                            migration_hint: Some(format!("Field '{}' is no longer supported", field)),
                        });
                    }
                }
            } else {
                // New schema entirely
                changes.push(SchemaChange {
                    schema_name: name.clone(),
                    change_type: ChangeType::FieldAdded,
                    description: "New schema added".to_string(),
                    breaking: false,
                    migration_hint: None,
                });
            }
        }
        
        changes
    }
}

/// Helper macro to register multiple types at once.
#[macro_export]
macro_rules! register_schemas {
    ($gen:expr, $($t:ty),+ $(,)?) => {
        $(
            $gen.register_schema::<$t>(stringify!($t));
        )+
    };
}

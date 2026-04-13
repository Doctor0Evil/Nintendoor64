use std::fs;
use std::path::PathBuf;

use gamemodeai_rust_cli::contracts::{RunCargoParams, RunCargoResult};
use schemars::schema_for;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();

    let schemas_dir = root.join("schemas");
    fs::create_dir_all(&schemas_dir).expect("create schemas dir");

    let params_schema = schema_for!(RunCargoParams);
    let result_schema = schema_for!(RunCargoResult);

    fs::write(
        schemas_dir.join("gamemodeai.rust.run-cargo-params.schema.json"),
        serde_json::to_vec_pretty(&params_schema).unwrap(),
    )
    .unwrap();

    fs::write(
        schemas_dir.join("gamemodeai.rust.run-cargo-result.schema.json"),
        serde_json::to_vec_pretty(&result_schema).unwrap(),
    )
    .unwrap();
}

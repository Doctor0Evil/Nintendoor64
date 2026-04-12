fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = format!("{}/include", crate_dir);
    std::fs::create_dir_all(&out_dir).unwrap();

    let config_path = format!("{}/cbindgen.toml", crate_dir);
    let config = cbindgen::Config::from_file(&config_path).unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(format!("{}/gamemodeai.h", out_dir));
}

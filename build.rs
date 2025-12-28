use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn generate_module_constants(config_path: &str, dest_path: &Path, constants_types: HashMap<&str, &str>) {
    let config_content = fs::read_to_string(config_path)
        .expect("Failed to read extractor.config.toml");
    
    let mut constants = String::new();

    for line in config_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // simple parsing: KEY = "VALUE" or KEY = true
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            // wrap in quotes se stringa
            let value_code = if value.starts_with('"') {
                value.to_string()
            } else {
                value.to_string()
            };
            let value_type = constants_types.get(key).unwrap_or(&"usize");

            constants.push_str(&format!(
                "pub const {}: {} = {};\n",
                key, value_type, value_code
            ));
        }
    }

    fs::write(&dest_path, constants).expect("Failed to write build_constants.rs");
    println!("cargo:rerun-if-changed={}", config_path);
}

fn main() {
    let extractor_constant_types_map = HashMap::from([
        ("DEFAULT_INSERT_BATCH_SIZE", "usize"),
        ("DEFAULT_INSERT_BATCH_TIMEOUT_MS", "usize"),
        ("DEFAULT_FLUSH_INTERVAL_MS", "u64"),
        ("DEFAULT_PDFIUM_LIB_PATH", "&'static str"),
        ("DEFAULT_MAX_TOKENS", "usize"),
        ("DEFAULT_TOKENS_MIN_LENGTH", "usize"),
    ]);

    let out_dir = env::var("OUT_DIR").unwrap();

    let extractor_config_path = "build/extractor.config.toml";
    let extractor_output_path = Path::new(&out_dir).join("extractor_constants.rs");

    generate_module_constants(extractor_config_path, &extractor_output_path, extractor_constant_types_map);

    let storage_constant_types_map = HashMap::from([
        ("DEFAULT_STORAGE_WORKER_RECEIVE_TIMEOUT_MS", "u64"),
        ("DEFAULT_STORAGE_DB_JOURNAL_MODE", "&'static str"),
        ("DEFAULT_STORAGE_DB_CACHE_SIZE", "&'static str"),
        ("DEFAULT_STORAGE_DB_TEMP_STORE", "&'static str"),
        ("DEFAULT_STORAGE_DB_LOCKING_MODE", "&'static str"),
        ("DEFAULT_STORAGE_DB_PATH", "&'static str"),
    ]);
    let storage_config_path = "build/storage.config.toml";
    let storage_output_path = Path::new(&out_dir).join("storage_constants.rs");

    generate_module_constants(storage_config_path, &storage_output_path, storage_constant_types_map);
}

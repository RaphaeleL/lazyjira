use std::{fs, path::Path};

pub fn save_cache(path: &str, data: &serde_json::Value) {
    fs::create_dir_all(Path::new(path).parent().unwrap()).ok();
    fs::write(path, data.to_string()).unwrap();
}

#[allow(dead_code)]
pub fn load_cache(path: &str) -> Option<serde_json::Value> {
    fs::read_to_string(path).ok().and_then(|s| {
        serde_json::from_str(&s).ok()
    })
}

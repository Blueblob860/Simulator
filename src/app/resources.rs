use std::{fs, path::Path};

pub fn load_string(file_name: &str) -> anyhow::Result<String> {
    let path = Path::new("./").join(file_name);
    Ok(fs::read_to_string(path)?)
}

pub fn load_bytes(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = Path::new("./").join(file_name);
    Ok(fs::read(path)?)
}
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ConvertTask {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub source_size: u64,
    // directory to write the convert vb script
    pub script: PathBuf,
}

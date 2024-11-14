use super::utils::{File, Language};
use serde::Serialize;
use std::{
    fs::{self, remove_file, OpenOptions},
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Default)]
pub struct CombineConfig {
    id: usize,
    language: Language,
    pub destination: PathBuf,
    workspace: PathBuf,
    cover: PathBuf,
    pub files: Vec<File>,
}

impl CombineConfig {
    pub fn new(id: usize) -> Self {
        CombineConfig {
            id,
            ..Default::default()
        }
    }
    pub fn set_language(&mut self, lang: Language) -> &mut Self {
        self.language = lang;
        self
    }
    pub fn set_destination(&mut self, destination: &Path) -> &mut Self {
        self.destination = destination.into();
        self
    }
    pub fn set_workspace(&mut self, workspace: &Path) -> &mut Self {
        self.workspace = workspace.join(format!(r"combine\\{}", self.id)).into();
        self
    }
    pub fn workspace(&self) -> PathBuf {
        self.workspace.clone()
    }
    pub fn set_cover(&mut self, cover: &Path) -> &mut Self {
        if cover.exists() {
            self.cover = cover.into();
        }
        self
    }
    pub fn set_files(&mut self, files: &[File]) -> &mut Self {
        self.files = files.to_vec();
        self
    }
    pub fn write_config(&self, filepath: &Path) -> anyhow::Result<PathBuf> {
        if !filepath.exists() {
            fs::create_dir_all(&filepath)?;
        }
        let config_path = filepath.join("config.json");
        if config_path.exists() {
            remove_file(&config_path)?;
        }
        let writer = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&config_path)?;
        serde_json::to_writer(writer, self)?;
        Ok(config_path)
    }
}

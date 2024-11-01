use std::{
    fs::{remove_file, OpenOptions},
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::utils::{File, Language};

#[derive(Debug, Serialize, Default)]
pub struct CombineConfig {
    language: Language,
    pub destination: PathBuf,
    workspace: PathBuf,
    cover: PathBuf,
    pub files: Vec<File>,
}

impl CombineConfig {
    pub fn new() -> Self {
        CombineConfig::default()
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
        if workspace.exists() {
            self.workspace = workspace.into();
        }
        self
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
        let config_path = if filepath.is_dir() {
            filepath.join("config.json")
        } else {
            filepath.to_path_buf()
        };
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

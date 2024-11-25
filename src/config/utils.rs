use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const WORKER_NUMBER_ENV: &str = "MK_WORD_WORKER";
const COMBINE_BIN: &str = "MK_COMBINE_BIN";
const APP_ROOT: &str = "MK_FUSION";

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub enum Language {
    #[default]
    CN,
    EN,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub enum FusionMode {
    #[default]
    PDF,
    RTF,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    pub filename: String,
    pub title: String,
    pub path: PathBuf,
    pub size: u64,
}

pub fn worker_number() -> usize {
    let default_workers = 5;
    match env::var(WORKER_NUMBER_ENV) {
        Ok(worker) => match worker.parse::<usize>() {
            Ok(n) => n,
            Err(_) => default_workers,
        },
        Err(_) => default_workers,
    }
}

pub fn combiner_bin() -> Option<PathBuf> {
    match env::var(COMBINE_BIN) {
        Ok(bin) => Some(Path::new(&bin).into()),
        Err(_) => None,
    }
}

pub fn workspace(id: Option<String>) -> anyhow::Result<PathBuf> {
    let mut is_temp = false;
    let root = fusion_app_root()?.join("workspace");
    if !root.exists() {
        fs::create_dir_all(&root)?;
    }
    let id = match id {
        Some(id) => id,
        None => {
            is_temp = true;
            "__temp__".into()
        }
    };
    let workspace = root.join(&id);
    if is_temp && workspace.exists() {
        fs::remove_dir_all(&workspace)?;
    }
    if !workspace.exists() {
        fs::create_dir_all(&workspace)?;
    }
    Ok(workspace)
}

pub fn fusion_app_root() -> anyhow::Result<PathBuf> {
    let root = env::var(APP_ROOT)?;
    Ok(Path::new(&root).into())
}

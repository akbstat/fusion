use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde::{Deserialize, Serialize};

use crate::config::convert::ConvertTask;
const SOURCE_FILE: &str = "source.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceRecord {
    file: PathBuf,
    modified_at: u64,
}

/// to find out or record source file modified time
#[derive(Debug, Clone)]
pub struct Source {
    filepath: PathBuf,
    data: HashMap<PathBuf, u64>,
}

impl Source {
    pub fn new(workspace: &Path) -> anyhow::Result<Self> {
        let mut data = HashMap::new();
        let filepath = workspace.join(SOURCE_FILE);
        if let Ok(bytes) = fs::read(&filepath) {
            let records = serde_json::from_slice::<Vec<SourceRecord>>(&bytes)?;
            records.into_iter().for_each(|source| {
                data.insert(source.file, source.modified_at);
            });
        };
        Ok(Source { data, filepath })
    }

    /// if return true stands for updated, else stands for not change
    fn is_updated(&self, source: &Path, destination: &Path) -> bool {
        if source.exists() && source.is_file() {
            match self.data.get(&source.to_path_buf()) {
                Some(last_modified) => {
                    let new_modified = fs::metadata(source).unwrap().modified().unwrap();
                    let is_source_updated = new_modified
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        .gt(last_modified);
                    if is_source_updated || !destination.exists() {
                        true
                    } else {
                        false
                    }
                }
                None => true,
            }
        } else {
            true
        }
    }

    pub fn filter_convert_tasks(&self, tasks: &[ConvertTask]) -> Vec<ConvertTask> {
        let mut filtered = Vec::with_capacity(tasks.len());
        for task in tasks {
            if self.is_updated(&task.source, &task.destination) {
                filtered.push(task.clone());
            }
        }
        filtered
    }

    pub fn update_source(&self, source: &Path) -> anyhow::Result<()> {
        let mut data = vec![];
        if source.is_dir() {
            for entry in fs::read_dir(source)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let filename = entry.file_name();
                    let filename = filename.to_string_lossy().to_string();
                    if filename.ends_with(".rtf") {
                        let modified = entry.metadata()?.modified()?;
                        let modified_at = modified.duration_since(UNIX_EPOCH)?.as_secs();
                        data.push(SourceRecord {
                            file: source.join(filename),
                            modified_at,
                        });
                    }
                }
            }
        }
        let f = fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(&self.filepath)?;
        serde_json::to_writer(f, &data)?;

        Ok(())
    }
}

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    combiner::config::CombineConfig,
    converter::task::ConvertTask,
    utils::{File, FusionMode, Language},
};

#[derive(Debug, Deserialize, Clone)]
pub struct FusionParam {
    pub id: Option<String>,
    pub tasks: Vec<FusionTask>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FusionTask {
    pub name: String,
    pub language: Language,
    pub cover: Option<PathBuf>,
    pub destination: PathBuf,
    pub mode: FusionMode,
    pub files: Vec<File>,
}

impl FusionParam {
    pub fn to_convert_task(&self, workspace: &Path) -> anyhow::Result<Vec<ConvertTask>> {
        let mut file_set = HashSet::<String>::new();
        let mut tasks = vec![];
        self.tasks.iter().for_each(|task| {
            if task.mode.eq(&FusionMode::PDF) {
                task.files.iter().for_each(|f| {
                    if !file_set.contains(&f.filename) {
                        tasks.push(ConvertTask {
                            source: f.path.clone(),
                            destination: converted_pdf_dir(&workspace)
                                .join(f.filename.replace(".rtf", ".pdf")),
                            source_size: f.size,
                            script: convert_script_dir(&workspace),
                        });
                        file_set.insert(f.filename.to_owned());
                    }
                });
            }
        });
        Ok(tasks)
    }

    pub fn convert_task_numer(&self) -> usize {
        let mut file_set = HashSet::<String>::new();
        self.tasks.iter().for_each(|task| {
            task.files.iter().for_each(|f| {
                file_set.insert(f.filename.clone());
            });
        });
        file_set.len()
    }

    pub fn to_combine_config(
        &self,
        workspace: &Path,
    ) -> anyhow::Result<(Vec<CombineConfig>, Vec<CombineConfig>)> {
        let mut pdf_configs = vec![];
        let mut rtf_configs = vec![];
        self.tasks.iter().for_each(|task| match task.mode {
            FusionMode::PDF => pdf_configs.push(pdf_combine_task(task, workspace)),
            FusionMode::RTF => rtf_configs.push(rtf_combine_task(task, workspace)),
        });
        Ok((pdf_configs, rtf_configs))
    }

    pub fn combine_task_number(&self) -> usize {
        self.tasks.len()
    }
}

fn converted_pdf_dir(workspace: &Path) -> PathBuf {
    let dir = workspace.join("converted");
    if !dir.exists() {
        fs::create_dir_all(&dir).ok();
    }
    dir
}

fn convert_script_dir(workspace: &Path) -> PathBuf {
    let dir = workspace.join("scripts");
    if !dir.exists() {
        fs::create_dir_all(&dir).ok();
    }
    dir
}

fn pdf_combine_task(task: &FusionTask, workspace: &Path) -> CombineConfig {
    let mut files = vec![];
    let mut config = CombineConfig::new();
    config.set_language(task.language.clone());
    config.set_destination(&task.destination.clone().join(format!("{}.pdf", task.name)));
    config.set_workspace(&workspace);
    if let Some(cover) = task.cover.clone() {
        config.set_cover(&cover);
    }
    task.files.iter().for_each(|f| {
        files.push(File {
            filename: f.filename.to_owned(),
            title: f.title.to_owned(),
            path: converted_pdf_dir(&workspace).join(f.filename.replace(".rtf", ".pdf")),
            size: 0,
        });
    });
    config.files = files;
    config
}

fn rtf_combine_task(task: &FusionTask, workspace: &Path) -> CombineConfig {
    let mut files = vec![];
    let mut config = CombineConfig::new();
    config.set_language(task.language.clone());
    config.set_destination(&task.destination.clone().join(format!("{}.rtf", task.name)));
    config.set_workspace(&workspace);
    task.files.iter().for_each(|f| {
        files.push(File {
            filename: f.filename.to_owned(),
            title: f.title.to_owned(),
            path: f.path.clone(),
            size: 0,
        });
    });
    config.files = files;
    config
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_to_convert_task() -> anyhow::Result<()> {
        let workspace = Path::new(r"D:\Users\yuqi01.chen\.temp\app\mobiuskit\fusion");
        let param = init();
        let tasks = param.to_convert_task(workspace)?;
        assert_eq!(tasks.len(), 5);
        Ok(())
    }
    #[test]
    fn to_combine_config() -> anyhow::Result<()> {
        let workspace = Path::new(r"D:\Users\yuqi01.chen\.temp\app\mobiuskit\fusion");
        let param = init();
        let (pdf_config, rtf_config) = param.to_combine_config(workspace)?;
        assert_eq!(pdf_config.len(), 2);
        assert_eq!(pdf_config[0].files.len(), 3);
        assert_eq!(pdf_config[1].files.len(), 2);
        assert_eq!(rtf_config.len(), 2);
        assert_eq!(rtf_config[0].files.len(), 3);
        assert_eq!(rtf_config[1].files.len(), 2);
        Ok(())
    }
    fn init() -> FusionParam {
        FusionParam {
            id: None,
            tasks: vec![FusionTask {
                name: "listing 1".into(),
                language: crate::utils::Language::CN,
                cover: None,
                destination: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined")
                    .into(),
                mode: crate::utils::FusionMode::PDF,
                files: vec![
                    File {
                        filename: "l-16-02-04-04-mh-fas.rtf".into(),
                        title: "列表 16.2.4.4: 既往病史 - 全分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-04-04-mh-fas.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "l-16-02-04-05-pre-ex-fas.rtf".into(),
                        title: "列表 16.2.4.5: 既往用药 - 全分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-04-05-pre-ex-fas.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "l-16-02-05-01-ex-sum-ss.rtf".into(),
                        title: "列表 16.2.5.1: 研究药物暴露 - 安全性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-05-01-ex-sum-ss.rtf").into(),
                        size: 0,
                    },
                ],
            },FusionTask {
                name: "listing 2".into(),
                language: crate::utils::Language::CN,
                cover: None,
                destination: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined")
                    .into(),
                mode: crate::utils::FusionMode::PDF,
                files: vec![
                    File {
                        filename: "l-16-02-08-01-04-lb-thyrabn-ss.rtf".into(),
                        title: "列表 16.2.8.1.4: 异常实验室检查 (甲状腺功能) - 安全性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-08-01-04-lb-thyrabn-ss.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "l-16-02-08-06-ecog-ss.rtf".into(),
                        title: "列表 16.2.8.6: ECOG评分 - 安全性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-08-06-ecog-ss.rtf").into(),
                        size: 0,
                    },
                ],
            }],
        }
    }
}

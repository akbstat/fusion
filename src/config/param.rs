use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{
    combine::{CombinePDFParam, PDFFile, RTFCombineParam},
    convert::ConvertTask,
    utils::{File, FusionMode, Language},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FusionParam {
    pub id: Option<String>,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub top: PathBuf,
    pub tasks: Vec<FusionTask>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FusionTask {
    pub name: String,
    pub language: Language,
    pub cover: Option<PathBuf>,
    pub destination: PathBuf,
    pub mode: FusionMode,
    pub files: Vec<File>,
    pub toc_headers: (String, String, String, String),
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
        self.tasks
            .iter()
            .filter(|task| task.mode.ne(&FusionMode::RTF))
            .for_each(|task| {
                task.files.iter().for_each(|f| {
                    file_set.insert(f.filename.clone());
                });
            });
        file_set.len()
    }

    pub fn to_combine_param(
        &self,
        workspace: &Path,
    ) -> anyhow::Result<(Vec<CombinePDFParam>, Vec<RTFCombineParam>)> {
        let mut pdf_configs = vec![];
        let mut rtf_configs = vec![];
        self.tasks.iter().for_each(|task| match task.mode {
            FusionMode::PDF => {
                if let Ok(param) = pdf_combine_task(pdf_configs.len(), task, workspace) {
                    pdf_configs.push(param)
                }
            }
            FusionMode::RTF => rtf_configs.push(rtf_combine_task(task)),
        });
        Ok((pdf_configs, rtf_configs))
    }

    pub fn combine_task_number(&self) -> usize {
        self.tasks.len()
    }

    pub fn fix(&mut self) -> anyhow::Result<()> {
        for (index, task) in self.tasks.clone().into_iter().enumerate() {
            let cover = if let Some(cover) = task.cover {
                if cover.exists() {
                    Some(cover)
                } else {
                    None
                }
            } else {
                None
            };
            let mut files = Vec::with_capacity(task.files.len());
            for file in task.files {
                if file.path.exists() {
                    let size = fs::metadata(&file.path)?.len();
                    files.push(File {
                        filename: file.filename.clone(),
                        title: file.title.clone(),
                        path: file.path.clone(),
                        size: size.into(),
                    });
                }
            }

            (*self.tasks.get_mut(index).unwrap()).files = files;
            (*self.tasks.get_mut(index).unwrap()).cover = cover;
        }
        Ok(())
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

fn pdf_combine_task(
    id: usize,
    task: &FusionTask,
    workspace: &Path,
) -> anyhow::Result<CombinePDFParam> {
    let combine_workspace = workspace.join("combine").join(id.to_string());
    let toc = &combine_workspace.join("toc.pdf");
    let mut files = Vec::with_capacity(task.files.len());
    task.files.iter().enumerate().for_each(|(id, file)| {
        files.push(PDFFile {
            id,
            title: file.title.clone(),
            filepath: workspace
                .join("converted")
                .join(file.filename.replace(".rtf", ".pdf")),
            ..Default::default()
        });
    });
    let param = CombinePDFParam::new(
        &combine_workspace,
        &task.language,
        &task.cover,
        toc,
        &files,
        &task.destination.join(format!("{}.pdf", &task.name)),
        &task.toc_headers,
    )?;
    Ok(param)
}

fn rtf_combine_task(task: &FusionTask) -> RTFCombineParam {
    let mut files = vec![];

    task.files.iter().for_each(|f| {
        files.push(f.path.clone());
    });

    RTFCombineParam {
        destination: task.destination.join(format!("{}.rtf", task.name)),
        files,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use sha2::{Digest, Sha256};

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
        let (pdf_config, rtf_config) = param.to_combine_param(workspace)?;
        assert_eq!(pdf_config.len(), 2);
        assert_eq!(pdf_config[0].files.len(), 3);
        assert_eq!(pdf_config[1].files.len(), 2);
        assert_eq!(rtf_config.len(), 2);
        assert_eq!(rtf_config[0].files.len(), 3);
        assert_eq!(rtf_config[1].files.len(), 2);
        Ok(())
    }

    #[test]
    fn param_fix_test() -> anyhow::Result<()> {
        let mut param = init();
        assert_eq!(param.tasks.get(0).unwrap().files.len(), 3);
        let task: &mut FusionTask = param.tasks.get_mut(0).unwrap();
        (*task.files.get_mut(0).unwrap()).path = Path::new(
            r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-04-04-mh-xxx.rtf",
        )
        .into();
        param.fix()?;
        assert_eq!(param.tasks.get(0).unwrap().files.len(), 2);
        Ok(())
    }

    fn init() -> FusionParam {
        FusionParam {
            id: None,
            source: Path::new(r"D:\Studies\ak112\303\stats\CSR\product\output").into(),
            destination: Path::new(r"D:\Studies\ak112\303\stats\CSR\product\output\combined").into(),
            top: Path::new(r"D:\Studies\ak112\303\stats\CSR\utility\top-ak112-303-CSR.xlsx").into(),
            tasks: vec![FusionTask {
                name: "listing 1".into(),
                language: Language::CN,
                cover: None,
                destination: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined")
                    .into(),
                mode: FusionMode::PDF,
                files: vec![
                    File {
                        filename: "l-16-02-04-04-mh-fas.rtf".into(),
                        title: "列表 16.2.4.4: 既往病史 - 全分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/l-16-02-04-04-mh-fas.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "l-16-02-04-05-pre-ex-fas.rtf".into(),
                        title: "列表 16.2.4.5: 既往用药 - 全分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/l-16-02-04-05-pre-ex-fas.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "l-16-02-05-01-ex-sum-ss.rtf".into(),
                        title: "列表 16.2.5.1: 研究药物暴露 - 安全性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/l-16-02-05-01-ex-sum-ss.rtf").into(),
                        size: 0,
                    },
                ],
                toc_headers: ("".into(), "".into(), "".into(), "".into()),
            },FusionTask {
                name: "listing 2".into(),
                language: Language::CN,
                cover: None,
                destination: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined")
                    .into(),
                mode: FusionMode::PDF,
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
                toc_headers: ("".into(), "".into(), "".into(), "".into()),
            }],
        }
    }

    #[test]
    fn task_id_test() -> anyhow::Result<()> {
        let data = r"D:/Studies/ak112/303/stats/CSR/product/output/combined";
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let result = format!("{:x}", result);
        println!("{}", result);
        Ok(())
    }
}

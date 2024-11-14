use super::param::FusionParam;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, read, remove_file, write, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct SaveConfigParam {
    pub id: Option<String>,
    pub name: String,
}

#[derive(Debug)]
pub struct ConfigManager {
    config_dir: PathBuf,
    configs: Vec<Config>,
    repo: PathBuf,
}

impl ConfigManager {
    pub fn new(root: &Path) -> Self {
        let repo: PathBuf = root.join("config_repo.json");
        let config_dir = root.join("config");
        if !config_dir.exists() {
            create_dir_all(&config_dir).unwrap();
        }
        let mut manager = ConfigManager {
            repo,
            config_dir,
            configs: vec![],
        };
        manager.update();
        manager
    }

    pub fn list_configs(&self) -> Vec<Config> {
        self.configs.clone()
    }

    pub fn find_config(&self, id: &str) -> anyhow::Result<Option<FusionParam>> {
        match self.find_config_index(id) {
            Some(index) => match self.configs.get(index) {
                Some(config) => {
                    let bytes = read(&config.path)?;
                    let mut param = serde_json::from_slice::<FusionParam>(&bytes)?;
                    param.fix()?;
                    Ok(Some(param))
                }
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub fn save_config(
        &mut self,
        config: &SaveConfigParam,
        param: &FusionParam,
    ) -> anyhow::Result<String> {
        let id = match &config.id {
            Some(id) => id.clone(),
            None => nanoid!(10),
        };

        let config_file = self.config_dir.join(format!("{}.json", &id));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&config_file)?;

        // find if config existed
        match self.find_config_index(&id) {
            Some(index) => {
                let c = self.configs.get_mut(index).unwrap();
                c.name = config.name.clone();
            }
            None => {
                self.configs.push(Config {
                    id: id.clone(),
                    name: config.name.clone(),
                    path: config_file,
                });
            }
        }
        let mut param = param.to_owned();
        param.id = Some(id.clone());
        param.fix()?;
        file.write_all(&serde_json::to_vec(&param)?)?;
        write(&self.repo, &serde_json::to_vec(&self.configs)?)?;
        self.update();
        Ok(id)
    }

    pub fn remove_config(&mut self, id: &str) -> anyhow::Result<()> {
        if let Some(index) = self.find_config_index(id) {
            remove_file(self.config_dir.join(format!("{}.json", id)))?;
            self.configs.swap_remove(index);
            write(&self.repo, &serde_json::to_vec(&self.configs)?)?;
            self.update();
        }
        Ok(())
    }

    fn find_config_index(&self, id: &str) -> Option<usize> {
        match self
            .configs
            .iter()
            .enumerate()
            .filter(|(_, c)| c.id.eq(&id))
            .map(|(index, _)| index)
            .collect::<Vec<usize>>()
            .first()
        {
            Some(index) => Some(*index),
            None => None,
        }
    }

    fn update(&mut self) {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.repo)
            .expect("Failed to read config repo");
        let mut bytes = vec![];
        self.configs = match file.read_to_end(&mut bytes) {
            Ok(_) => match serde_json::from_slice::<Vec<Config>>(&bytes) {
                Ok(configs) => configs,
                Err(_) => vec![],
            },
            Err(_) => vec![],
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{
        param::FusionTask,
        utils::{File, FusionMode, Language},
    };

    use super::*;
    #[test]
    fn config_test() {
        let root = Path::new(r"D:\Users\yuqi01.chen\.temp\app\mobiuskit\fusion");
        let mut config = ConfigManager::new(root);
        config
            .save_config(
                &&SaveConfigParam {
                    id: None,
                    name: "csr".into(),
                },
                &param(),
            )
            .unwrap();
        assert_eq!(config.configs.len(), 1);

        // create another config
        config
            .save_config(
                &&SaveConfigParam {
                    id: None,
                    name: "adhoc".into(),
                },
                &param(),
            )
            .unwrap();
        assert_eq!(config.configs.len(), 2);

        // get one config id
        let id = config.list_configs().first().unwrap().id.clone();

        // update one existed config
        config
            .save_config(
                &SaveConfigParam {
                    id: Some(id.clone()),
                    name: "demo".into(),
                },
                &param(),
            )
            .unwrap();
        assert_eq!(config.configs.len(), 2);

        // find param
        assert!(config.find_config(&id).unwrap().is_some());

        // remove config
        config.remove_config(&id).unwrap();
        assert_eq!(config.configs.len(), 1);
    }

    fn param() -> FusionParam {
        FusionParam {
            id: None,
            source: Path::new(r"D:\Studies\ak112\303\stats\CSR\product\output").into(),
            destination: Path::new(r"D:\Studies\ak112\303\stats\CSR\product\output\combined").into(),
            top: Path::new(r"D:\Studies\ak112\303\stats\CSR\utility\top-ak112-303-CSR.xlsx").into(),
            tasks: vec![FusionTask {
                name: "all_listings".into(),
                language: Language::CN,
                cover: None,
                destination: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined")
                    .into(),
                mode: FusionMode::PDF,
                files: vec![
                    File {
                        filename: "l-16-02-07-04-teae-wt-ss.rtf".into(),
                        title: "列表 16.2.7.4: 导致依沃西单抗/帕博利珠单抗永久停用的TEAE - 安全性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-07-04-teae-wt-ss.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "t-14-02-08-03-02-eq-index-fas.rtf".into(),
                        title: "表 14.2.8.3.2: EuroQol EQ-5D-5L问卷结果总结2 - 效应指数值和健康状态评分 - EuroQol EQ-5D-5L分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-02-08-03-02-eq-index-fas.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "t-14-03-03-01-05-irsae-ss.rtf".into(),
                        title: "表 14.3.3.1.5: 严重的irAE按照irAE分组、PT和CTCAE分级总结 - 安全性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-03-03-01-05-irsae-ss.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "t-14-03-04-05-01-eg-intp-ss.rtf".into(),
                        title: "表 14.3.4.5.1: ECG 整体评估 - 安全性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-03-04-05-01-eg-intp-ss.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "t-14-03-04-10-is-ada-sub-ims.rtf".into(),
                        title: "表 14.3.4.10: ADA检测结果与疗效相关的亚组分析 - 免疫原性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-03-04-10-is-ada-sub-ims.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "l-16-02-08-01-04-lb-thyrabn-ss.rtf".into(),
                        title: "Large size output 0".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-08-01-04-lb-thyrabn-ss.rtf").into(),
                        size: 0,
                    },
                ],
            }, FusionTask {
                name: "all_listings".into(),
                language: Language::CN,
                cover: None,
                destination: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined")
                    .into(),
                mode: FusionMode::RTF,
                files: vec![
                    File {
                        filename: "l-16-02-07-04-teae-wt-ss.rtf".into(),
                        title: "列表 16.2.7.4: 导致依沃西单抗/帕博利珠单抗永久停用的TEAE - 安全性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-07-04-teae-wt-ss.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "t-14-02-08-03-02-eq-index-fas.rtf".into(),
                        title: "表 14.2.8.3.2: EuroQol EQ-5D-5L问卷结果总结2 - 效应指数值和健康状态评分 - EuroQol EQ-5D-5L分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-02-08-03-02-eq-index-fas.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "t-14-03-03-01-05-irsae-ss.rtf".into(),
                        title: "表 14.3.3.1.5: 严重的irAE按照irAE分组、PT和CTCAE分级总结 - 安全性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-03-03-01-05-irsae-ss.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "t-14-03-04-05-01-eg-intp-ss.rtf".into(),
                        title: "表 14.3.4.5.1: ECG 整体评估 - 安全性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-03-04-05-01-eg-intp-ss.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "t-14-03-04-10-is-ada-sub-ims.rtf".into(),
                        title: "表 14.3.4.10: ADA检测结果与疗效相关的亚组分析 - 免疫原性分析集".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-03-04-10-is-ada-sub-ims.rtf").into(),
                        size: 0,
                    },
                    File {
                        filename: "l-16-02-08-01-04-lb-thyrabn-ss.rtf".into(),
                        title: "Large size output 0".into(),
                        path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-08-01-04-lb-thyrabn-ss.rtf").into(),
                        size: 0,
                    },
                ],
            }],
        }
    }
}

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    thread,
};

pub struct PDFCombiner {
    bin: PathBuf,
    is_finished: Arc<Mutex<bool>>,
}

impl PDFCombiner {
    pub fn new(bin: &Path) -> Self {
        PDFCombiner {
            bin: bin.into(),
            is_finished: Arc::new(Mutex::new(false)),
        }
    }
    pub fn run(&self, config: &Path) {
        let is_finished = self.is_finished.clone();
        let bin = self.bin.clone();
        let config = config.to_path_buf();
        thread::spawn(move || {
            Command::new("cmd")
                .arg("/C")
                .arg(bin)
                .arg(config)
                .output()
                .unwrap();
            *(is_finished.lock().unwrap()) = true;
        });
    }
    pub fn is_finished(&self) -> bool {
        if let Ok(is_finished) = self.is_finished.lock() {
            *is_finished
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        combiner::config::CombineConfig,
        utils::{File, Language},
    };
    #[test]
    fn combiner_test() -> anyhow::Result<()> {
        let bin = Path::new(r"D:\projects\py\combiner\dist\combiner.exe");
        let files = vec![
            File {
                filename: "".into(),
                title: "列表 16.2.4.4: 既往病史 - 全分析集".into(),
                path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-04-04-mh-fas.pdf").into(),
                size: 1,
            },
            File {
                filename: "".into(),
                title: "列表 16.2.4.5: 既往用药 - 全分析集".into(),
                path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-04-05-pre-ex-fas.pdf").into(),
                size: 2,
            },
            File {
                filename: "".into(),
                title: "列表 16.2.5.1: 研究药物暴露 - 安全性分析集".into(),
                path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-05-01-ex-sum-ss.pdf").into(),
                size: 3,
            },
            File {
                filename: "".into(),
                title: "列表 16.2.8.6: ECOG评分 - 安全性分析集".into(),
                path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-08-06-ecog-ss.pdf").into(),
                size: 4,
            },
            File {
                filename: "".into(),
                title: "列表 16.2.8.1.4: 异常实验室检查 (甲状腺功能) - 安全性分析集".into(),
                path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-08-01-04-lb-thyrabn-ss.pdf").into(),
                size: 5,
            },
        ];
        let cfg_path = Path::new(r"D:\Studies\ak112\303\stats\CSR\product\output\combined");
        let mut cfg = CombineConfig::new();
        cfg.set_destination(Path::new(
            r"D:\Studies\ak112\303\stats\CSR\product\output\combined\final.pdf",
        ))
        .set_language(Language::CN)
        .set_workspace(Path::new(
            r"D:\Studies\ak112\303\stats\CSR\product\output\combined\workspace",
        ))
        .set_files(&files);
        let cfg_path = cfg.write_config(cfg_path)?;
        let c = PDFCombiner::new(bin);
        c.run(&cfg_path);
        loop {
            if c.is_finished() {
                break;
            }
        }
        Ok(())
    }
}

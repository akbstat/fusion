use crate::{
    combiner::pdf::controller::PDFCombineController,
    converter::controller::ConvertController,
    fusion::param::FusionParam,
    utils::{combiner_bin, worker_number},
};
use std::{
    path::{Path, PathBuf},
    sync::{mpsc::Sender, Arc, Mutex},
};

pub struct FusionController {
    param: FusionParam,
    workspace: PathBuf,
}

impl FusionController {
    pub fn new(param: &FusionParam, workspace: &Path) -> anyhow::Result<Self> {
        Ok(FusionController {
            param: param.to_owned(),
            workspace: workspace.into(),
        })
    }

    /// convert rtf to pdf
    pub fn convert(
        &self,
        status: Arc<Mutex<Sender<()>>>,
        logger: Arc<Mutex<Sender<String>>>,
    ) -> anyhow::Result<()> {
        let workers = worker_number();
        let converter = ConvertController::new(workers, status, logger);
        let tasks = self.param.to_convert_task(&self.workspace)?;
        converter.execute(&tasks);
        Ok(())
    }

    /// combine outputs
    pub fn combine(
        &self,
        status: Arc<Mutex<Sender<()>>>,
        logger: Arc<Mutex<Sender<String>>>,
    ) -> anyhow::Result<()> {
        let combiner_bin = combiner_bin().expect("Error: invalid binary combiner executor");
        let workers = worker_number();
        let controller = PDFCombineController::new(workers, status, logger, &combiner_bin);
        let configs = self
            .param
            .to_combine_config(&self.workspace)?
            .into_iter()
            .enumerate()
            .map(|(id, config)| {
                let name = config
                    .destination
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                let config = config
                    .write_config(&self.workspace.join(format!("combine_config_{}.json", id)))
                    .unwrap();
                (name, config)
            })
            .collect::<Vec<(String, PathBuf)>>();
        controller.combine(&configs);
        Ok(())
    }
}

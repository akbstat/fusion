use crate::{
    combiner::pdf_combine::PDFCombiner,
    converter::controller::ConvertController,
    fusion::param::FusionParam,
    utils::{combiner_bin, convert_worker_number},
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
        let workers = convert_worker_number();
        let converter = ConvertController::new(workers, status, logger);
        let tasks = self.param.to_convert_task(&self.workspace)?;
        converter.execute(&tasks);
        Ok(())
    }

    /// combine outputs
    pub fn combine(&self) -> anyhow::Result<()> {
        let combiner_bin = combiner_bin().expect("Error: invalid binary combiner executor");
        let pdf_combiner = PDFCombiner::new(&combiner_bin);
        let configs = self.param.to_combine_config(&self.workspace)?;
        for (index, config) in configs.iter().enumerate() {
            let filepath = &self
                .workspace
                .join(format!("combine_config_{}.json", index));
            if let Ok(config_path) = config.write_config(filepath) {
                pdf_combiner.run(&config_path);
            }
        }
        Ok(())
    }
}

use crate::{
    combiner::{pdf::controller::PDFCombineController, rtf::controller::RTFCombineController},
    config::{
        convert::ConvertTask,
        param::FusionParam,
        utils::{combiner_bin, worker_number},
    },
    converter::controller::ConvertController,
};
use std::{
    fs,
    path::PathBuf,
    sync::{mpsc::Sender, Arc, Mutex},
};

pub struct FusionController;

impl FusionController {
    pub fn new(param: &FusionParam) -> anyhow::Result<Self> {
        // make sure destination exists
        if !param.destination.exists() {
            fs::create_dir_all(&param.destination)?;
        }

        Ok(FusionController)
    }

    /// convert rtf to pdf
    pub fn convert(
        &self,
        tasks: &[ConvertTask],
        status: Arc<Mutex<Sender<()>>>,
        logger: Arc<Mutex<Sender<String>>>,
    ) -> anyhow::Result<()> {
        let workers = worker_number();
        let converter = ConvertController::new(workers, status, logger);
        converter.execute(&tasks);
        Ok(())
    }

    /// combine outputs
    pub fn combine(
        &self,
        pdf_configs: &[(String, PathBuf)],
        rtf_configs: &[(PathBuf, Vec<PathBuf>)],
        status: Arc<Mutex<Sender<()>>>,
        logger: Arc<Mutex<Sender<String>>>,
    ) -> anyhow::Result<()> {
        let combiner_bin = combiner_bin().expect("Error: invalid binary combiner executor");

        let workers = worker_number();
        let pdf_controller = PDFCombineController::new(
            workers,
            Arc::clone(&status),
            Arc::clone(&logger),
            &combiner_bin,
        );
        let rtf_controller =
            RTFCombineController::new(workers, Arc::clone(&status), Arc::clone(&logger));
        pdf_controller.combine(pdf_configs);
        rtf_controller.combine(rtf_configs);
        Ok(())
    }
}

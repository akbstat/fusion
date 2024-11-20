use super::{combiner::combine, outline::add_outline};
use crate::config::combine::CombinePDFParam;
use std::{
    path::Path,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

pub struct PDFCombineWorker {
    thread: Option<thread::JoinHandle<()>>,
}

impl PDFCombineWorker {
    pub fn new(
        id: usize,
        bin: &Path,
        receiver: Arc<Mutex<mpsc::Receiver<CombinePDFParam>>>,
        status: Arc<Mutex<mpsc::Sender<()>>>,
        logger: Arc<Mutex<mpsc::Sender<String>>>,
    ) -> Self {
        let bin = bin.to_owned();
        let handler = thread::spawn(move || {
            logger
                .lock()
                .unwrap()
                .send(format!("[INFO] PDF combine worker {} launch\n", id))
                .ok();
            loop {
                let receiver = receiver.lock().unwrap().recv();
                match receiver {
                    Ok(mut param) => {
                        let filename = param.destination.clone();
                        let filename = filename.file_stem().unwrap().to_string_lossy();
                        logger
                            .lock()
                            .unwrap()
                            .send(format!("[INFO] {:?} pdf combine start\n", filename))
                            .ok();
                        // combine pdfs(with toc)
                        match combine(&mut param) {
                            Ok(_) => {
                                logger
                                    .lock()
                                    .unwrap()
                                    .send(format!("[INFO] {} pdf combine successfully\n", filename))
                                    .ok();
                                logger
                                    .lock()
                                    .unwrap()
                                    .send(format!("[INFO] {:?} add outline start\n", filename))
                                    .ok();
                                // combine pdf succesfully, then start to build outlines
                                match add_outline(&param.workspace, &param.to_outline_param(), &bin)
                                {
                                    Ok(_) => {
                                        status.lock().unwrap().send(()).ok();
                                        logger
                                            .lock()
                                            .unwrap()
                                            .send(format!(
                                                "[INFO] {} pdf combine complete\n",
                                                filename
                                            ))
                                            .ok();
                                    }
                                    Err(err) => {
                                        logger
                                            .lock()
                                            .unwrap()
                                            .send(format!(
                                                "[ERROR] {} add outline failed, because: {}\n",
                                                filename, err
                                            ))
                                            .ok();
                                    }
                                }
                            }
                            Err(err) => {
                                logger
                                    .lock()
                                    .unwrap()
                                    .send(format!(
                                        "[ERROR] {} pdf combine failed, because: {}\n",
                                        filename, err
                                    ))
                                    .ok();
                            }
                        };
                    }
                    Err(_) => break,
                }
                thread::sleep(Duration::from_millis(100));
            }
            logger
                .lock()
                .unwrap()
                .send(format!("[INFO] PDF combine worker {} exit\n", id))
                .ok();
        });
        PDFCombineWorker {
            thread: Some(handler),
        }
    }
    pub fn handler(&mut self) -> Option<thread::JoinHandle<()>> {
        self.thread.take()
    }
}

use std::{
    path::{Path, PathBuf},
    process::Command,
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
        receiver: Arc<Mutex<mpsc::Receiver<(String, PathBuf)>>>,
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
                    Ok(config) => {
                        let (name, config) = config;
                        logger
                            .lock()
                            .unwrap()
                            .send(format!("[INFO] {} pdf combine start\n", &name))
                            .ok();
                        let result = Command::new("cmd")
                            .arg("/C")
                            .arg(bin.clone())
                            .arg(config)
                            .output()
                            .unwrap();
                        if !result.status.success() {
                            let error_message = String::from_utf8(result.stderr).unwrap();
                            logger
                                .lock()
                                .unwrap()
                                .send(format!(
                                    "[ERROR] {} pdf combine failed, because: {}\n",
                                    &name, error_message
                                ))
                                .ok();
                        } else {
                            status.lock().unwrap().send(()).ok();
                            logger
                                .lock()
                                .unwrap()
                                .send(format!("[INFO] {} pdf combine complete\n", &name))
                                .ok();
                        }
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

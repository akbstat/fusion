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
                        Command::new("cmd")
                            .arg("/C")
                            .arg(bin.clone())
                            .arg(config)
                            .output()
                            .unwrap();
                        status.lock().unwrap().send(()).ok();
                        logger
                            .lock()
                            .unwrap()
                            .send(format!("[INFO] {} pdf combine complete\n", &name))
                            .ok();
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

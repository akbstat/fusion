use rtf2pdf::rtf2pdf;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use crate::config::convert::ConvertTask;

pub struct Worker {
    haneler: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(
        id: usize,
        logger: Arc<Mutex<mpsc::Sender<String>>>,
        receiver: Arc<Mutex<mpsc::Receiver<ConvertTask>>>,
        status: Arc<Mutex<mpsc::Sender<()>>>,
        cancel: Arc<Mutex<mpsc::Receiver<()>>>,
    ) -> Self {
        let handler = thread::spawn(move || {
            let is_cancel = Arc::new(Mutex::new(false));
            let is_cancel_clone = Arc::clone(&is_cancel);
            thread::spawn(move || loop {
                if let Err(_) = cancel.lock().unwrap().recv() {
                    *is_cancel_clone.lock().unwrap() = true;
                    return;
                }
            });
            logger
                .lock()
                .unwrap()
                .send(format!("[INFO] Convert worker {} launch\n", id))
                .ok();
            loop {
                if *is_cancel.lock().unwrap() {
                    break;
                }

                let task = receiver.lock().unwrap().recv();
                match task {
                    Ok(task) => {
                        let source = task.source.clone();
                        let destination = task.destination.clone();
                        let s = source.clone();
                        let task_name = s.file_stem().unwrap().to_string_lossy();
                        logger
                            .lock()
                            .unwrap()
                            .send(format!("[INFO] {} convert start\n", &task_name))
                            .ok();
                        rtf2pdf(vec![(source, destination)], &task.script).ok();
                        status.lock().unwrap().send(()).unwrap();
                        logger
                            .lock()
                            .unwrap()
                            .send(format!("[INFO] {} convert complete\n", task_name))
                            .ok();
                    }
                    Err(_) => break,
                }
                thread::sleep(Duration::from_millis(100));
            }

            logger
                .lock()
                .unwrap()
                .send(format!("[INFO] Convert worker {} exit\n", id))
                .ok();
        });
        Worker {
            haneler: Some(handler),
        }
    }
    pub fn handler(&mut self) -> Option<thread::JoinHandle<()>> {
        self.haneler.take()
    }
}

use std::sync::{mpsc, Arc, Mutex};

use crate::config::combine::RTFCombineParam;

use super::worker::RTFCombineWokrer;

pub struct RTFCombineController {
    sender: Option<mpsc::Sender<RTFCombineParam>>,
    workers: Vec<RTFCombineWokrer>,
}

impl RTFCombineController {
    pub fn new(
        worker_number: usize,
        status: Arc<Mutex<mpsc::Sender<()>>>,
        logger: Arc<Mutex<mpsc::Sender<String>>>,
    ) -> Self {
        let mut workers = Vec::with_capacity(worker_number);
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        for id in 0..worker_number {
            workers.push(RTFCombineWokrer::new(
                id,
                Arc::clone(&rx),
                Arc::clone(&status),
                Arc::clone(&logger),
            ));
        }
        RTFCombineController {
            workers,
            sender: Some(tx),
        }
    }

    pub fn combine(&self, params: &[RTFCombineParam]) {
        for param in params {
            if let Some(sender) = self.sender.as_ref() {
                sender.send(param.clone()).ok();
            }
        }
    }
}

impl Drop for RTFCombineController {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(worker) = worker.handler() {
                worker.join().unwrap();
            }
        }
    }
}

use std::{
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct RTFCombineWokrer {
    handler: Option<thread::JoinHandle<()>>,
}

impl RTFCombineWokrer {
    pub fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<(PathBuf, Vec<PathBuf>)>>>,
        status: Arc<Mutex<mpsc::Sender<()>>>,
        logger: Arc<Mutex<mpsc::Sender<String>>>,
    ) -> Self {
        let handler = thread::spawn(move || {
            logger
                .lock()
                .unwrap()
                .send(format!("[INFO] RTF combine worker {} launch\n", id))
                .ok();
            loop {
                let task = receiver.lock().unwrap().recv();
                match task {
                    Ok((destination, sources)) => {
                        let name = destination.file_stem().unwrap().to_string_lossy();
                        logger
                            .lock()
                            .unwrap()
                            .send(format!("[INFO] {} rtf combine start\n", name))
                            .ok();
                        rtf_operator::combiner::combine(&sources, &destination).unwrap();
                        status.lock().unwrap().send(()).ok();
                        logger
                            .lock()
                            .unwrap()
                            .send(format!("[INFO] {} rtf combine complete\n", name))
                            .ok();
                    }
                    Err(_) => break,
                }
            }
            logger
                .lock()
                .unwrap()
                .send(format!("[INFO] RTF combine worker {} exit\n", id))
                .ok();
        });
        RTFCombineWokrer {
            handler: Some(handler),
        }
    }
    pub fn handler(&mut self) -> Option<thread::JoinHandle<()>> {
        self.handler.take()
    }
}

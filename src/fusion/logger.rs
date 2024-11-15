use std::{
    fs::{read_to_string, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct Logger {
    log_path: PathBuf,
    reader: Mutex<File>,
    handler: Option<thread::JoinHandle<()>>,
}

impl Logger {
    pub fn new(write_channel: mpsc::Receiver<String>, log_path: &Path) -> anyhow::Result<Self> {
        let writer = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(log_path)?;
        let reader = OpenOptions::new().read(true).open(log_path)?;
        let writer = Arc::new(Mutex::new(writer));
        let handler = thread::spawn(move || loop {
            match write_channel.recv() {
                Ok(message) => {
                    writer.lock().unwrap().write(message.as_bytes()).ok();
                }
                Err(_) => break,
            }
        });
        Ok(Logger {
            reader: Mutex::new(reader),
            handler: Some(handler),
            log_path: log_path.into(),
        })
    }
    pub fn read(&self) -> anyhow::Result<String> {
        let mut message = String::new();
        if let Ok(mut reader) = self.reader.lock() {
            reader.read_to_string(&mut message)?;
        }
        Ok(message)
    }
    pub fn read_all(&self) -> anyhow::Result<String> {
        let contents = read_to_string(&self.log_path)?;
        Ok(contents)
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.handler.take();
    }
}

#[cfg(test)]
mod tests {

    use std::{fs, thread, time::Duration};

    use super::*;
    #[test]
    fn test_logger() -> anyhow::Result<()> {
        let log_path = Path::new(r"D:\Studies\ak112\303\stats\CSR\product\output\combined\log.txt");
        if log_path.exists() {
            fs::remove_file(log_path)?;
        }
        let mut handlers = vec![];
        let (tx, rx) = mpsc::channel();
        let tx = Arc::new(Mutex::new(tx));
        let logger = Logger::new(rx, log_path)?;
        handlers.push(thread::spawn(move || {
            for _ in 0..10 {
                let message = logger.read().unwrap();
                if !message.is_empty() {
                    println!("receive message:\n{}", message);
                }
                thread::sleep(Duration::from_millis(1000));
            }
        }));

        for i in 0..9 {
            let tx = Arc::clone(&tx);
            handlers.push(thread::spawn(move || {
                let message = format!("task {} start\n", i);
                tx.lock().unwrap().send(message).unwrap();
                thread::sleep(Duration::from_millis(5000));
                let message = format!("task {} complete\n", i);
                tx.lock().unwrap().send(message).unwrap();
            }));
        }
        for handler in handlers {
            handler.join().unwrap();
        }
        Ok(())
    }
}

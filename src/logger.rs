// use std::{
//     fs::{self, File, OpenOptions},
//     io::{Read, Write},
//     path::Path,
//     sync::{
//         atomic::{AtomicUsize, Ordering},
//         Mutex,
//     },
// };

// pub struct Logger {
//     reader: Mutex<File>,
//     writer: Mutex<File>,
//     convertion_finished_count: AtomicUsize,
// }

// impl Logger {
//     pub fn new(filepath: &Path) -> anyhow::Result<Self> {
//         if filepath.exists() {
//             fs::remove_file(filepath)?;
//         }
//         let writer = OpenOptions::new()
//             .create_new(true)
//             .append(true)
//             .open(filepath)?;
//         let reader = OpenOptions::new().read(true).open(filepath)?;
//         Ok(Logger {
//             reader: Mutex::new(reader),
//             writer: Mutex::new(writer),
//             convertion_finished_count: AtomicUsize::new(0),
//         })
//     }

//     pub fn worker_init(&self, worker_id: usize) {
//         if let Ok(mut writer) = self.writer.lock() {
//             writer
//                 .write(format!("[INFO] Worker {} has been initialized\n", worker_id).as_bytes())
//                 .ok();
//         }
//     }

//     pub fn worker_exit(&self, worker_id: usize) {
//         if let Ok(mut writer) = self.writer.lock() {
//             writer
//                 .write(format!("[INFO] Worker {} has been exited\n", worker_id).as_bytes())
//                 .ok();
//         }
//     }

//     pub fn convert_start(&self, item: &str) {
//         if let Ok(mut writer) = self.writer.lock() {
//             writer
//                 .write(format!("[INFO] Convertion Start: {}\n", item).as_bytes())
//                 .ok();
//         }
//     }

//     pub fn convert_finish(&self, item: &str) {
//         if let Ok(mut writer) = self.writer.lock() {
//             writer
//                 .write(format!("[INFO] Convertion Finished: {}\n", item).as_bytes())
//                 .ok();
//             self.convertion_finished_count
//                 .fetch_add(1, Ordering::SeqCst);
//         }
//     }

//     pub fn convert_failed(&self, item: &str, reason: &str) {
//         if let Ok(mut writer) = self.writer.lock() {
//             writer
//                 .write(format!("[ERROR] Convertion Failed, {}: {}\n", item, reason).as_bytes())
//                 .ok();
//         }
//     }

//     pub fn convertion_finished_count(&self) -> usize {
//         self.convertion_finished_count.load(Ordering::SeqCst)
//     }

//     pub fn combine_start(&self, item: &str) {
//         if let Ok(mut writer) = self.writer.lock() {
//             writer
//                 .write(format!("[INFO] Combine Start: {}\n", item).as_bytes())
//                 .ok();
//         }
//     }

//     pub fn combine_finish(&self, item: &str) {
//         if let Ok(mut writer) = self.writer.lock() {
//             writer
//                 .write(format!("[INFO] Combine Finished: {}\n", item).as_bytes())
//                 .ok();
//         }
//     }

//     pub fn combine_failed(&self, item: &str, reason: &str) {
//         if let Ok(mut writer) = self.writer.lock() {
//             writer
//                 .write(format!("[ERROR] Combine Failed, {}: {}\n", item, reason).as_bytes())
//                 .ok();
//         }
//     }

//     pub fn log(&self) -> anyhow::Result<String> {
//         let mut content = String::new();
//         if let Ok(mut reader) = self.reader.lock() {
//             reader.read_to_string(&mut content)?;
//         }
//         Ok(content)
//     }
// }

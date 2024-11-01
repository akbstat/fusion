use std::{
    ops::{Add, Div},
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
};

use super::param::FusionParam;

#[derive(Debug, Clone, PartialEq)]
pub enum FusionStage {
    Created,
    Converting,
    Combining,
    Completed,
}

pub struct ShareStates {
    convert_complete_number: Arc<Mutex<usize>>,
    combine_complete_number: Arc<Mutex<usize>>,
    convert_tasks: usize,
    combine_tasks: usize,
    convert_rx: Arc<Mutex<mpsc::Receiver<()>>>,
    combine_rx: Arc<Mutex<mpsc::Receiver<()>>>,
}

impl ShareStates {
    pub fn new(
        param: &FusionParam,
        convert_rx: mpsc::Receiver<()>,
        combine_rx: mpsc::Receiver<()>,
        combine_stage_notifier: Arc<Condvar>,
    ) -> Self {
        let convert_tasks = param.convert_task_numer();
        let combine_tasks = param.combine_task_number();
        let state = ShareStates {
            convert_complete_number: Arc::new(Mutex::new(0)),
            combine_complete_number: Arc::new(Mutex::new(0)),
            convert_tasks,
            combine_tasks,
            convert_rx: Arc::new(Mutex::new(convert_rx)),
            combine_rx: Arc::new(Mutex::new(combine_rx)),
        };
        state.run(combine_stage_notifier);
        state
    }

    fn run(&self, combine_stage_notifier: Arc<Condvar>) {
        let convert_rx = Arc::clone(&self.convert_rx);
        let combine_rx = Arc::clone(&self.combine_rx);
        let convert_complete_number = Arc::clone(&self.convert_complete_number);
        let combine_complete_number = Arc::clone(&self.combine_complete_number);
        let convert_tasks = self.convert_tasks;
        thread::spawn(move || loop {
            match convert_rx.lock().unwrap().recv() {
                Ok(_) => {
                    *convert_complete_number.lock().unwrap() += 1;
                    if (*convert_complete_number.lock().unwrap()).eq(&convert_tasks) {
                        combine_stage_notifier.notify_one();
                    }
                }
                Err(_) => return,
            };
        });
        thread::spawn(move || loop {
            match combine_rx.lock().unwrap().recv() {
                Ok(_) => *combine_complete_number.lock().unwrap() += 1,
                Err(_) => return,
            }
        });
    }

    pub fn progress(&self) -> (f64, FusionStage) {
        let convert_complete = *self.convert_complete_number.lock().unwrap();
        let combine_complete = *self.combine_complete_number.lock().unwrap();
        let complete_task = convert_complete.add(combine_complete) as f64;
        let all_task = self.convert_tasks.add(self.combine_tasks) as f64;
        if all_task.eq(&0f64) {
            return (100f64, FusionStage::Completed);
        }
        let progress = complete_task.div(all_task);
        let stage = if convert_complete.eq(&0) && combine_complete.eq(&0) {
            FusionStage::Created
        } else if convert_complete.lt(&self.convert_tasks) && combine_complete.eq(&0) {
            FusionStage::Converting
        } else if convert_complete.eq(&self.convert_tasks)
            && combine_complete.lt(&self.combine_tasks)
            && progress.lt(&100f64)
        {
            FusionStage::Combining
        } else {
            FusionStage::Completed
        };
        (progress, stage)
    }
}

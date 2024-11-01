use std::{
    env,
    path::Path,
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use fusion::{
    fusion::{
        controller::FusionController,
        logger::Logger,
        param::{FusionParam, FusionTask},
        status::{FusionStage, ShareStates},
    },
    utils::{workspace, File},
};

#[test]
fn fusion_integration_test() -> anyhow::Result<()> {
    let param = param();
    let id = param.id.clone();
    env_prepare()?;
    let workspace = workspace(id)?;
    let log_path = workspace.join("log.txt");
    let combine_stage_notifier = Arc::new(Condvar::new());
    let convert_complete = Arc::new(Mutex::new(false));

    // 0. prepare channels
    let (convert_tx, convert_rx) = mpsc::channel();
    let convert_tx = Arc::new(Mutex::new(convert_tx));
    let (_, combine_rx) = mpsc::channel();
    let (log_tx, log_rx) = mpsc::channel();
    let log_tx = Arc::new(Mutex::new(log_tx));
    // let (exit_tx, exit_rx) = mpsc::channel();

    // 1. prepare status machine
    let state_machine = ShareStates::new(
        &param,
        convert_rx,
        combine_rx,
        Arc::clone(&combine_stage_notifier),
    );

    // 2. prepare logger
    let logger = Logger::new(log_rx, &log_path)?;

    // 3. init controller
    let controller = FusionController::new(&param, &workspace)?;

    // 4. launch task
    let phase1 = Arc::clone(&convert_complete);
    let handler_0 = thread::spawn(move || {
        let mut phase = phase1.lock().unwrap();
        controller
            .convert(Arc::clone(&convert_tx), Arc::clone(&log_tx))
            .ok();
        while *phase {
            phase = combine_stage_notifier.wait(phase).unwrap();
        }
        println!("[RUNNER] Combine start");
        controller.combine().ok();
    });
    let phase2 = Arc::clone(&convert_complete);
    let handler_1 = thread::spawn(move || loop {
        let (progress, stage) = state_machine.progress();
        println!("[STATUS] Current progress: {:.2}", progress);
        println!("[STATUS] Current Stage: {:?}", stage);
        if stage.eq(&FusionStage::Combining) {
            *phase2.lock().unwrap() = true;
        }
        // if progress.ge(&100f64) {
        //     println!("share state machine exit");
        //     exit_tx.send(()).unwrap();
        //     return;
        // }
        thread::sleep(Duration::from_secs(1));
    });
    let handler_2 = thread::spawn(move || loop {
        let content = logger.read().unwrap();
        println!("[LOGGER] Fetch log: {}", content);
        thread::sleep(Duration::from_secs(1));
    });
    handler_0.join().unwrap();
    handler_1.join().unwrap();
    handler_2.join().unwrap();
    Ok(())
}

fn param() -> FusionParam {
    FusionParam {
        id: None,
        tasks: vec![FusionTask {
            name: "all_listings".into(),
            language: fusion::utils::Language::CN,
            cover: None,
            destination: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined")
                .into(),
            mode: fusion::utils::FusionMode::PDF,
            files: vec![
                File {
                    filename: "l-16-02-07-04-teae-wt-ss.rtf".into(),
                    title: "列表 16.2.7.4: 导致依沃西单抗/帕博利珠单抗永久停用的TEAE - 安全性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-07-04-teae-wt-ss.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-02-08-03-02-eq-index-fas.rtf".into(),
                    title: "表 14.2.8.3.2: EuroQol EQ-5D-5L问卷结果总结2 - 效应指数值和健康状态评分 - EuroQol EQ-5D-5L分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-02-08-03-02-eq-index-fas.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-03-03-01-05-irsae-ss.rtf".into(),
                    title: "表 14.3.3.1.5: 严重的irAE按照irAE分组、PT和CTCAE分级总结 - 安全性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-03-03-01-05-irsae-ss.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-03-04-05-01-eg-intp-ss.rtf".into(),
                    title: "表 14.3.4.5.1: ECG 整体评估 - 安全性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-03-04-05-01-eg-intp-ss.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-03-04-10-is-ada-sub-ims.rtf".into(),
                    title: "表 14.3.4.10: ADA检测结果与疗效相关的亚组分析 - 免疫原性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/t-14-03-04-10-is-ada-sub-ims.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "l-16-02-08-01-04-lb-thyrabn-ss.rtf".into(),
                    title: "Large size output 0".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/l-16-02-08-01-04-lb-thyrabn-ss.rtf").into(),
                    size: 0,
                },
            ],
        }],
    }
}

fn env_prepare() -> anyhow::Result<()> {
    const WORKER_NUMBER_ENV: &str = "MK_WORD_WORKER";
    const COMBINER_BIN: &str = "MK_COMBINER_BIN";
    const APP_ROOT: &str = "MK_FUSION";
    env::set_var(WORKER_NUMBER_ENV, 5.to_string());
    env::set_var(COMBINER_BIN, r"D:\projects\py\combiner\dist\combiner.exe");
    env::set_var(APP_ROOT, r"D:\Users\yuqi01.chen\.temp\app\mobiuskit\fusion");
    Ok(())
}
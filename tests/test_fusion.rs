use std::{
    env,
    path::Path,
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use fusion::{
    config::{
        param::{FusionParam, FusionTask},
        utils::{workspace, File, FusionMode, Language},
    },
    fusion::{controller::FusionController, logger::Logger, source::Source, state::ShareStates},
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
    let (exit_tx, exit_rx) = mpsc::channel();

    // 0. prepare channels
    let (convert_tx, convert_rx) = mpsc::channel();
    let convert_tx = Arc::new(Mutex::new(convert_tx));
    let (combine_tx, combine_rx) = mpsc::channel();
    let combine_tx = Arc::new(Mutex::new(combine_tx));
    let (log_tx, log_rx) = mpsc::channel();
    let log_tx = Arc::new(Mutex::new(log_tx));

    // filter converted output files
    let source = Source::new(&workspace)?;
    let convert_tasks = source.filter_convert_tasks(&param.to_convert_task(&workspace)?);
    source.update_source(&param.source)?;
    let (pdf_combine_config, rtf_combine_config) = param.to_combine_param(&workspace)?;
    // let convert_tasks = 0;
    // let combine_tasks = 0;

    // 1. prepare status machine
    let state_machine = ShareStates::new(
        convert_tasks.len(),
        pdf_combine_config.len() + rtf_combine_config.len(),
        convert_rx,
        combine_rx,
        Arc::clone(&combine_stage_notifier),
    );

    // 2. prepare logger
    let logger = Logger::new(log_rx, &log_path)?;

    // 3. init controller
    let controller = FusionController::new(&param)?;

    // 4. launch task
    let phase1 = Arc::clone(&convert_complete);
    let handler_0 = thread::spawn(move || {
        let mut phase = phase1.lock().unwrap();
        controller
            .convert(&convert_tasks, Arc::clone(&convert_tx), Arc::clone(&log_tx))
            .ok();
        while !*phase {
            phase = combine_stage_notifier.wait(phase).unwrap();
            *phase = true;
        }
        println!("[RUNNER] Combine start");
        controller
            .combine(
                &pdf_combine_config,
                &rtf_combine_config,
                Arc::clone(&combine_tx),
                Arc::clone(&log_tx),
            )
            .ok();
    });
    let handler_1 = thread::spawn(move || loop {
        let (progress, stage) = state_machine.progress();
        println!("[STATUS] Current progress: {:.2}", progress);
        println!("[STATUS] Current Stage: {:?}", stage);
        if progress.eq(&1f64) {
            exit_tx.send(()).ok();
            break;
        }
        thread::sleep(Duration::from_secs(1));
    });
    let handler_2 = thread::spawn(move || loop {
        let content = logger.read().unwrap();
        if !content.is_empty() {
            println!("[LOGGER] Fetch log: {}", content);
        }
        if let Ok(_) = exit_rx.try_recv() {
            break;
        }
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
        source: Path::new(r"D:\Studies\ak112\303\stats\CSR\product\output").into(),
        destination: Path::new(r"D:\Studies\ak112\303\stats\CSR\product\output\combined").into(),
        top: Path::new(r"D:\Studies\ak112\303\stats\CSR\utility\top-ak112-303-CSR.xlsx").into(),
        tasks: vec![FusionTask {
            name: "all_listings".into(),
            language: Language::EN,
            cover: Some(Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/cover.pdf").into()),
            destination: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined")
                .into(),
            mode: FusionMode::PDF,
            files: vec![
                File {
                    filename: "l-16-02-07-04-teae-wt-ss.rtf".into(),
                    title: "列表 16.2.7.4: 导致依沃西单抗/帕博利珠单抗永久停用的TEAE - 安全性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/l-16-02-07-04-teae-wt-ss.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-02-08-03-02-eq-index-fas.rtf".into(),
                    title: "表 14.2.8.3.2: EuroQol EQ-5D-5L问卷结果总结2 - 效应指数值和健康状态评分 - EuroQol EQ-5D-5L分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/t-14-02-08-03-02-eq-index-fas.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-03-03-01-05-irsae-ss.rtf".into(),
                    title: "表 14.3.3.1.5: 严重的irAE按照irAE分组、PT和CTCAE分级总结 - 安全性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/t-14-03-03-01-05-irsae-ss.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-03-04-05-01-eg-intp-ss.rtf".into(),
                    title: "表 14.3.4.5.1: ECG 整体评估 - 安全性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/t-14-03-04-05-01-eg-intp-ss.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-03-04-10-is-ada-sub-ims.rtf".into(),
                    title: "表 14.3.4.10: ADA检测结果与疗效相关的亚组分析 - 免疫原性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/t-14-03-04-10-is-ada-sub-ims.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "l-16-02-08-01-04-lb-thyrabn-ss.rtf".into(),
                    title: "Large size output 0".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/l-16-02-08-01-04-lb-thyrabn-ss.rtf").into(),
                    size: 0,
                },
            ],
        }, FusionTask {
            name: "all_listings".into(),
            language: Language::CN,
            cover: None,
            destination: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined")
                .into(),
            mode: FusionMode::RTF,
            files: vec![
                File {
                    filename: "l-16-02-07-04-teae-wt-ss.rtf".into(),
                    title: "列表 16.2.7.4: 导致依沃西单抗/帕博利珠单抗永久停用的TEAE - 安全性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/l-16-02-07-04-teae-wt-ss.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-02-08-03-02-eq-index-fas.rtf".into(),
                    title: "表 14.2.8.3.2: EuroQol EQ-5D-5L问卷结果总结2 - 效应指数值和健康状态评分 - EuroQol EQ-5D-5L分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/t-14-02-08-03-02-eq-index-fas.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-03-03-01-05-irsae-ss.rtf".into(),
                    title: "表 14.3.3.1.5: 严重的irAE按照irAE分组、PT和CTCAE分级总结 - 安全性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/t-14-03-03-01-05-irsae-ss.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-03-04-05-01-eg-intp-ss.rtf".into(),
                    title: "表 14.3.4.5.1: ECG 整体评估 - 安全性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/t-14-03-04-05-01-eg-intp-ss.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "t-14-03-04-10-is-ada-sub-ims.rtf".into(),
                    title: "表 14.3.4.10: ADA检测结果与疗效相关的亚组分析 - 免疫原性分析集".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/t-14-03-04-10-is-ada-sub-ims.rtf").into(),
                    size: 0,
                },
                File {
                    filename: "l-16-02-08-01-04-lb-thyrabn-ss.rtf".into(),
                    title: "Large size output 0".into(),
                    path: Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/l-16-02-08-01-04-lb-thyrabn-ss.rtf").into(),
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
    env::set_var(COMBINER_BIN, r"D:\projects\py\outlines\dist\outline.exe");
    env::set_var(APP_ROOT, r"D:\Users\yuqi01.chen\.temp\app\mobiuskit\fusion");
    Ok(())
}

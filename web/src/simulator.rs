/**
 * @file simulator.rs
 * @author Nguyen Le Duy
 * @date 04/05/2025
 * @brief Handling of simulator tasks
 */
use api_types::{CompilationResponse, Language};
use egui::Context;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::stream::StreamExt;
use futures::FutureExt;
use rp2350::simulator::Pico2;
use std::cell::RefCell;
use std::rc::Rc;
use web_time as time;

pub enum TaskCommand {
    Run,
    Pause,
    Step,
    Stop,
    FlashCode(Language, String),
}

pub fn pick_file_into_pico2(ctx: Context, pico2: Rc<RefCell<Pico2>>) {
    let file_picker = rfd::AsyncFileDialog::new();
    log::info!("Opening file picker");

    wasm_bindgen_futures::spawn_local(async move {
        log::info!("Waiting for file picker");
        let Some(file) = file_picker.pick_file().await else {
            log::warn!("No file selected");
            return;
        };

        let file_name = file.file_name();
        log::info!("Selected file: {}", file_name);

        if file_name.ends_with(".bin") {
            log::info!("Flashing bin file");
            let file = file.read().await;
            if let Err(why) = pico2.borrow_mut().flash_bin(&file) {
                log::error!("Failed to flash bin file: {}", why);
            }
        } else if file_name.ends_with(".uf2") {
            log::info!("Flashing uf2 file");
            let file = file.read().await;
            if let Err(why) = pico2.borrow_mut().flash_uf2(&file) {
                log::error!("Failed to flash uf2 file: {}", why);
            }
        } else {
            log::error!("Unsupported file type: {}", file_name);
            return;
        }

        ctx.request_repaint();
    })
}

async fn compile_source_code(lang: Language, code: &str) -> Result<Vec<u8>, String> {
    // The code maybe in a cache, so it may complete immediately
    let id = match crate::api::compile(lang, code).await? {
        CompilationResponse::InProgress { id } => id,
        CompilationResponse::Done { uf2 } => return Ok(uf2),
        CompilationResponse::Error { message } => return Err(message),
    };

    loop {
        // Check the status of the compilation
        let status_request = crate::api::compilation_result(&id).await?;
        match status_request {
            CompilationResponse::Done { uf2 } => {
                log::info!("Compilation done");
                return Ok(uf2);
            }
            CompilationResponse::Error { message } => {
                log::error!("Compilation error: {}", message);
                return Err(message);
            }
            CompilationResponse::InProgress { id } => {
                log::info!("Compilation in progress: {}", id);
                // Wait for a while before checking again
                gloo::timers::future::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn flash_code(ctx: &Context, pico2: &mut Pico2, lang: Language, code: &str) {
    // TODO add a loading spinner
    let uf2 = match compile_source_code(lang, code).await {
        Ok(uf2) => uf2,
        Err(err) => {
            log::error!("Failed to compile code: {}", err);
            // TODO show error message in the UI
            return;
        }
    };
    if let Err(why) = pico2.flash_uf2(&uf2) {
        log::error!("Failed to flash uf2 file: {}", why);
        // TODO show error message in the UI
        return;
    }

    // TODO add a message to the UI
}

pub fn run_pico2_sim(
    ctx: Context,
    pico2: Rc<RefCell<Pico2>>,
    is_running: Rc<RefCell<bool>>,
) -> Sender<TaskCommand> {
    let (tx, mut rx): (Sender<TaskCommand>, Receiver<TaskCommand>) = channel(4);

    wasm_bindgen_futures::spawn_local(async move {
        let mut now = time::Instant::now();
        let mut future = rx.next();
        let mut request_repaint = 5;

        loop {
            if *is_running.borrow() {
                now = time::Instant::now();
                pico2.borrow_mut().step();

                request_repaint -= 1;

                if request_repaint == 0 {
                    // Request repaint every 5 steps in running mode
                    request_repaint = 5;
                    ctx.request_repaint();
                }

                let elapsed = now.elapsed();
                let target = time::Duration::from_secs_f64(1.0 / 150.0e6);

                let sleep_dur = if elapsed < target {
                    target - elapsed
                } else {
                    time::Duration::ZERO
                };

                let timeout_future = gloo::timers::future::sleep(sleep_dur).fuse();
                let recv_future = rx.next().fuse();

                futures::pin_mut!(timeout_future, recv_future);

                futures::select_biased! {
                    _ = timeout_future => {},
                    cmd = recv_future => {
                        match cmd {
                            Some(TaskCommand::Stop) => {
                                *is_running.borrow_mut() = false;
                                pico2.borrow_mut().reset();
                            }
                            Some(TaskCommand::Pause) => *is_running.borrow_mut() = false,
                            Some(TaskCommand::FlashCode(language, code)) => {
                                *is_running.borrow_mut() = false;
                                flash_code(&ctx, &mut pico2.borrow_mut(), language, &code).await;
                            }
                            _ => {}
                        }
                    }
                };
            } else {
                match rx.next().await {
                    Some(TaskCommand::Run) => *is_running.borrow_mut() = true,
                    Some(TaskCommand::Step) => pico2.borrow_mut().step(),
                    Some(TaskCommand::Stop) => pico2.borrow_mut().reset(),
                    Some(TaskCommand::Pause) => *is_running.borrow_mut() = false,
                    Some(TaskCommand::FlashCode(language, code)) => {
                        flash_code(&ctx, &mut pico2.borrow_mut(), language, &code).await;
                    }
                    None => {}
                }
            }
        }
    });

    tx
}

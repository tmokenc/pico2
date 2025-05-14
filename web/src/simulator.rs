/**
 * @file simulator.rs
 * @author Nguyen Le Duy
 * @date 04/05/2025
 * @brief Handling of simulator tasks
 */
use crate::app::disassembler::Disassembler;
use api_types::{CompilationResponse, Language};
use egui::Context;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::stream::StreamExt;
use rp2350::simulator::Pico2;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{LazyLock, Mutex};

type ShoulSkipBootrom = bool;

static FLASHED_CODE: LazyLock<Mutex<Vec<u8>>> = LazyLock::new(|| Mutex::new(vec![]));

pub enum TaskCommand {
    Run,
    Pause,
    Step,
    Stop,
    FlashCode(Language, String, ShoulSkipBootrom, Rc<RefCell<bool>>),
}

pub fn pick_file_into_pico2(
    ctx: Context,
    pico2: Rc<RefCell<Pico2>>,
    skip_bootrom: ShoulSkipBootrom,
) {
    let file_picker = rfd::AsyncFileDialog::new();

    wasm_bindgen_futures::spawn_local(async move {
        let Some(file) = file_picker.pick_file().await else {
            crate::notify::warning("No file selected");
            return;
        };

        let file_name = file.file_name();
        crate::notify::info(format!("Selected file: {}", file_name));

        let mut pico2 = pico2.borrow_mut();

        if file_name.ends_with(".bin") {
            let file = file.read().await;
            if let Err(why) = pico2.flash_bin(&file) {
                crate::notify::error(format!("Failed to flash bin file: {}", why));
            } else {
                crate::notify::success("Flashed bin file successfully");
            }
        } else if file_name.ends_with(".uf2") {
            let file = file.read().await;
            if let Err(why) = pico2.flash_uf2(&file) {
                crate::notify::error(format!("Failed to flash uf2 file: {}", why));
            } else {
                crate::notify::success("Flashed uf2 file successfully");
            }

            // Save the flashed code to the global variable
            let mut flashed_code = FLASHED_CODE.lock().unwrap();
            flashed_code.clear();
            flashed_code.extend_from_slice(&file);
        } else {
            crate::notify::error(format!("Unsupported file type: {}", file_name));
            return;
        }

        if skip_bootrom {
            pico2.skip_bootrom();
        }

        drop(pico2);

        ctx.request_repaint();
    })
}

struct CompilationResult {
    uf2: Vec<u8>,
    disassembler: String,
}

pub fn export_file() {
    let file_picker = rfd::AsyncFileDialog::new()
        .set_file_name("main.uf2")
        .add_filter("UF2", &["uf2"])
        .save_file();

    wasm_bindgen_futures::spawn_local(async move {
        let Some(file) = file_picker.await else {
            crate::notify::warning("No file selected");
            return;
        };

        let file_name = file.file_name();
        crate::notify::info(format!("Selected file: {}", file_name));

        let flashed_code = FLASHED_CODE.lock().unwrap();
        if let Err(why) = file.write(&flashed_code).await {
            crate::notify::error(format!("Failed to write to file: {}", why));
        } else {
            crate::notify::success("Exported code successfully");
        }
    });
}

async fn compile_source_code(lang: Language, code: &str) -> Result<CompilationResult, String> {
    // The code maybe in a cache, so it may complete immediately
    let id = match crate::api::compile(lang, code).await? {
        CompilationResponse::InProgress { id } => id,
        CompilationResponse::Done { uf2, disassembler } => {
            return Ok(CompilationResult { uf2, disassembler })
        }
        CompilationResponse::Error { message } => return Err(message),
    };

    loop {
        // Check the status of the compilation
        let status_request = crate::api::compilation_result(&id).await?;
        match status_request {
            CompilationResponse::Done { uf2, disassembler } => {
                log::info!("Compilation done");
                return Ok(CompilationResult { uf2, disassembler });
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

async fn flash_code(
    pico2: Rc<RefCell<Pico2>>,
    lang: Language,
    code: &str,
    skip_bootrom: bool,
    disassembler: &Rc<RefCell<Disassembler>>,
) {
    // TODO add a loading spinner
    let res = match compile_source_code(lang, code).await {
        Ok(res) => res,
        Err(err) => {
            crate::notify::error(format!("Failed to compile code: {}", err));
            return;
        }
    };

    let mut mcu = pico2.borrow_mut();
    if let Err(why) = mcu.flash_uf2(&res.uf2) {
        crate::notify::error(format!("Failed to flash uf2 file: {}", why));
        return;
    }

    if skip_bootrom {
        mcu.skip_bootrom();
    }

    {
        let mut disassembler = disassembler.borrow_mut();
        disassembler.update_file(&res.disassembler);
    }

    crate::notify::success("Code flashed successfully");
}

pub fn run_pico2_sim(
    ctx: Context,
    pico2: Rc<RefCell<Pico2>>,
    is_running: Rc<RefCell<bool>>,
    disassembler: Rc<RefCell<Disassembler>>,
) -> Sender<TaskCommand> {
    let (tx, mut rx): (Sender<TaskCommand>, Receiver<TaskCommand>) = channel(4);

    wasm_bindgen_futures::spawn_local(async move {
        let mut request_repaint = 5;
        let mut skipped_bootrom = false;

        loop {
            if *is_running.borrow() {
                request_repaint -= 1;

                {
                    let mut pico2 = pico2.borrow_mut();
                    pico2.step();
                    let pc0 = pico2.processor[0].get_pc();
                    let pc1 = pico2.processor[1].get_pc();
                    drop(pico2);
                    let disassembler = disassembler.borrow();
                    if disassembler.has_breakpoint(&pc0) || disassembler.has_breakpoint(&pc1) {
                        drop(disassembler);
                        *is_running.borrow_mut() = false;
                    }
                }

                if request_repaint == 0 {
                    // Request repaint every 2000 steps in running mode
                    request_repaint = 2000;
                    ctx.request_repaint();
                    // try to notify the async runtime to avoid blocking
                    yield_now().await;
                }

                match rx.try_next() {
                    Ok(Some(TaskCommand::Stop)) => {
                        *is_running.borrow_mut() = false;
                        pico2.borrow_mut().reset();
                        if skipped_bootrom {
                            pico2.borrow_mut().skip_bootrom();
                        }
                    }
                    Ok(Some(TaskCommand::Pause)) => *is_running.borrow_mut() = false,
                    Ok(Some(TaskCommand::FlashCode(language, code, skip_bootrom, is_flashing))) => {
                        *is_running.borrow_mut() = false;
                        *is_flashing.borrow_mut() = true;
                        skipped_bootrom = skip_bootrom;
                        flash_code(pico2.clone(), language, &code, skip_bootrom, &disassembler)
                            .await;
                        *is_flashing.borrow_mut() = false;
                    }
                    _ => {}
                }
            } else {
                match rx.next().await {
                    Some(TaskCommand::Run) => *is_running.borrow_mut() = true,
                    Some(TaskCommand::Step) => pico2.borrow_mut().step(),
                    Some(TaskCommand::Stop) => {
                        pico2.borrow_mut().reset();
                        if skipped_bootrom {
                            pico2.borrow_mut().skip_bootrom();
                        }
                    }
                    Some(TaskCommand::Pause) => *is_running.borrow_mut() = false,
                    Some(TaskCommand::FlashCode(language, code, skip_bootrom, is_flashing)) => {
                        *is_flashing.borrow_mut() = true;
                        flash_code(pico2.clone(), language, &code, skip_bootrom, &disassembler)
                            .await;
                        *is_flashing.borrow_mut() = false;
                    }
                    None => {}
                }
            }
        }
    });

    tx
}

fn yield_now() -> impl Future<Output = ()> {
    gloo::timers::future::TimeoutFuture::new(0)
}

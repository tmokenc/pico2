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

pub fn run_pico2_sim(ctx: Context, pico2: Rc<RefCell<Pico2>>) -> Sender<TaskCommand> {
    let (tx, mut rx): (Sender<TaskCommand>, Receiver<TaskCommand>) = channel(4);

    wasm_bindgen_futures::spawn_local(async move {
        let mut is_running = false;
        let mut now = time::Instant::now();
        let mut future = rx.next();
        let mut request_repaint = 5;

        loop {
            if is_running {
                now = time::Instant::now();
                pico2.borrow_mut().step();

                log::info!("Pico2 step");
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

                log::info!("Pico2 sleep for {:?}", sleep_dur);

                futures::select_biased! {
                    _ = timeout_future => {},
                    cmd = recv_future => {
                        match cmd {
                            Some(TaskCommand::Stop) => {
                                is_running = false;
                                pico2.borrow_mut().reset();
                            }
                            Some(TaskCommand::Pause) => is_running = false,
                            _ => {}
                        }
                    }
                };

                log::info!("Pico2 woke up");
            } else {
                match rx.next().await {
                    Some(TaskCommand::Run) => is_running = true,
                    Some(TaskCommand::Step) => pico2.borrow_mut().step(),
                    Some(TaskCommand::Stop) => {
                        is_running = false;
                        pico2.borrow_mut().reset();
                    }
                    Some(TaskCommand::Pause) => is_running = false,
                    None => {}
                }
            }
        }
    });

    tx
}

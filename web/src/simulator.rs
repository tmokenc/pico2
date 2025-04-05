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

pub fn run_pico2_sim(pico2: Rc<RefCell<Pico2>>) -> Sender<TaskCommand> {
    let (tx, mut rx): (Sender<TaskCommand>, Receiver<TaskCommand>) = channel(4);

    wasm_bindgen_futures::spawn_local(async move {
        loop {
            let mut is_running = false;
            let mut now = time::Instant::now();
            let mut future = rx.next();

            loop {
                if is_running {
                    now = time::Instant::now();
                    pico2.borrow_mut().step();

                    log::info!("Pico2 step");

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
                                Some(TaskCommand::Stop) => is_running = false,
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
        }
    });

    tx
}

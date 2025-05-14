/**
 * @file compile.rs
 * @author Nguyen Le Duy
 * @date 09/04/2025
 * @brief Handling compilation requests and responses.
 */
use api_types::*;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::time::sleep;
use warp::reject::Reject;

use crate::config::ServerConfig;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Compilation error: {0}")]
    CompilationError(String),
    #[error("No code provided")]
    NoCode,
    #[error("Unsupported multiple files")]
    UnsupportedMultipleFiles,
    #[error("Unsupported file name. Currently only main.c is allowed")]
    UnsupportedFileName,
    #[error("File system error: {0}")]
    FileSystemError(#[from] std::io::Error),
}

impl Reject for CompileError {}

const MAX_RESULT_STORAGE_LEN: usize = 500;

type Id = String;

#[derive(Debug)]
enum CompilationStatus {
    Success,
    Failure(CompileError),
    InProgress,
}

impl PartialEq for CompilationStatus {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Success, Self::Success)
                | (Self::InProgress, Self::InProgress)
                | (Self::Failure(_), Self::Failure(_))
        )
    }
}

impl Eq for CompilationStatus {}

struct CompilationResult {
    status: CompilationStatus,
    updated_on: Instant,
    served: bool,
}

pub struct Compiler {
    results: Arc<Mutex<HashMap<Id, CompilationResult>>>,
    queue: Arc<Mutex<VecDeque<(Id, CompilationRequest)>>>,
    notifier: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    build_dir: PathBuf,
    data_dir: PathBuf,
    result_dir: PathBuf,
}

impl Compiler {
    pub async fn new(config: &ServerConfig) -> Result<Self, CompileError> {
        ensure_new_dir(&config.data_dir).await?;
        let data_dir = fs::canonicalize(PathBuf::from(&config.data_dir)).await?;
        let build_dir = data_dir.join("build");
        let result_dir = data_dir.join("results");

        if !has_dir(&result_dir).await? {
            fs::create_dir(&result_dir).await?;
        }

        let res = Self {
            results: Default::default(),
            queue: Default::default(),
            notifier: Default::default(),
            build_dir,
            result_dir,
            data_dir,
        };

        res.prepare_build_env(config).await?;
        res.spawn_compilation_handler();
        res.spawm_clean_up_task();

        Ok(res)
    }

    fn spawm_clean_up_task(&self) {
        let results_lock = self.results.clone();
        let result_dir = self.build_dir.clone();

        tokio::spawn(async move {
            loop {
                let mut results = results_lock.lock().await;

                // TODO should check by length or by total size???
                if results.len() > MAX_RESULT_STORAGE_LEN {
                    let mut keys: Vec<Id> = results
                        .iter()
                        .filter_map(|(k, v)| {
                            if v.status == CompilationStatus::InProgress {
                                None
                            } else {
                                Some(Id::from(k))
                            }
                        })
                        .collect();

                    keys.sort_unstable_by(|a, b| {
                        let a = results.get(a).unwrap();
                        let b = results.get(b).unwrap();

                        if a.served && b.served {
                            a.updated_on.cmp(&b.updated_on)
                        } else if a.served {
                            Ordering::Less
                        } else {
                            Ordering::Greater
                        }
                    });

                    for key in keys.iter().take(results.len() - MAX_RESULT_STORAGE_LEN) {
                        results.remove(key);
                        let _ = fs::remove_file(result_dir.join(format!("{key}.uf2"))).await;
                        let _ = fs::remove_file(result_dir.join(format!("{key}.dis"))).await;
                    }
                }

                drop(results);
                sleep(Duration::from_secs(60)).await; // do it one per minute
            }
        });
    }

    fn spawn_compilation_handler(&self) {
        let notifier = self.notifier.clone();
        let queue = self.queue.clone();
        let results = self.results.clone();
        let build_dir = self.build_dir.clone();
        let result_dir = self.result_dir.clone();

        tokio::spawn(async move {
            loop {
                while let Some((id, req)) = queue.lock().await.pop_front() {
                    log::info!("Compiling request {id}");

                    // Add one again here to avoid data race
                    results.lock().await.insert(
                        id.clone(),
                        CompilationResult {
                            status: CompilationStatus::InProgress,
                            updated_on: Instant::now(),
                            served: false,
                        },
                    );

                    let res = match req.lang {
                        Language::C => compile_c_code(&id, &req, &build_dir, &result_dir).await,
                    };

                    log::info!("Request {id} done");
                    results.lock().await.insert(
                        id,
                        CompilationResult {
                            status: match res {
                                Ok(_) => CompilationStatus::Success,
                                Err(e) => CompilationStatus::Failure(e),
                            },
                            updated_on: Instant::now(),
                            served: false,
                        },
                    );
                }

                // Queue is empty, wait for it to has at least one
                let (tx, rx) = oneshot::channel();
                notifier.lock().await.replace(tx);
                let _ = rx.await;
            }
        });
    }

    pub async fn compile(&mut self, req: CompilationRequest) -> CompilationResponse {
        let id = generate_id();

        // TODO caching to avoid compile the same code multiple times
        // TOOD clean up

        self.queue.lock().await.push_back((id.clone(), req));
        self.results.lock().await.insert(
            id.clone(),
            CompilationResult {
                status: CompilationStatus::InProgress,
                updated_on: Instant::now(),
                served: false,
            },
        );

        log::info!("Added request {id} to the queue");

        // Notify the compilation handler to continue its work
        if let Some(notifier) = self.notifier.lock().await.take() {
            let _ = notifier.send(());
        }

        CompilationResponse::InProgress { id }
    }

    pub async fn get_uf2(&mut self, id: &str) -> Result<Vec<u8>, CompileError> {
        let uf2_path = self.result_dir.join(format!("{}.uf2", id));
        fs::read(uf2_path)
            .await
            .map_err(CompileError::FileSystemError)
    }

    pub async fn get_dis(&mut self, id: &str) -> Result<String, CompileError> {
        let dis_path = self.result_dir.join(format!("{}.dis", id));
        fs::read_to_string(dis_path)
            .await
            .map_err(CompileError::FileSystemError)
    }

    pub async fn get_result(&mut self, id: &str) -> CompilationResponse {
        let mut lock = self.results.lock().await;
        let Some(result) = lock.get_mut(id) else {
            return CompilationResponse::Error {
                message: String::from("ID not found or have been cleaned up"),
            };
        };

        match &result.status {
            CompilationStatus::InProgress => CompilationResponse::InProgress { id: id.to_string() },
            CompilationStatus::Success => {
                result.served = true;
                drop(lock);

                let uf2 = match self.get_uf2(id).await {
                    Ok(uf2) => uf2,
                    Err(e) => {
                        return CompilationResponse::Error {
                            message: e.to_string(),
                        }
                    }
                };

                let dis = match self.get_dis(id).await {
                    Ok(dis) => dis,
                    Err(e) => {
                        return CompilationResponse::Error {
                            message: e.to_string(),
                        }
                    }
                };

                CompilationResponse::Done {
                    uf2,
                    disassembler: dis,
                }
            }
            CompilationStatus::Failure(e) => {
                result.served = true;
                CompilationResponse::Error {
                    message: e.to_string(),
                }
            }
        }
    }

    pub async fn prepare_build_env(&self, config: &ServerConfig) -> Result<(), CompileError> {
        log::info!("Preparing build environment");

        const CMAKE_FILE: &'static [u8] = include_bytes!("../assets/CMakeLists.txt");
        const TOOLCHAIN_FILE: &'static [u8] = include_bytes!("../assets/pico_sdk_import.cmake");
        const DUMMY_FILE: &'static [u8] = include_bytes!("../assets/dummy_main.c");

        let sdk_path = config.pico_sdk.as_deref();

        if !has_dir(&self.data_dir).await? {
            fs::create_dir(&self.data_dir).await?;
        }

        ensure_new_dir(&self.build_dir).await?;
        ensure_new_dir(&self.build_dir.join("build")).await?;
        fs::write(self.build_dir.join("CMakeLists.txt"), CMAKE_FILE).await?;
        fs::write(self.build_dir.join("pico_sdk_import.cmake"), TOOLCHAIN_FILE).await?;
        fs::write(self.build_dir.join("main.c"), DUMMY_FILE).await?;

        // Run cmake
        let mut cmd = Command::new("cmake");

        cmd.current_dir(self.build_dir.join("build"))
            .arg(&self.build_dir)
            .arg("-DPICO_BOARD=pico2")
            .arg("-DPICO_PLATFORM=rp2350-riscv");

        if let Some(path) = sdk_path {
            cmd.arg(format!("-DPICO_SDK_PATH={}", path));
        }

        let cmake_build_result = cmd.output().await?;

        if !cmake_build_result.status.success() {
            return Err(CompileError::CompilationError(format!(
                "Failed to run cmake: {}",
                String::from_utf8_lossy(&cmake_build_result.stderr),
            )));
        }

        // Initial build to speed up the first compilation
        Command::new("make")
            .current_dir(self.build_dir.join("build"))
            .output()
            .await?;

        Ok(())
    }
}

async fn ensure_new_dir(path: impl AsRef<Path>) -> Result<(), CompileError> {
    if has_dir(&path).await? {
        fs::remove_dir_all(&path).await?;
    }

    fs::create_dir(path).await?;
    Ok(())
}

async fn has_dir(path: impl AsRef<Path>) -> Result<bool, CompileError> {
    if !fs::try_exists(&path).await? {
        return Ok(false);
    }

    Ok(fs::metadata(path).await?.is_dir())
}

fn generate_id() -> String {
    nanoid::nanoid!(21, &nanoid::alphabet::SAFE)
}

async fn compile_c_code(
    id: &str,
    req: &CompilationRequest,
    build_dir: impl AsRef<Path>,
    result_dir: impl AsRef<Path>,
) -> Result<(), CompileError> {
    if req.source.len() > 1 {
        return Err(CompileError::UnsupportedMultipleFiles);
    }

    let Some(code) = req.source.iter().next() else {
        return Err(CompileError::NoCode);
    };

    if code.filename != "main.c" {
        return Err(CompileError::UnsupportedFileName);
    }

    let build_dir = build_dir.as_ref();
    let result_dir = result_dir.as_ref();

    let path = build_dir.join(&code.filename);
    let build_path = build_dir.join("build");
    let uf2_path = result_dir.join(format!("{}.uf2", id));
    let dis_path = result_dir.join(format!("{}.dis", id));
    fs::write(path, &code.code).await?;

    let mut cmd = Command::new("make");
    cmd.current_dir(&build_path);

    let Ok(process) = cmd.output().await else {
        let comp_err =
            CompileError::CompilationError("Failed to start the compilation process".to_string());

        return Err(comp_err);
    };

    if !process.status.success() {
        let comp_err =
            CompileError::CompilationError(String::from_utf8_lossy(&process.stderr).to_string());

        return Err(comp_err);
    }

    log::info!("Compilation successful");
    fs::rename(build_path.join("main.uf2"), uf2_path.clone()).await?;
    fs::rename(build_path.join("main.dis"), dis_path.clone()).await?;
    Ok(())
}

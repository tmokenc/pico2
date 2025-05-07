/**
 * @file compile.rs
 * @author Nguyen Le Duy
 * @date 09/04/2025
 * @brief Handling compilation requests and responses.
 */
use api_types::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
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

#[derive(Debug)]
enum CompilationStatus {
    Success,
    Failure(CompileError),
    InProgress,
}

pub struct Compiler {
    results: Arc<Mutex<HashMap<String, CompilationStatus>>>,
    sender: mpsc::UnboundedSender<(String, CompilationStatus)>,
    build_dir: PathBuf,
    data_dir: PathBuf,
    result_dir: PathBuf,
}

impl Compiler {
    pub async fn new(config: &ServerConfig) -> Result<Self, CompileError> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        ensure_new_dir(&config.data_dir).await?;
        let data_dir = fs::canonicalize(PathBuf::from(&config.data_dir)).await?;
        let build_dir = data_dir.join("build");
        let result_dir = data_dir.join("results");

        if !has_dir(&result_dir).await? {
            fs::create_dir(&result_dir).await?;
        }

        let res = Self {
            results: Default::default(),
            sender: tx,
            build_dir,
            result_dir,
            data_dir,
        };

        res.prepare_build_env(config).await?;
        let results_clone = Arc::clone(&res.results);

        tokio::spawn(async move {
            while let Some((id, status)) = rx.recv().await {
                log::info!("Received status for id {}: {:?}", id, status);
                results_clone.lock().await.insert(id, status);
            }
        });

        Ok(res)
    }

    pub async fn compile(&mut self, req: CompilationRequest) -> CompilationResponse {
        let id = generate_id();

        let res = match req.lang {
            Language::C => self.compile_c_code(&id, &req).await,
        };

        match res {
            Ok(_) => match self.get_uf2(&id).await {
                Ok(uf2) => CompilationResponse::Done { uf2 },
                Err(e) => CompilationResponse::Error {
                    message: e.to_string(),
                },
            },
            Err(e) => CompilationResponse::Error {
                message: e.to_string(),
            },
        }
    }

    pub async fn get_uf2(&mut self, id: &str) -> Result<Vec<u8>, CompileError> {
        let uf2_path = self.result_dir.join(format!("{}.uf2", id));
        fs::read(uf2_path)
            .await
            .map_err(CompileError::FileSystemError)
    }

    pub async fn get_result(&mut self, id: &str) -> CompilationResponse {
        let lock = self.results.lock().await;

        match lock.get(id) {
            Some(CompilationStatus::InProgress) => {
                CompilationResponse::InProgress { id: id.to_string() }
            }
            Some(CompilationStatus::Success) => {
                drop(lock);

                match self.get_uf2(id).await {
                    Ok(uf2) => CompilationResponse::Done { uf2 },
                    Err(e) => CompilationResponse::Error {
                        message: e.to_string(),
                    },
                }
            }
            Some(CompilationStatus::Failure(e)) => CompilationResponse::Error {
                message: e.to_string(),
            },
            None => CompilationResponse::Error {
                message: "Unknown ID".to_string(),
            },
        }
    }

    async fn compile_c_code(&self, id: &str, req: &CompilationRequest) -> Result<(), CompileError> {
        if req.source.len() > 1 {
            return Err(CompileError::UnsupportedMultipleFiles);
        }

        let Some(code) = req.source.iter().next() else {
            return Err(CompileError::NoCode);
        };

        if code.filename != "main.c" {
            return Err(CompileError::UnsupportedFileName);
        }

        let path = self.build_dir.join(&code.filename);
        let build_path = self.build_dir.join("build");
        let uf2_path = self.result_dir.join(format!("{}.uf2", id));
        fs::write(path, &code.code).await?;

        let id = id.to_owned();
        let tx = self.sender.clone();

        tokio::spawn(async move {
            tx.send((id.to_owned(), CompilationStatus::InProgress))
                .unwrap();

            let mut cmd = Command::new("make");
            cmd.current_dir(&build_path);

            let Ok(process) = cmd.output().await else {
                let err = CompilationStatus::Failure(CompileError::CompilationError(
                    "Failed to start the compilation process".to_string(),
                ));

                tx.send((id, err)).unwrap();
                return;
            };

            if !process.status.success() {
                let err = CompilationStatus::Failure(CompileError::CompilationError(
                    String::from_utf8_lossy(&process.stderr).to_string(),
                ));

                tx.send((id, err)).unwrap();
                return;
            }

            match fs::rename(build_path.join("main.uf2"), uf2_path).await {
                Ok(_) => {
                    tx.send((id, CompilationStatus::Success)).unwrap();
                }
                Err(e) => {
                    let err = CompilationStatus::Failure(CompileError::FileSystemError(e));
                    tx.send((id, err)).unwrap();
                }
            }
        });

        Ok(())
    }

    pub async fn prepare_build_env(&self, config: &ServerConfig) -> Result<(), CompileError> {
        const CMAKE_FILE: &'static [u8] = include_bytes!("../assets/CMakeLists.txt");
        const TOOLCHAIN_FILE: &'static [u8] = include_bytes!("../assets/pico_sdk_import.cmake");
        const DUMMY_FILE: &'static [u8] = include_bytes!("../assets/dummy_main.c");

        let sdk_path = config.sdk_path.as_deref();

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

        // .arg("-DPICO_SDK_PATH=/opt/pico-sdk")
        let cmake_build_result = cmd.output().await?;

        if !cmake_build_result.status.success() {
            return Err(CompileError::CompilationError(format!(
                "Failed to run cmake: {}",
                String::from_utf8_lossy(&cmake_build_result.stderr),
            )));
        }

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

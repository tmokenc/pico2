//! pico2
//! Author: Nguyen Le Duy
//! Date: 06/04/2025
//! Description: This module defines the data structures and enums used for
//! interacting between the client and server in the pico2 project.

use serde::{Deserialize, Serialize};

/// Represents the response from the server after a compilation request.
/// It can be in one of three states:
/// 1. InProgress: The compilation is still ongoing.
/// 2. Done: The compilation has completed successfully.
/// 3. Error: An error occurred during the compilation process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompilationResponse {
    /// The compilation is still in progress.
    InProgress {
        /// Unique identifier for the compilation status request.
        id: String,
    },
    /// The compilation has completed successfully.
    Done {
        /// uf2 binary data in base64 format.
        #[serde(with = "serde_bytes")]
        uf2: Vec<u8>,
    },
    /// An error occurred during the compilation process.
    Error { message: String },
}

/// Supported programming languages for compilation.
/// Currently, only C is supported.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "c")]
    C,
}

/// Supported compilation target architectures.
/// Currently, only RISC-V is supported.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Target {
    #[serde(rename = "riscv")]
    RiscV,
}

/// Represents the source code to be compiled.
/// It includes the filename and the actual code content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCode {
    pub filename: String,
    pub code: String,
}

/// Represents a request to compile source code.
/// It includes the programming language, source code, target architecture,
/// and optional compiler options.
/// The source code is provided as a vector of `SourceCode` structs.
/// The `compiler_options` field is optional and can be used to specify
/// additional compilation flags or settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationRequest {
    /// The programming language of the source code.
    pub lang: Language,
    /// The source code to be compiled.
    pub source: Vec<SourceCode>,
    /// The target architecture for the compilation.
    pub target: Target,
    /// Optional compiler options.
    pub compiler_options: Option<String>,
}

/// Represents a request to check the status of a compilation.
/// It includes the unique identifier of the compilation request.
/// The server will use this ID to look up the status of the compilation process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationStatusRequest {
    /// Unique identifier for the compilation request.
    pub id: String,
}

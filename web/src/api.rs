//! pico2
//! Author: Nguyen Le Duy
//! Date: 06/04/2025
//! Description: Handling API for the pico2 project

use api_types::*;

/// Represents the result of a compilation process.
pub async fn compile(code: &str) -> Result<CompilationResponse, String> {
    let compilation_request = CompilationRequest {
        lang: Language::C,
        source: vec![SourceCode {
            filename: "main.c".to_string(),
            code: code.to_string(),
        }],
        target: Target::RiscV,
        compiler_options: None,
    };

    let request =
        ehttp::Request::json("/api/compile", &compilation_request).map_err(|e| e.to_string())?;

    ehttp::fetch_async(request)
        .await
        .map_err(|e| e.to_string())
        .and_then(|response| {
            if response.ok {
                response
                    .json::<CompilationResponse>()
                    .map_err(|e| e.to_string())
            } else {
                Err(format!("Error: {}", response.status))
            }
        })
}

pub async fn compilation_result(id: &str) -> Result<CompilationResponse, String> {
    let compilation_status_request = CompilationStatusRequest { id: id.to_string() };

    let request = ehttp::Request::json("/api/result", &compilation_status_request)
        .map_err(|e| e.to_string())?;

    ehttp::fetch_async(request)
        .await
        .map_err(|e| e.to_string())
        .and_then(|response| {
            if response.ok {
                response
                    .json::<CompilationResponse>()
                    .map_err(|e| e.to_string())
            } else {
                Err(format!("Error: {}", response.status))
            }
        })
}

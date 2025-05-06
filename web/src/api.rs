/**
 * @file api.rs
 * @author Nguyen Le Duy
 * @date 14/04/2025
 * @brief API module to communicate with the server
 */
use api_types::*;

/// Represents the result of a compilation process.
pub async fn compile(lang: Language, code: &str) -> Result<CompilationResponse, String> {
    let compilation_request = CompilationRequest {
        lang,
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

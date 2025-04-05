use serde::{Deserialize, Serialize};
use std::net;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use warp::Filter;

mod compile;
mod config;

use compile::*;

const CONFIG_PATH: &str = "config.toml";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::ServerConfig::parse(CONFIG_PATH)?;

    let dir = fs::canonicalize(&config.dir).await?;
    let index_file = warp::fs::file(dir.join("index.html"));
    let ip_address: net::IpAddr = config.ip.parse().expect("Invalid IP address");

    let compiler = Compiler::new(&config).await?;
    let compiler = Arc::new(Mutex::new(compiler));
    let compiler_clone = compiler.clone();

    // Compile endpoint
    let compile_route = warp::path("compile")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || compiler_clone.clone()))
        .and_then(compile_handler);

    // Result endpoint
    let result_route = warp::path("result")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || compiler.clone()))
        .and_then(result_handler);

    // let logger = warp::any().map(warp::reply).with(warp::log("server"));

    // Serve static files from a directory
    let static_files = warp::fs::dir(dir);
    let index = warp::path::end().and(index_file);

    // Combine routes
    let api = warp::path("api").and(compile_route.or(result_route));
    let routes = index.or(static_files).or(api);

    // Start the server
    warp::serve(routes).run((ip_address, config.port)).await;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
enum CompilationResponse {
    InProgress { id: String },
    Done { uf2: Vec<u8> },
    Error { message: String },
}

async fn compile_handler(
    request: CompileRequest,
    compiler: Arc<Mutex<Compiler>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut compiler = compiler.lock().await;
    let result = compiler.compile(request).await;
    drop(compiler); // Release the lock before sending the response
    Ok(warp::reply::json(&result))
}

async fn result_handler(
    request: CompileResultRequest,
    compiler: Arc<Mutex<Compiler>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut compiler = compiler.lock().await;
    let result = compiler.get_result(&request.id).await;
    drop(compiler); // Release the lock before sending the response
    Ok(warp::reply::json(&result))
}

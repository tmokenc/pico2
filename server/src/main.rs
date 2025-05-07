/**
 * @file main.rs
 * @author Nguyen Le Duy
 * @date 09/04/2025
 * @brief Main entry point for the server.
 */
use api_types::{CompilationRequest, CompilationStatusRequest};
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
    println!("Config: {:?}", config);

    let dir = fs::canonicalize(&config.static_dir).await?;

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

async fn compile_handler(
    request: CompilationRequest,
    compiler: Arc<Mutex<Compiler>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut compiler = compiler.lock().await;
    let result = compiler.compile(request).await;
    drop(compiler); // Release the lock before sending the response
    Ok(warp::reply::json(&result))
}

async fn result_handler(
    request: CompilationStatusRequest,
    compiler: Arc<Mutex<Compiler>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut compiler = compiler.lock().await;
    let result = compiler.get_result(&request.id).await;
    drop(compiler); // Release the lock before sending the response
    Ok(warp::reply::json(&result))
}

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

async fn build_web_app(static_dir: &str) -> anyhow::Result<()> {
    // Build the web app
    let output = tokio::process::Command::new("trunk")
        .args(&[
            "build",
            "--release",
            "--minify",
            "--config",
            "web/Trunk.toml",
        ])
        .arg("--dist")
        .arg(static_dir)
        .stdout(std::process::Stdio::inherit())
        .output()
        .await?;

    if !output.status.success() {
        log::error!(
            "Error building web app: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        anyhow::bail!("Failed to build the web app");
    }

    log::info!("Web app built successfully.");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let config = config::ServerConfig::parse(CONFIG_PATH)?;

    let dir = match fs::canonicalize(&config.static_dir).await {
        Ok(path) => path,
        Err(_) => {
            // If the static directory does not exist, create it
            fs::create_dir_all(&config.static_dir).await?;
            fs::canonicalize(&config.static_dir).await?
        }
    };

    if !dir.join("index.html").exists() {
        // missing the static directory or index.html
        // rebuild the web app
        log::info!("index.html not found. Rebuilding the web app...");
        build_web_app(&config.static_dir).await?;
    }

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

    // Logger middleware
    let logger = warp::any().map(warp::reply).with(warp::log("server"));

    // Serve static files from a directory
    let static_files = warp::fs::dir(dir);
    let index = warp::path::end().and(index_file);

    // Combine API routes
    let api = warp::path("api").and(compile_route.or(result_route));

    // Combine all routes
    let routes = index.or(static_files).or(api).or(logger);

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

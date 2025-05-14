# Raspberry Pi Pico 2 Simulator – Manual

This project is a web-based simulator for the Raspberry Pi Pico 2, designed as part of my thesis work. It allows users to simulate GPIO interactions, write embedded code, and observe its behavior directly in the browser. The simulator is implemented using Rust and WebAssembly for the frontend, with a Rust backend server that simulates the embedded environment.

Official repository: https://github.com/tmokenc/pico2

The application is hosted at: **160.251.41.78:8080** and will be available at least until **31.08.2025**.

## Using Docker (Recommended)

The primary way to run this application is through Docker with `docker-compose`, as it depends on a specific version of the GCC compiler for proper program compilation. Instructions for installing Docker and Docker Compose can be found online.
https://docs.docker.com/get-docker/

Once installed, navigate to the root of the project source code and run:

```
$ docker compose up
```

This process takes approximately 30 minutes on the first run to build and prepare the environment. After that, the server will be up and ready for use.

## Running Locally (Manual Setup)

Note: Tested only on Ubuntu 22.04.

### Web App

First, install the Rust toolchain, as the project is written in Rust:

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Next, install the `trunk` tool for building the WebAssembly frontend:

```
$ cargo install trunk
```

Add the WebAssembly target to Rust:

```
$ rustup target add wasm32-unknown-unknown
```

Navigate to the `web/` directory and build the frontend:

```
$ cd web
$ trunk build --release
```

The output will be located in the `dist/` directory. Note that the app still has all its functionality except for flashing source code into the MCU, which requires the backend server.

### Backend Server

First, extract the GCC toolchain located at `resources/corev-openhw-gcc/corev-openhw-gcc.tar.gz`, and add its `bin` directory to your `PATH`. If the `resources` directory is not present, execute the script `download_resource.sh` to fetch it first.

```
$ cd resources/corev-openhw-gcc
$ tar -xf corev-openhw-gcc.tar.gz --strip-components=1
$ export PATH=$PATH:$(pwd)/bin
```

Then clone the official Pico SDK:

```
$ git clone \
    -b "2.0.0" \
    -c advice.detachedHead=false \
    https://github.com/raspberrypi/pico-sdk.git \
    resources/pico-sdk
$ git -C resources/pico-sdk submodule update --init --depth=1
```

After preparing the environment, configure the server by setting environment variables or using a configuration file (see Configuration section). Then start the server:

```
$ export SERVER_PORT=8080
$ export SERVER_IP=localhost
$ export SERVER_STATIC_DIR=./web/dist
$ export SERVER_DATA_DIR=./data
$ export SERVER_PICO_SDK=./resources/pico-sdk
$ export RUST_LOG=info

$ cargo run --release --bin server
```

The server will now be available at http://localhost:8080

# Configuration

The server supports five main configuration options that control its behavior:

- port — Port number for the server to listen on (default: 8080)
- ip — IP address the server binds to (default: 127.0.0.1)
- static_dir — Path to the web application directory, which should contain an index.html file (default: ./web/dist)
- data_dir — Path for runtime data storage and project files (default: ./data)
- pico_sdk — Path to the Pico SDK cloned from GitHub (default: ./resources/pico-sdk)

## Configuration Methods

These settings can be provided in two ways:

1. Environment Variables:
   Prefix each variable with SERVER_ and use uppercase for names.
   For example:

```
SERVER_IP="0.0.0.0"
SERVER_PORT=80
SERVER_STATIC_DIR="./web/dist"
SERVER_DATA_DIR="/tmp/server_data"
SERVER_PICO_SDK="./resources/pico-sdk"
```

Additionally, the RUST_LOG environment variable may be used to control logging verbosity.

2. Configuration File config.toml:
   A file named config.toml can be placed in the root directory. Example:

```
port = 8081
ip = "0.0.0.0"
static_dir = "./web/dist"
data_dir = "/tmp/server_data"
pico_sdk = "./resources/pico-sdk"
```

It is also possible to mix both methods. In such cases, environment variables take precedence over values specified in the config.toml file.

Defaults:
If a value is not provided via environment variables or the configuration file, the server will fall back to its default values.

# syntax=docker/dockerfile:1

ARG PYTHON_VERSION=3.12
FROM python:${PYTHON_VERSION}-bookworm

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        cmake \
        ninja-build \
        gcc-arm-none-eabi \
        libnewlib-arm-none-eabi \
        libstdc++-arm-none-eabi-newlib \
    ; \
    rm -rf /var/lib/apt/lists/*

RUN pip install pycryptodome

WORKDIR /opt/corev-openhw-gcc
COPY resources/corev-openhw-gcc/corev-openhw-gcc.tar.gz /opt/corev-openhw-gcc/corev-openhw-gcc.tar.gz

RUN set -eux; \
    tar -xf corev-openhw-gcc.tar.gz --strip-components=1; \
    rm corev-openhw-gcc.tar.gz
ENV PATH=$PATH:/opt/corev-openhw-gcc/bin

ARG PICO_SDK_VERSION=2.0.0
ENV PICO_SDK_VERSION=${PICO_SDK_VERSION}
ENV PICO_SDK_PATH=/opt/pico-sdk
RUN set -eux; \
    git clone -b "${PICO_SDK_VERSION}" -c advice.detachedHead=false "https://github.com/raspberrypi/pico-sdk.git" "${PICO_SDK_PATH}"; \
    git -C "${PICO_SDK_PATH}" submodule update --init --depth=1

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install trunk && \
    rustup target add wasm32-unknown-unknown


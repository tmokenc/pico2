#!/bin/bash

RESOURCE_DIR="./resources"
PICO_SDK_VERSION=2.0.0
PICO_SDK_PATH="${RESOURCE_DIR}/pico-sdk"
COREV_GCC_PATH="${RESOURCE_DIR}/corev-openhw-gcc"


# create resources directory if not exists
if [ ! -d "${RESOURCE_DIR}" ]; then
    mkdir -p "${RESOURCE_DIR}"
fi

if [ ! -d "${COREV_GCC_PATH}" ]; then
    mkdir -p "${COREV_GCC_PATH}"
fi


curl -SL \
    https://buildbot.embecosm.com/job/corev-gcc-ubuntu2204/47/artifact/corev-openhw-gcc-ubuntu2204-20240530.tar.gz \
    --output "${COREV_GCC_PATH}/corev-openhw-gcc.tar.gz"

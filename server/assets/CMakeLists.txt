cmake_minimum_required(VERSION 3.13...3.27)

# initialize the SDK based on PICO_SDK_PATH
# note: this must happen before project()
include(pico_sdk_import.cmake)

project(nrf CXX C ASM)

project(my_project)

# initialize the Raspberry Pi Pico SDK
pico_sdk_init()

# rest of your project

add_executable(main
    main.c
)

# Add pico_stdlib library which aggregates commonly used features
target_link_libraries(
    main 
    pico_stdlib
    hardware_pwm
    hardware_sha256
    hardware_dma
    hardware_spi
    hardware_i2c
    pico_multicore
    pico_sha256
)

# create map/bin/hex/uf2 file in addition to ELF.
pico_add_extra_outputs(main)

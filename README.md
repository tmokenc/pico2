# Pico2
Raspberry Pi Pico 2 simulator in Rust. Currently WIP.

## Literature
- [Raspberry Pi Pico 2 Datasheet](https://datasheets.raspberrypi.com/pico/pico-2-datasheet.pdf)
- [RP2350 Datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf) - The CPU of Raspberry Pi Pico 2
- [pico-sdk](https://github.com/raspberrypi/pico-sdk) - Source code of the SDK for Raspberry Pi Pico
    - [/src/rp2350](https://github.com/raspberrypi/pico-sdk/tree/master/src/rp2350) - Part specific to the Raspberry Pi Pico 2

### RISC-V Hazard3
[Hazard3 documentation](https://github.com/Wren6991/Hazard3/blob/v1.0-rc1/doc/hazard3.pdf)

Instruction set (extensions) as in the section 3.8.1.1. of the Rp2350 datasheet:
- [Unprivileged ISA 20191213](https://github.com/riscv/riscv-isa-manual/releases/download/Ratified-IMAFDQC/riscv-spec-20191213.pdf)
    - `RV32I` v2.1
    - `M` v2.0
    - `A` v2.1
    - `C` v2.0
    - `Zicsr` v2.0
    - `Zifencei` v2.0
- [Bit Manipulation ISA extensions 20210628](https://github.com/riscv/riscv-bitmanip/releases/download/1.0.0/bitmanip-1.0.0-38-g865e7a7.pdf)
    - `Zba` v1.0.0
    - `Zbb` v1.0.0
    - `Zbs` v1.0.0
- [Scalar Cryptography ISA extensions 20220218](https://github.com/riscv/riscv-crypto/releases/download/v1.0.1-scalar/riscv-crypto-spec-scalar-v1.0.1.pdf)
    - `Zbkb` v1.0.1
- [Code Size Reduction extensions frozen v1.0.3-1](https://github.com/riscv/riscv-code-size-reduction/releases/download/v1.0.3-1/Zc-v1.0.3-1.pdf)
    - `Zcb` v1.0.3-1 
    - `Zcmp` v1.0.3-1
- [Privileged Architecture 20211203](https://github.com/riscv/riscv-isa-manual/releases/download/Priv-v1.12/riscv-privileged-20211203.pdf)
    - `Machine ISA` v1.12
- [RISC-V External Debug Support 20190322](https://riscv.org/wp-content/uploads/2019/03/riscv-debug-release.pdf)
    - `Debug` v0.13.2 

### ARM Cortex-M33
TBD

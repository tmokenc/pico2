use crate::common::*;
use crate::gpio::GpioController;
use crate::interrupts::Interrupts;
use std::cell::RefCell;
use std::rc::Rc;

pub mod bootram;
pub mod busctrl;
pub mod clocks;
pub mod sio;
pub mod uart;

pub use bootram::BootRam;
pub use busctrl::BusCtrl;
pub use clocks::Clocks;
pub use sio::Sio;
pub use uart::Uart;

#[derive(Default)]
pub struct Peripherals {
    // APB peripherals
    pub sysinfo: UnimplementedPeripheral,
    pub syscfg: UnimplementedPeripheral,
    pub clocks: Clocks,
    pub psm: UnimplementedPeripheral,
    pub resets: UnimplementedPeripheral,
    pub io_bank0: UnimplementedPeripheral,
    pub io_qspi: UnimplementedPeripheral,
    pub pads_bank0: UnimplementedPeripheral,
    pub pads_qspi: UnimplementedPeripheral,
    pub xosc: UnimplementedPeripheral,
    pub pll_sys: UnimplementedPeripheral,
    pub pll_usb: UnimplementedPeripheral,
    pub accessctrl: UnimplementedPeripheral,
    pub busctrl: BusCtrl,
    pub uart: [Uart; 2],
    pub spi: [UnimplementedPeripheral; 2],
    pub i2c: [UnimplementedPeripheral; 2],
    pub adc: UnimplementedPeripheral,
    pub pwm: UnimplementedPeripheral,
    pub timer: [UnimplementedPeripheral; 2],
    pub hstx_ctrl: UnimplementedPeripheral,
    pub xip_ctrl: UnimplementedPeripheral,
    pub xip_qmi: UnimplementedPeripheral,
    pub watch_dog: UnimplementedPeripheral,
    pub bootram: BootRam, // only allow secure access
    pub rosc: UnimplementedPeripheral,
    pub trng: UnimplementedPeripheral,
    pub sha256: UnimplementedPeripheral,
    pub powman: UnimplementedPeripheral,
    pub ticks: UnimplementedPeripheral,
    pub otp: UnimplementedPeripheral,
    pub otp_data: UnimplementedPeripheral,
    pub otp_data_raw: UnimplementedPeripheral,
    pub otp_data_guarded: UnimplementedPeripheral,
    pub otp_data_raw_guarded: UnimplementedPeripheral,
    pub coresight_periph: UnimplementedPeripheral,
    pub coresight_romtable: UnimplementedPeripheral,
    pub coresight_ahb_ap: [UnimplementedPeripheral; 2],
    pub coresight_timestamp_gen: UnimplementedPeripheral,
    pub coresight_atb_funnel: UnimplementedPeripheral,
    pub coresight_tpiu: UnimplementedPeripheral,
    pub coresight_cti: UnimplementedPeripheral,
    pub coresight_apb_ap_riscv: UnimplementedPeripheral,
    pub glitch_detector: UnimplementedPeripheral,
    pub tbman: UnimplementedPeripheral,

    // AHB peripherals
    pub dma: UnimplementedPeripheral,
    pub usbctrl: UnimplementedPeripheral,
    pub usbctrl_dpram: UnimplementedPeripheral,
    pub usbctrl_regs: UnimplementedPeripheral,
    pub pio: [UnimplementedPeripheral; 3],
    pub xip_aux: UnimplementedPeripheral,
    pub hstx_fifo: UnimplementedPeripheral,
    pub coresight_trace: UnimplementedPeripheral,

    // Core local
    pub sio: Sio,
}

impl Peripherals {
    pub fn new(gpio: Rc<RefCell<GpioController>>, interrupts: Rc<RefCell<Interrupts>>) -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn find_mut(&mut self, address: u32, requestor: Requestor) -> Option<&mut dyn Peripheral> {
        // TODO don't know if this address mask correct or not...
        // All I know for now is that it will not work correctly with
        // the Coresight peripherals, which are not implemented yet.
        //
        // Also missing the Cortex exclusive peripherals.
        let result = match address & 0xFFFF_C000 {
            0x4000_0000 => &mut self.sysinfo as &mut dyn Peripheral,
            0x4000_8000 => &mut self.syscfg as &mut dyn Peripheral,
            0x4001_0000 => &mut self.clocks as &mut dyn Peripheral,
            0x4001_8000 => &mut self.psm as &mut dyn Peripheral,
            0x4002_0000 => &mut self.resets as &mut dyn Peripheral,
            0x4002_8000 => &mut self.io_bank0 as &mut dyn Peripheral,
            0x4003_0000 => &mut self.io_qspi as &mut dyn Peripheral,
            0x4003_8000 => &mut self.pads_bank0 as &mut dyn Peripheral,
            0x4004_0000 => &mut self.pads_qspi as &mut dyn Peripheral,
            0x4004_8000 => &mut self.xosc as &mut dyn Peripheral,
            0x4005_0000 => &mut self.pll_sys as &mut dyn Peripheral,
            0x4005_8000 => &mut self.pll_usb as &mut dyn Peripheral,
            0x4006_0000 => &mut self.accessctrl as &mut dyn Peripheral,
            0x4006_8000 => &mut self.busctrl as &mut dyn Peripheral,
            0x4007_0000 => &mut self.uart[0] as &mut dyn Peripheral,
            0x4007_8000 => &mut self.uart[1] as &mut dyn Peripheral,
            0x4008_0000 => &mut self.spi[0] as &mut dyn Peripheral,
            0x4008_8000 => &mut self.spi[1] as &mut dyn Peripheral,
            0x4009_0000 => &mut self.i2c[0] as &mut dyn Peripheral,
            0x4009_8000 => &mut self.i2c[1] as &mut dyn Peripheral,
            0x400A_0000 => &mut self.adc as &mut dyn Peripheral,
            0x400A_8000 => &mut self.pwm as &mut dyn Peripheral,
            0x400B_0000 => &mut self.timer[0] as &mut dyn Peripheral,
            0x400B_8000 => &mut self.timer[1] as &mut dyn Peripheral,
            0x400C_0000 => &mut self.hstx_ctrl as &mut dyn Peripheral,
            0x400C_8000 => &mut self.xip_ctrl as &mut dyn Peripheral,
            0x400D_0000 => &mut self.xip_qmi as &mut dyn Peripheral,
            0x400D_8000 => &mut self.watch_dog as &mut dyn Peripheral,
            0x400E_0000 => &mut self.bootram as &mut dyn Peripheral,
            0x400E_8000 => &mut self.rosc as &mut dyn Peripheral,
            0x400F_0000 => &mut self.trng as &mut dyn Peripheral,
            0x400F_8000 => &mut self.sha256 as &mut dyn Peripheral,
            0x4010_0000 => &mut self.powman as &mut dyn Peripheral,
            0x4010_8000 => &mut self.ticks as &mut dyn Peripheral,
            0x4012_0000 => &mut self.otp as &mut dyn Peripheral,
            0x4013_0000 => &mut self.otp_data as &mut dyn Peripheral,
            0x4013_4000 => &mut self.otp_data_raw as &mut dyn Peripheral,
            0x4013_8000 => &mut self.otp_data_guarded as &mut dyn Peripheral,
            0x4013_C000 => &mut self.otp_data_raw_guarded as &mut dyn Peripheral,
            0x4014_0000 => &mut self.coresight_periph as &mut dyn Peripheral,
            // 0x4014_0000 => Some(&mut self.coresight_romtable as &mut dyn Peripheral,
            0x4014_2000 => &mut self.coresight_ahb_ap[0] as &mut dyn Peripheral,
            0x4014_4000 => &mut self.coresight_ahb_ap[1] as &mut dyn Peripheral,
            0x4014_6000 => &mut self.coresight_timestamp_gen as &mut dyn Peripheral,
            0x4014_7000 => &mut self.coresight_atb_funnel as &mut dyn Peripheral,
            0x4014_8000 => &mut self.coresight_tpiu as &mut dyn Peripheral,
            0x4014_9000 => &mut self.coresight_cti as &mut dyn Peripheral,
            0x4014_A000 => &mut self.coresight_apb_ap_riscv as &mut dyn Peripheral,
            0x4015_8000 => &mut self.glitch_detector as &mut dyn Peripheral,
            0x4016_0000 => &mut self.tbman as &mut dyn Peripheral,

            // AHB
            0x5000_0000 => &mut self.dma as &mut dyn Peripheral,
            0x5010_0000 => &mut self.usbctrl as &mut dyn Peripheral,
            // 0x5010_0000 => Some(&mut self.usbctrl_dpram as &mut dyn Peripheral,
            0x5011_0000 => &mut self.usbctrl_regs as &mut dyn Peripheral,
            0x5020_0000 => &mut self.pio[0] as &mut dyn Peripheral,
            0x5030_8000 => &mut self.pio[1] as &mut dyn Peripheral,
            0x5040_0000 => &mut self.pio[2] as &mut dyn Peripheral,
            0x5050_0000 => &mut self.xip_aux as &mut dyn Peripheral,
            0x5060_0000 => &mut self.hstx_fifo as &mut dyn Peripheral,
            0x5070_0000 => &mut self.coresight_trace as &mut dyn Peripheral,

            0xd0000000 | 0xd0020000 if requestor.is_proc() => &mut self.sio as &mut dyn Peripheral,
            _ => return None,
        };

        Some(result)
    }

    pub fn find(&self, address: u32, requestor: Requestor) -> Option<&dyn Peripheral> {
        let result = match address & 0xFFFF_C000 {
            0x4000_0000 => &self.sysinfo as &dyn Peripheral,
            0x4000_8000 => &self.syscfg as &dyn Peripheral,
            0x4001_0000 => &self.clocks as &dyn Peripheral,
            0x4001_8000 => &self.psm as &dyn Peripheral,
            0x4002_0000 => &self.resets as &dyn Peripheral,
            0x4002_8000 => &self.io_bank0 as &dyn Peripheral,
            0x4003_0000 => &self.io_qspi as &dyn Peripheral,
            0x4003_8000 => &self.pads_bank0 as &dyn Peripheral,
            0x4004_0000 => &self.pads_qspi as &dyn Peripheral,
            0x4004_8000 => &self.xosc as &dyn Peripheral,
            0x4005_0000 => &self.pll_sys as &dyn Peripheral,
            0x4005_8000 => &self.pll_usb as &dyn Peripheral,
            0x4006_0000 => &self.accessctrl as &dyn Peripheral,
            0x4006_8000 => &self.busctrl as &dyn Peripheral,
            0x4007_0000 => &self.uart[0] as &dyn Peripheral,
            0x4007_8000 => &self.uart[1] as &dyn Peripheral,
            0x4008_0000 => &self.spi[0] as &dyn Peripheral,
            0x4008_8000 => &self.spi[1] as &dyn Peripheral,
            0x4009_0000 => &self.i2c[0] as &dyn Peripheral,
            0x4009_8000 => &self.i2c[1] as &dyn Peripheral,
            0x400A_0000 => &self.adc as &dyn Peripheral,
            0x400A_8000 => &self.pwm as &dyn Peripheral,
            0x400B_0000 => &self.timer[0] as &dyn Peripheral,
            0x400B_8000 => &self.timer[1] as &dyn Peripheral,
            0x400C_0000 => &self.hstx_ctrl as &dyn Peripheral,
            0x400C_8000 => &self.xip_ctrl as &dyn Peripheral,
            0x400D_0000 => &self.xip_qmi as &dyn Peripheral,
            0x400D_8000 => &self.watch_dog as &dyn Peripheral,
            0x400E_0000 => &self.bootram as &dyn Peripheral,
            0x400E_8000 => &self.rosc as &dyn Peripheral,
            0x400F_0000 => &self.trng as &dyn Peripheral,
            0x400F_8000 => &self.sha256 as &dyn Peripheral,
            0x4010_0000 => &self.powman as &dyn Peripheral,
            0x4010_8000 => &self.ticks as &dyn Peripheral,
            0x4012_0000 => &self.otp as &dyn Peripheral,
            0x4013_0000 => &self.otp_data as &dyn Peripheral,
            0x4013_4000 => &self.otp_data_raw as &dyn Peripheral,
            0x4013_8000 => &self.otp_data_guarded as &dyn Peripheral,
            0x4013_C000 => &self.otp_data_raw_guarded as &dyn Peripheral,
            0x4014_0000 => &self.coresight_periph as &dyn Peripheral,
            // 0x4014_0000 => Some(&self.coresight_romtable as &dyn Peripheral,
            0x4014_2000 => &self.coresight_ahb_ap[0] as &dyn Peripheral,
            0x4014_4000 => &self.coresight_ahb_ap[1] as &dyn Peripheral,
            0x4014_6000 => &self.coresight_timestamp_gen as &dyn Peripheral,
            0x4014_7000 => &self.coresight_atb_funnel as &dyn Peripheral,
            0x4014_8000 => &self.coresight_tpiu as &dyn Peripheral,
            0x4014_9000 => &self.coresight_cti as &dyn Peripheral,
            0x4014_A000 => &self.coresight_apb_ap_riscv as &dyn Peripheral,
            0x4015_8000 => &self.glitch_detector as &dyn Peripheral,
            0x4016_0000 => &self.tbman as &dyn Peripheral,

            // AHB
            0x5000_0000 => &self.dma as &dyn Peripheral,
            0x5010_0000 => &self.usbctrl as &dyn Peripheral,
            // 0x5010_0000 => Some(&self.usbctrl_dpram as &dyn Peripheral,
            0x5011_0000 => &self.usbctrl_regs as &dyn Peripheral,
            0x5020_0000 => &self.pio[0] as &dyn Peripheral,
            0x5030_8000 => &self.pio[1] as &dyn Peripheral,
            0x5040_0000 => &self.pio[2] as &dyn Peripheral,
            0x5050_0000 => &self.xip_aux as &dyn Peripheral,
            0x5060_0000 => &self.hstx_fifo as &dyn Peripheral,
            0x5070_0000 => &self.coresight_trace as &dyn Peripheral,

            0xd0000000 | 0xd0020000 if requestor.is_proc() => &self.sio as &dyn Peripheral,
            _ => return None,
        };

        Some(result)
    }
}

#[derive(Debug, PartialEq)]
pub enum PeripheralError {
    OutOfBounds,
    MissingPermission,
}

pub type PeripheralResult<T> = std::result::Result<T, PeripheralError>;

#[derive(Debug, Default, Clone, Copy)]
pub struct PeripheralAccessContext {
    pub secure: bool,
    pub requestor: Requestor,
}

// Purpose: Define the Peripheral trait and a default implementation for unimplemented peripherals.
pub trait Peripheral {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32>;
    fn write_raw(
        &mut self,
        address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()>;

    fn write(
        &mut self,
        address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        let address = address & 0x0000_0FFF; // Address is 12 bits

        // Atomic access (SIO does not has this features)
        match dbg!((address >> 12) & 0xF) {
            // Normal
            0x0 => self.write_raw(address, value, ctx),
            // XOR on write
            0x1 => {
                let current_value = self.read(address, ctx)?;
                let value = current_value ^ value;
                self.write_raw(address, value, ctx)
            }
            // bitmask set on write
            0x2 => {
                let current_value = self.read(address, ctx)?;
                let value = current_value | value;
                self.write_raw(address, value, ctx)
            }
            // bitmask clear on write
            0x3 => {
                let current_value = self.read(address, ctx)?;
                let value = current_value & !value;
                self.write_raw(address, value, ctx)
            }
            _ => Err(PeripheralError::OutOfBounds),
        }
    }
}

#[derive(Default)]
pub struct UnimplementedPeripheral;

impl Peripheral for UnimplementedPeripheral {
    fn read(&self, _address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        log::warn!("Unimplemented peripheral read");
        Ok(0)
    }

    fn write_raw(
        &mut self,
        _address: u16,
        _value: u32,
        _ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        log::warn!("Unimplemented peripheral write");
        Ok(())
    }
}

/**
 * @file peripherals.rs
 * @author Nguyen Le Duy
 * @date 02/01/2025
 * @brief Peripheral module for the RP2350
 */
use crate::clock::Clock;
use crate::gpio::GpioController;
use crate::interrupts::Interrupts;
use crate::{common::*, InspectorRef};
use std::cell::RefCell;
use std::rc::Rc;

pub mod bootram;
pub mod busctrl;
pub mod clocks;
pub mod dma;
pub mod i2c;
pub mod io;
pub mod otp;
pub mod pads;
pub mod pll;
pub mod pwm;
pub mod reset;
pub mod sha256;
pub mod sio;
// pub mod spi;
pub mod ticks;
pub mod timer;
pub mod trng;
pub mod uart;
pub mod watchdog;
pub mod xosc;

pub use bootram::BootRam;
pub use busctrl::BusCtrl;
pub use clocks::Clocks;
pub use dma::Dma;
pub use i2c::I2c;
pub use io::IoBank0;
pub use otp::Otp;
pub use pads::PadsBank0;
pub use pll::Pll;
pub use pwm::Pwm;
pub use reset::Reset;
pub use sha256::Sha256;
pub use sio::Sio;
pub use ticks::Ticks;
pub use timer::Timer;
pub use trng::Trng;
pub use uart::Uart;
pub use watchdog::WatchDog;
pub use xosc::Xosc;

#[derive(Default)]
pub struct Peripherals {
    // APB peripherals
    pub sysinfo: UnimplementedPeripheral,
    pub syscfg: UnimplementedPeripheral,
    pub clocks: Rc<RefCell<Clocks>>,
    pub psm: UnimplementedPeripheral,
    pub resets: Reset,
    pub io_bank0: IoBank0,
    pub io_qspi: UnimplementedPeripheral,
    pub pads_bank0: PadsBank0,
    pub pads_qspi: UnimplementedPeripheral,
    pub xosc: Xosc,
    pub pll_sys: Pll<0>,
    pub pll_usb: Pll<1>,
    pub accessctrl: UnimplementedPeripheral,
    pub busctrl: BusCtrl,
    pub uart0: Rc<RefCell<Uart<0>>>,
    pub uart1: Rc<RefCell<Uart<1>>>,
    pub spi0: UnimplementedPeripheral,
    pub spi1: UnimplementedPeripheral,
    pub i2c0: Rc<RefCell<I2c<0>>>,
    pub i2c1: Rc<RefCell<I2c<1>>>,
    pub adc: UnimplementedPeripheral,
    pub pwm: Rc<RefCell<Pwm>>,
    pub timer0: Rc<RefCell<Timer<0>>>,
    pub timer1: Rc<RefCell<Timer<1>>>,
    pub hstx_ctrl: UnimplementedPeripheral,
    pub xip_ctrl: UnimplementedPeripheral,
    pub xip_qmi: UnimplementedPeripheral,
    pub watch_dog: WatchDog,
    pub bootram: BootRam, // only allow secure access
    pub rosc: UnimplementedPeripheral,
    pub trng: Trng,
    pub sha256: Rc<RefCell<Sha256>>,
    pub powman: UnimplementedPeripheral,
    pub ticks: Ticks,
    pub otp: Otp,
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
    pub dma: Rc<RefCell<Dma>>,
    pub usbctrl: UnimplementedPeripheral,
    pub usbctrl_dpram: UnimplementedPeripheral,
    pub usbctrl_regs: UnimplementedPeripheral,
    pub pio: [UnimplementedPeripheral; 3],
    pub xip_aux: UnimplementedPeripheral,
    pub hstx_fifo: UnimplementedPeripheral,
    pub coresight_trace: UnimplementedPeripheral,

    // Core local
    pub sio: Sio,

    clock: Rc<Clock>,
    interrupts: Rc<RefCell<Interrupts>>,
    gpio: Rc<RefCell<GpioController>>,
    pub(crate) inspector: InspectorRef,
}

impl Peripherals {
    pub fn new(
        gpio: Rc<RefCell<GpioController>>,
        interrupts: Rc<RefCell<Interrupts>>,
        clock: Rc<Clock>,
        inspector: InspectorRef,
    ) -> Self {
        let result = Self {
            gpio,
            interrupts,
            clock,
            inspector,
            ..Default::default()
        };

        timer::start_timer(
            result.timer0.clone(),
            Rc::clone(&result.clock),
            Rc::clone(&result.interrupts),
        );

        timer::start_timer(
            result.timer1.clone(),
            Rc::clone(&result.clock),
            Rc::clone(&result.interrupts),
        );

        sio::timer::start_timer(
            result.sio.timer.clone(),
            Rc::clone(&result.clock),
            Rc::clone(&result.interrupts),
        );

        result
    }

    pub fn get_context(
        &self,
        address: u32,
        requestor: Requestor,
        secure: bool,
    ) -> PeripheralAccessContext {
        PeripheralAccessContext {
            secure,
            requestor,
            address,
            gpio: Rc::clone(&self.gpio),
            interrupts: Rc::clone(&self.interrupts),
            clock: Rc::clone(&self.clock),
            dma: Rc::clone(&self.dma),
            inspector: self.inspector.clone(),
        }
    }

    pub fn reset(&mut self) {
        let Self {
            watch_dog,
            clock,
            gpio,
            interrupts,
            inspector,
            ..
        } = core::mem::take(self);

        self.watch_dog = watch_dog;
        self.clock = clock;
        self.gpio = gpio;
        self.interrupts = interrupts;
        self.inspector = inspector;
        self.watch_dog.reset();

        timer::reschedule_timer_tick(
            self.timer0.clone(),
            self.clock.clone(),
            self.interrupts.clone(),
        );

        timer::reschedule_timer_tick(
            self.timer1.clone(),
            self.clock.clone(),
            self.interrupts.clone(),
        );

        sio::timer::reschedule_timer(
            self.sio.timer.clone(),
            self.clock.clone(),
            self.interrupts.clone(),
        );
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
            0x4007_0000 => &mut self.uart0 as &mut dyn Peripheral,
            0x4007_8000 => &mut self.uart1 as &mut dyn Peripheral,
            0x4008_0000 => &mut self.spi0 as &mut dyn Peripheral,
            0x4008_8000 => &mut self.spi1 as &mut dyn Peripheral,
            0x4009_0000 => &mut self.i2c0 as &mut dyn Peripheral,
            0x4009_8000 => &mut self.i2c1 as &mut dyn Peripheral,
            0x400A_0000 => &mut self.adc as &mut dyn Peripheral,
            0x400A_8000 => &mut self.pwm as &mut dyn Peripheral,
            0x400B_0000 => &mut self.timer0 as &mut dyn Peripheral,
            0x400B_8000 => &mut self.timer1 as &mut dyn Peripheral,
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
            0x4007_0000 => &self.uart0 as &dyn Peripheral,
            0x4007_8000 => &self.uart1 as &dyn Peripheral,
            0x4008_0000 => &self.spi0 as &dyn Peripheral,
            0x4008_8000 => &self.spi1 as &dyn Peripheral,
            0x4009_0000 => &self.i2c0 as &dyn Peripheral,
            0x4009_8000 => &self.i2c1 as &dyn Peripheral,
            0x400A_0000 => &self.adc as &dyn Peripheral,
            0x400A_8000 => &self.pwm as &dyn Peripheral,
            0x400B_0000 => &self.timer0 as &dyn Peripheral,
            0x400B_8000 => &self.timer1 as &dyn Peripheral,
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
    Reserved,
}

pub type PeripheralResult<T> = std::result::Result<T, PeripheralError>;

#[derive(Default, Clone)]
pub struct PeripheralAccessContext {
    pub secure: bool,
    pub requestor: Requestor,
    pub address: u32,
    pub gpio: Rc<RefCell<GpioController>>,
    pub interrupts: Rc<RefCell<Interrupts>>,
    pub clock: Rc<Clock>,
    pub dma: Rc<RefCell<Dma>>,
    pub inspector: InspectorRef,
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
    fn read(&self, _address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        log::warn!(
            "Unimplemented peripheral read at address {:#X}",
            ctx.address
        );
        Ok(0)
    }

    fn write_raw(
        &mut self,
        _address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        log::warn!(
            "Unimplemented peripheral write at address {:#X} with value {:#X}",
            ctx.address,
            value
        );
        Ok(())
    }
}

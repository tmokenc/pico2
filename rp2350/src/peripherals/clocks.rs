//! RP2350 Datasheet Section 8.1

use super::*;
// use std::cell::RefCell;

pub const CLK_GPOUT0_CTRL: u16 = 0x00; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_GPOUT0_DIV: u16 = 0x04; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_GPOUT0_SELECTED: u16 = 0x08; // Indicates which src is currently selected (one-hot)
pub const CLK_GPOUT1_CTRL: u16 = 0x0C; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_GPOUT1_DIV: u16 = 0x10; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_GPOUT1_SELECTED: u16 = 0x14; // Indicates which src is currently selected (one-hot)
pub const CLK_GPOUT2_CTRL: u16 = 0x18; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_GPOUT2_DIV: u16 = 0x1C; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_GPOUT2_SELECTED: u16 = 0x20; // Indicates which src is currently selected (one-hot)
pub const CLK_GPOUT3_CTRL: u16 = 0x24; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_GPOUT3_DIV: u16 = 0x28; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_GPOUT3_SELECTED: u16 = 0x2C; // Indicates which src is currently selected (one-hot)
pub const CLK_REF_CTRL: u16 = 0x30; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_REF_DIV: u16 = 0x34; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_REF_SELECTED: u16 = 0x38; // Indicates which src is currently selected (one-hot)
pub const CLK_SYS_CTRL: u16 = 0x3C; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_SYS_DIV: u16 = 0x40; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_SYS_SELECTED: u16 = 0x44; // Indicates which src is currently selected (one-hot)
pub const CLK_PERI_CTRL: u16 = 0x48; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_PERI_DIV: u16 = 0x4C; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_PERI_SELECTED: u16 = 0x50; // Indicates which src is currently selected (one-hot)
pub const CLK_HSTX_CTRL: u16 = 0x54; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_HSTX_DIV: u16 = 0x58; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_HSTX_SELECTED: u16 = 0x5C; // Indicates which src is currently selected (one-hot)
pub const CLK_USB_CTRL: u16 = 0x60; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_USB_DIV: u16 = 0x64; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_USB_SELECTED: u16 = 0x68; // Indicates which src is currently selected (one-hot)
pub const CLK_ADC_CTRL: u16 = 0x6C; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_ADC_DIV: u16 = 0x70; // Clock control, can be changed on-the-fly (except for auxsrc)
pub const CLK_ADC_SELECTED: u16 = 0x74; // Indicates which src is currently selected (one-hot)
pub const DFTCLK_XOSC_CTRL: u16 = 0x78; //
pub const DFTCLK_ROSC_CTRL: u16 = 0x7C; //
pub const DFTCLK_LPOSC_CTRL: u16 = 0x80; //
pub const CLK_SYS_RESUS_CTRL: u16 = 0x84; //
pub const CLK_SYS_RESUS_STATUS: u16 = 0x88; //
pub const FC0_REF_KHZ: u16 = 0x8C; // Reference clock frequency in kHz
pub const FC0_MIN_KHZ: u16 = 0x90; // Minimum pass frequency in kHz. This is optional. Set to 0 if you are not using the pass/fail flags
pub const FC0_MAX_KHZ: u16 = 0x94; // Maximum pass frequency in kHz. This is optional. Set to 0x1FFFFF if you are not using the pass/fail flags
pub const FC0_DELAY: u16 = 0x98; // Delays the start of frequency counting to allow the mux to settle, Delay is measured in multiples of the reference clock period
pub const FC0_INTERVAL: u16 = 0x9C; // The test interval is 0.98us * 2^interval, but let's call it 1us * 2^interval, The default gives a test interval of 250us
pub const FC0_SRC: u16 = 0xA0; // Clock sent to frequency counter, set to 0 when not required, Writing to this register initiates the frequency count
pub const FC0_STATUS: u16 = 0xA4; // Frequency counter status
pub const FC0_RESULT: u16 = 0xA8; // Result of frequency measurement, only valid when status_done=1
pub const WAKE_EN0: u16 = 0xAC; // enable clock in wake mode
pub const WAKE_EN1: u16 = 0xB0; // enable clock in wake mode
pub const SLEEP_EN0: u16 = 0xB4; // enable clock in sleep mode
pub const SLEEP_EN1: u16 = 0xB8; // enable clock in sleep mode
pub const ENABLED0: u16 = 0xBC; // indicates the state of the clock enable
pub const ENABLED1: u16 = 0xC0; // indicates the state of the clock enable
pub const INTR: u16 = 0xC4; // Raw Interrupts
pub const INTE: u16 = 0xC8; // Interrupt Enable
pub const INTF: u16 = 0xCC; // Interrupt Force
pub const INTS: u16 = 0xD0; // Interrupt status after masking & forcing

pub struct ClockState<const DIV_MASK: u32> {
    ctrl: u32,
    div: u32,
}

impl<const DIV_MASK: u32> Default for ClockState<DIV_MASK> {
    fn default() -> Self {
        Self {
            ctrl: 0,
            div: 1 << 16,
        }
    }
}

impl<const DIV_MASK: u32> ClockState<DIV_MASK> {
    const CLK_ENABLE_MASK: u32 = 1 << 28;
    pub fn is_enabled(&self) -> bool {
        // 28th bit is the enabled
        (self.ctrl & Self::CLK_ENABLE_MASK) != 0
    }

    pub fn enable(&mut self) {
        self.ctrl |= Self::CLK_ENABLE_MASK;
    }

    pub fn disable(&mut self) {
        self.ctrl &= !Self::CLK_ENABLE_MASK;
    }

    pub fn clk_div_int(&self) -> u32 {
        (self.div & DIV_MASK) >> 16
    }

    pub fn clk_div_frac(&self) -> u32 {
        self.div & (DIV_MASK & 0xFFFF)
    }

    fn write_ctrl(&mut self, value: u32) {
        // clear all bits except the 28th
        self.ctrl &= Self::CLK_ENABLE_MASK;
        // set the new value (reserve the 28th bit)
        self.ctrl |= (value & !Self::CLK_ENABLE_MASK);
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ClockMode {
    Wake,
    Sleep,
}

pub struct Clocks {
    pub gp_outs: [ClockState<0xFFFF_FFFF>; 4],
    pub clk_ref: ClockState<{ 0xFF << 16 }>,
    pub clk_sys: ClockState<{ 0xFFFF_FFFF }>,
    pub clk_peri: ClockState<{ 0b11 << 16 }>,
    pub clk_hstx: ClockState<{ 0b11 << 16 }>,
    pub clk_usb: ClockState<{ 0b1111 << 16 }>,
    pub clk_adc: ClockState<{ 0b1111 << 16 }>,

    pub clk_sys_resus_status: bool,
    pub dftclk_xosc_ctrl: u8,
    pub dftclk_rosc_ctrl: u8,
    pub dftclk_losc_ctrl: u8,
    pub clk_sys_resus_ctrl: u32,
    pub fc0_ref_khz: u32,
    pub fc0_min_khz: u32,
    pub fc0_max_khz: u32,
    pub fc0_delay: u8,
    pub fc0_interval: u8,
    pub fc0_src: u8,
    pub fc0_status: u32,
    pub fc0_result: u32,

    mode: ClockMode,
    pub clock_en_wake: [u32; 2],
    pub clock_en_sleep: [u32; 2],

    interrupt_enabled: bool,
    interrupt_force: bool,
    // TODO
}

impl Clocks {
    pub fn change_mode(&mut self, mode: ClockMode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> ClockMode {
        self.mode
    }
}

impl Default for Clocks {
    fn default() -> Self {
        Self {
            gp_outs: Default::default(),
            clk_ref: Default::default(),
            clk_sys: Default::default(),
            clk_peri: Default::default(),
            clk_hstx: Default::default(),
            clk_usb: Default::default(),
            clk_adc: Default::default(),
            dftclk_xosc_ctrl: 0,
            dftclk_rosc_ctrl: 0,
            dftclk_losc_ctrl: 0,
            clk_sys_resus_ctrl: 0xff,
            fc0_ref_khz: 0,
            fc0_min_khz: 0,
            fc0_max_khz: 0x1ff_ffff,
            fc0_delay: 0x1,
            fc0_interval: 0x8,
            fc0_src: 0,
            fc0_status: 0,
            fc0_result: 0,
            mode: ClockMode::Wake,
            clock_en_wake: [0xFFFF_FFFF, !(1 << 31)],
            clock_en_sleep: [0xFFFF_FFFF, !(1 << 31)],
            interrupt_enabled: false,
            interrupt_force: false,
            clk_sys_resus_status: false,
        }
    }
}

impl Peripheral for Clocks {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let value = match address {
            CLK_GPOUT0_CTRL => self.gp_outs[0].ctrl,
            CLK_GPOUT0_DIV => self.gp_outs[0].div,
            CLK_GPOUT1_CTRL => self.gp_outs[1].ctrl,
            CLK_GPOUT1_DIV => self.gp_outs[1].div,
            CLK_GPOUT2_CTRL => self.gp_outs[2].ctrl,
            CLK_GPOUT2_DIV => self.gp_outs[2].div,
            CLK_GPOUT3_CTRL => self.gp_outs[3].ctrl,
            CLK_GPOUT3_DIV => self.gp_outs[3].div,
            CLK_REF_CTRL => self.clk_ref.ctrl,
            CLK_REF_DIV => self.clk_ref.div,
            CLK_REF_SELECTED => 1 << (self.clk_ref.ctrl & 0b11),
            CLK_SYS_CTRL => self.clk_sys.ctrl,
            CLK_SYS_DIV => self.clk_sys.div,
            CLK_SYS_SELECTED => 1 << (self.clk_sys.ctrl & 0b1),
            CLK_PERI_CTRL => self.clk_peri.ctrl,
            CLK_PERI_DIV => self.clk_peri.div,
            CLK_HSTX_CTRL => self.clk_hstx.ctrl,
            CLK_HSTX_DIV => self.clk_hstx.div,
            CLK_USB_CTRL => self.clk_usb.ctrl,
            CLK_USB_DIV => self.clk_usb.div,
            CLK_ADC_CTRL => self.clk_adc.ctrl,
            CLK_ADC_DIV => self.clk_adc.div,

            CLK_GPOUT0_SELECTED  // hardwired to 1
            | CLK_GPOUT1_SELECTED
            | CLK_GPOUT2_SELECTED
            | CLK_GPOUT3_SELECTED
            | CLK_PERI_SELECTED
            | CLK_HSTX_SELECTED
            | CLK_USB_SELECTED
            | CLK_ADC_SELECTED => 0x1,

            DFTCLK_XOSC_CTRL => self.dftclk_xosc_ctrl as u32,
            DFTCLK_ROSC_CTRL => self.dftclk_rosc_ctrl as u32,
            DFTCLK_LPOSC_CTRL => self.dftclk_losc_ctrl as u32,
            CLK_SYS_RESUS_CTRL => self.clk_sys_resus_ctrl,
            CLK_SYS_RESUS_STATUS => self.clk_sys_resus_status as u32,
            FC0_REF_KHZ => self.fc0_ref_khz,
            FC0_MIN_KHZ => self.fc0_min_khz,
            FC0_MAX_KHZ => self.fc0_max_khz,
            FC0_DELAY => self.fc0_delay as u32,
            FC0_INTERVAL => self.fc0_interval as u32,
            FC0_SRC => self.fc0_src as u32,
            FC0_STATUS => self.fc0_status,
            FC0_RESULT => self.fc0_result,
            WAKE_EN0 => self.clock_en_wake[0],
            WAKE_EN1 => self.clock_en_wake[1],
            SLEEP_EN0 => self.clock_en_sleep[0],
            SLEEP_EN1 => self.clock_en_sleep[1],
            ENABLED0 => match self.mode {
                ClockMode::Wake => self.clock_en_wake[0],
                ClockMode::Sleep => self.clock_en_sleep[0],
            },
            ENABLED1 => match self.mode {
                ClockMode::Wake => self.clock_en_wake[1],
                ClockMode::Sleep => self.clock_en_sleep[1],
            },
            INTR => 0, // TODO is this correct?
            INTE => self.interrupt_enabled as u32,
            INTF => self.interrupt_force as u32,
            INTS => 0, // TODO is this correct?

            _ => {
                return Err(PeripheralError::OutOfBounds);
            }
        };

        Ok(value)
    }

    fn write_raw(
        &mut self,
        address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        match address {
            CLK_GPOUT0_CTRL => self.gp_outs[0].write_ctrl(value),
            CLK_GPOUT0_DIV => self.gp_outs[0].div = value,
            CLK_GPOUT1_CTRL => self.gp_outs[1].write_ctrl(value),
            CLK_GPOUT1_DIV => self.gp_outs[1].div = value,
            CLK_GPOUT2_CTRL => self.gp_outs[2].write_ctrl(value),
            CLK_GPOUT2_DIV => self.gp_outs[2].div = value,
            CLK_GPOUT3_CTRL => self.gp_outs[3].write_ctrl(value),
            CLK_GPOUT3_DIV => self.gp_outs[3].div = value,
            CLK_REF_CTRL => self.clk_ref.write_ctrl(value),
            CLK_REF_DIV => self.clk_ref.div = value,
            CLK_SYS_CTRL => self.clk_sys.write_ctrl(value),
            CLK_SYS_DIV => self.clk_sys.div = value,
            CLK_PERI_CTRL => self.clk_peri.write_ctrl(value),
            CLK_PERI_DIV => self.clk_peri.div = value,
            CLK_HSTX_CTRL => self.clk_hstx.write_ctrl(value),
            CLK_HSTX_DIV => self.clk_hstx.div = value,
            CLK_USB_CTRL => self.clk_usb.write_ctrl(value),
            CLK_USB_DIV => self.clk_usb.div = value,
            CLK_ADC_CTRL => self.clk_adc.write_ctrl(value),
            CLK_ADC_DIV => self.clk_adc.div = value,
            DFTCLK_XOSC_CTRL => self.dftclk_xosc_ctrl = value as u8,
            DFTCLK_ROSC_CTRL => self.dftclk_rosc_ctrl = value as u8,
            DFTCLK_LPOSC_CTRL => self.dftclk_losc_ctrl = value as u8,
            FC0_REF_KHZ => self.fc0_ref_khz = value,
            FC0_MIN_KHZ => self.fc0_min_khz = value,
            FC0_MAX_KHZ => self.fc0_max_khz = value,
            FC0_DELAY => self.fc0_delay = value as u8,
            FC0_INTERVAL => self.fc0_interval = value as u8,
            FC0_SRC => self.fc0_src = value as u8,
            WAKE_EN0 => self.clock_en_wake[0] = value,
            WAKE_EN1 => self.clock_en_wake[1] = value,
            SLEEP_EN0 => self.clock_en_sleep[0] = value,
            SLEEP_EN1 => self.clock_en_sleep[1] = value,
            INTE => self.interrupt_enabled = (value & 1) != 0,
            INTF => self.interrupt_force = (value & 1) != 0,

            CLK_REF_SELECTED  // readonly
            | CLK_SYS_SELECTED
            | CLK_PERI_SELECTED
            | CLK_HSTX_SELECTED
            | CLK_USB_SELECTED
            | CLK_ADC_SELECTED
            | CLK_SYS_RESUS_STATUS
            | FC0_STATUS
            | FC0_RESULT
            | ENABLED0
            | ENABLED1
            | INTR
            | INTS
            | CLK_GPOUT0_SELECTED  // hardwired
            | CLK_GPOUT1_SELECTED
            | CLK_GPOUT2_SELECTED
            | CLK_GPOUT3_SELECTED
            | CLK_SYS_RESUS_CTRL => {}

            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}

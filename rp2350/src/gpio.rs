use crate::utils::{extract_bit, extract_bits};
use std::cell::RefCell;
use std::rc::Rc;

pub enum GpioFunction {
    Input,
    Output,
    AltFunction,
}

pub enum DriveStrength {
    _2mA,
    _4mA,
    _8mA,
    _12mA,
}

impl From<u32> for DriveStrength {
    fn from(value: u32) -> Self {
        match value {
            0 => DriveStrength::_2mA,
            1 => DriveStrength::_4mA,
            2 => DriveStrength::_8mA,
            3 => DriveStrength::_12mA,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[rustfmt::skip]
pub enum FunctionSelect {
    #[default]
    None,
    SPI0_RX,
    SPI0_CSn,
    SPI0_SCK,
    SPI0_TX,
    SPI1_RX,
    SPI1_CSn,
    SPI1_SCK,
    SPI1_TX,
    UART0_TX,
    UART0_RX,
    UART0_CTS,
    UART0_RTS,
    UART1_TX,
    UART1_RX,
    UART1_CTS,
    UART1_RTS,
    I2C0_SDA,
    I2C0_SCL,
    I2C1_SDA,
    I2C1_SCL,
    PWM0_A,
    PWM0_B,
    PWM1_A,
    PWM1_B,
    PWM2_A,
    PWM2_B,
    PWM3_A,
    PWM3_B,
    PWM4_A,
    PWM4_B,
    PWM5_A,
    PWM5_B,
    PWM6_A,
    PWM6_B,
    PWM7_A,
    PWM7_B,
    SIO,
    PIO_0,
    PIO_1,
    PIO_2,
    HSTX,
    QMI_CS1n,
    TRACECKL,
    TRACEDATA0,
    TRACEDATA1,
    TRACEDATA2,
    TRACEDATA3,
    CLOCK_GPINO,
    CLOCK_GPOUTO,
    CLOCK_GPIN1,
    CLOCK_GPOUT1,
    CLOCK_GPOUT2,
    CLOCK_GPOUT3,
    USB_OVCUR_DET,
    USB_VBUS_DET,
    USB_VBUS_EN,
}

use FunctionSelect::*;

/// GPIO functions for each pin
/// As in the datasheet for the RP2350 section 1.2.3.
#[rustfmt::skip]
pub const FUNCTION_SELECTS: [[FunctionSelect; 12]; 30] = [
/* GPIO 00 */ [None, SPI0_RX,  UART0_TX,  I2C0_SDA, PWM0_A, SIO, PIO_0, PIO_1, PIO_2, QMI_CS1n,     USB_OVCUR_DET, None],
/* GPIO 01 */ [None, SPI0_CSn, UART0_RX,  I2C0_SCL, PWM0_B, SIO, PIO_0, PIO_1, PIO_2, TRACECKL,     USB_VBUS_DET,  None],
/* GPIO 02 */ [None, SPI0_SCK, UART0_CTS, I2C1_SDA, PWM1_A, SIO, PIO_0, PIO_1, PIO_2, TRACEDATA0,   USB_VBUS_EN,   UART0_TX],
/* GPIO 03 */ [None, SPI0_TX,  UART0_RTS, I2C1_SCL, PWM1_B, SIO, PIO_0, PIO_1, PIO_2, TRACEDATA1,   USB_OVCUR_DET, UART0_RX],
/* GPIO 04 */ [None, SPI0_RX,  UART1_TX,  I2C0_SDA, PWM2_A, SIO, PIO_0, PIO_1, PIO_2, TRACEDATA2,   USB_VBUS_DET,  None],
/* GPIO 05 */ [None, SPI0_CSn, UART1_RX,  I2C0_SCL, PWM2_B, SIO, PIO_0, PIO_1, PIO_2, TRACEDATA3,   USB_VBUS_EN,   None],
/* GPIO 06 */ [None, SPI0_SCK, UART1_CTS, I2C1_SDA, PWM3_A, SIO, PIO_0, PIO_1, PIO_2, None,         USB_OVCUR_DET, UART1_TX],
/* GPIO 07 */ [None, SPI0_TX,  UART1_RTS, I2C1_SCL, PWM3_B, SIO, PIO_0, PIO_1, PIO_2, None,         USB_VBUS_DET,  UART1_RX],
/* GPIO 08 */ [None, SPI1_RX,  UART1_TX,  I2C0_SDA, PWM4_A, SIO, PIO_0, PIO_1, PIO_2, QMI_CS1n,     USB_VBUS_EN,   None],
/* GPIO 09 */ [None, SPI1_CSn, UART1_RX,  I2C0_SCL, PWM4_B, SIO, PIO_0, PIO_1, PIO_2, None,         USB_OVCUR_DET, None],
/* GPIO 10 */ [None, SPI1_SCK, UART1_CTS, I2C1_SDA, PWM5_A, SIO, PIO_0, PIO_1, PIO_2, None,         USB_VBUS_DET,  UART1_TX],
/* GPIO 11 */ [None, SPI1_TX,  UART1_RTS, I2C1_SCL, PWM5_B, SIO, PIO_0, PIO_1, PIO_2, None,         USB_VBUS_EN,   UART1_RX],
/* GPIO 12 */ [HSTX, SPI1_RX,  UART0_TX,  I2C0_SDA, PWM6_A, SIO, PIO_0, PIO_1, PIO_2, CLOCK_GPINO,  USB_OVCUR_DET, None],
/* GPIO 13 */ [HSTX, SPI1_CSn, UART0_RX,  I2C0_SCL, PWM6_B, SIO, PIO_0, PIO_1, PIO_2, CLOCK_GPOUTO, USB_VBUS_DET,  None],
/* GPIO 14 */ [HSTX, SPI1_SCK, UART0_CTS, I2C1_SDA, PWM7_A, SIO, PIO_0, PIO_1, PIO_2, CLOCK_GPIN1,  USB_VBUS_EN,   UART0_TX],
/* GPIO 15 */ [HSTX, SPI1_TX,  UART0_RTS, I2C1_SCL, PWM7_B, SIO, PIO_0, PIO_1, PIO_2, CLOCK_GPOUT1, USB_OVCUR_DET, UART0_RX],
/* GPIO 16 */ [HSTX, SPI0_RX,  UART0_TX,  I2C0_SDA, PWM0_A, SIO, PIO_0, PIO_1, PIO_2, None,         USB_VBUS_DET,  None],
/* GPIO 17 */ [HSTX, SPI0_CSn, UART0_RX,  I2C0_SCL, PWM0_B, SIO, PIO_0, PIO_1, PIO_2, None,         USB_VBUS_EN,   None],
/* GPIO 18 */ [HSTX, SPI0_SCK, UART0_CTS, I2C1_SDA, PWM1_A, SIO, PIO_0, PIO_1, PIO_2, None,         USB_OVCUR_DET, UART0_TX],
/* GPIO 19 */ [HSTX, SPI0_TX,  UART0_RTS, I2C1_SCL, PWM1_B, SIO, PIO_0, PIO_1, PIO_2, QMI_CS1n,     USB_VBUS_DET,  UART0_RX],
/* GPIO 20 */ [None, SPI0_RX,  UART1_TX,  I2C0_SDA, PWM2_A, SIO, PIO_0, PIO_1, PIO_2, CLOCK_GPINO,  USB_VBUS_EN,   None],
/* GPIO 21 */ [None, SPI0_CSn, UART1_RX,  I2C0_SCL, PWM2_B, SIO, PIO_0, PIO_1, PIO_2, CLOCK_GPOUTO, USB_OVCUR_DET, None],
/* GPIO 22 */ [None, SPI0_SCK, UART1_CTS, I2C1_SDA, PWM3_A, SIO, PIO_0, PIO_1, PIO_2, CLOCK_GPIN1,  USB_VBUS_DET,  UART1_TX],
/* GPIO 23 */ [None, SPI0_TX,  UART1_RTS, I2C1_SCL, PWM3_B, SIO, PIO_0, PIO_1, PIO_2, CLOCK_GPOUT1, USB_VBUS_EN,   UART1_RX],
/* GPIO 24 */ [None, SPI1_RX,  UART1_TX,  I2C0_SDA, PWM4_A, SIO, PIO_0, PIO_1, PIO_2, CLOCK_GPOUT2, USB_OVCUR_DET, None],
/* GPIO 25 */ [None, SPI1_CSn, UART1_RX,  I2C0_SCL, PWM4_B, SIO, PIO_0, PIO_1, PIO_2, CLOCK_GPOUT3, USB_VBUS_DET,  None],
/* GPIO 26 */ [None, SPI1_SCK, UART1_CTS, I2C1_SDA, PWM5_A, SIO, PIO_0, PIO_1, PIO_2, None,         USB_VBUS_EN,   UART1_TX],
/* GPIO 27 */ [None, SPI1_TX,  UART1_RTS, I2C1_SCL, PWM5_B, SIO, PIO_0, PIO_1, PIO_2, None,         USB_OVCUR_DET, UART1_RX],
/* GPIO 28 */ [None, SPI1_RX,  UART0_TX,  I2C0_SDA, PWM6_A, SIO, PIO_0, PIO_1, PIO_2, None,         USB_VBUS_DET,  None],
/* GPIO 29 */ [None, SPI1_CSn, UART0_RX,  I2C0_SCL, PWM6_B, SIO, PIO_0, PIO_1, PIO_2, None,         USB_VBUS_EN,   None],
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Override {
    #[default]
    Normal, // drive output from peripheral signal selected by funcsel
    Invert, // drive output from inverse of peripheral signal selected by funcsel
    Low,    // drive output low
    High,   // drive output hight
}

impl From<u32> for Override {
    fn from(value: u32) -> Self {
        match value {
            0 => Override::Normal,
            1 => Override::Invert,
            2 => Override::Low,
            3 => Override::High,
            _ => unreachable!(),
        }
    }
}

impl Override {
    fn to_u32(&self) -> u32 {
        match self {
            Override::Normal => 0,
            Override::Invert => 1,
            Override::Low => 2,
            Override::High => 3,
        }
    }

    fn apply(&self, value: f32) -> f32 {
        match self {
            Override::Normal => value,
            Override::Invert => 3.3 - value,
            Override::Low => 0.0,
            Override::High => 3.3,
        }
    }

    fn is_enabled(&self) -> bool {
        *self == Override::High
    }

    fn is_disabled(&self) -> bool {
        *self == Override::Low
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpioPinValue {
    value: Rc<RefCell<f32>>,
    // value: Rc<RefCell<f32>>,
}

impl From<bool> for GpioPinValue {
    fn from(value: bool) -> Self {
        let value = if value { 3.3 } else { 0.0 };
        GpioPinValue {
            value: Rc::new(RefCell::new(value)),
        }
    }
}

impl From<f32> for GpioPinValue {
    fn from(value: f32) -> Self {
        GpioPinValue {
            value: Rc::new(RefCell::new(value)),
        }
    }
}

impl GpioPinValue {
    pub fn set_value(&mut self, value: f32) {
        *self.value.borrow_mut() = value;
    }

    pub fn get_value(&self) -> f32 {
        *self.value.borrow()
    }

    pub fn set_high(&mut self) {
        self.set_value(3.3);
    }

    pub fn set_low(&mut self) {
        self.set_value(0.0);
    }

    pub fn is_high(&self) -> bool {
        self.get_value() > 1.5
    }

    pub fn is_low(&self) -> bool {
        self.get_value() < 1.5
    }
}

pub struct GpioController {
    pub pins: [GpioPin; 30],
}

impl Default for GpioController {
    fn default() -> Self {
        let pins = (0usize..30)
            .map(|index| GpioPin::new(index))
            .collect::<Vec<GpioPin>>()
            .try_into()
            .unwrap();

        GpioController { pins }
    }
}

impl GpioController {
    pub fn get_pin(&self, index: u8) -> &GpioPin {
        &self.pins[index as usize]
    }

    pub fn get_pin_mut(&mut self, index: u8) -> &mut GpioPin {
        &mut self.pins[index as usize]
    }

    pub fn select(&mut self, func: FunctionSelect) -> Option<&mut GpioPin> {
        self.pins.iter_mut().find(|v| v.func_sel() == func)
    }
}

#[derive(Debug, Default, Clone)]
pub struct GpioPin {
    ctrl: u32,
    pad: u32,
    index: usize,
    pub value: GpioPinValue,
}

impl GpioPin {
    fn new(index: usize) -> Self {
        Self {
            index,
            ctrl: 0,
            pad: 0,
            value: GpioPinValue::default(),
        }
    }

    pub fn write_ctrl(&mut self, value: u32) {
        self.ctrl = value;
    }

    pub fn ctrl(&self) -> u32 {
        self.ctrl
    }

    pub fn write_pad(&mut self, value: u32) {
        self.pad = value;
    }

    pub fn pad(&self) -> u32 {
        self.pad
    }

    pub fn func_sel(&self) -> FunctionSelect {
        let index = self.ctrl & 0b1111;
        FUNCTION_SELECTS[self.index][index as usize]
    }

    pub fn out_override(&self) -> Override {
        Override::from(extract_bits(self.ctrl, 12..=13))
    }

    pub fn in_override(&self) -> Override {
        Override::from(extract_bits(self.ctrl, 16..=17))
    }

    pub fn oe_override(&self) -> Override {
        Override::from(extract_bits(self.ctrl, 14..=15))
    }

    pub fn irq_override(&self) -> Override {
        Override::from(extract_bits(self.ctrl, 28..=29))
    }

    // from pad
    // 1 == Fast
    pub fn slew_rate(&self) -> bool {
        extract_bit(self.pad, 0) == 1
    }

    // Has priority over output enable from peripherals
    pub fn output_disable(&self) -> bool {
        extract_bit(self.pad, 7) == 1
    }

    pub fn drive_strength(&self) -> DriveStrength {
        DriveStrength::from(extract_bits(self.pad, 4..=5))
    }

    pub fn input_enable(&self) -> bool {
        extract_bit(self.ctrl, 6) == 1
    }

    pub fn pad_isolation_control(&self) -> bool {
        extract_bit(self.pad, 8) == 1
    }

    pub fn schmitt(&self) -> bool {
        extract_bit(self.pad, 1) == 1
    }

    pub fn pull_up_enable(&self) -> bool {
        extract_bit(self.pad, 3) == 1
    }

    pub fn pull_down_enable(&self) -> bool {
        extract_bit(self.pad, 2) == 1
    }

    pub fn status(&self) -> u32 {
        // after override is applied
        let irq_to_proc = 0;
        let in_from_pad = 0;
        let oe_to_pad = 0;
        let out_to_pad = 0;

        (out_to_pad << 9) | (oe_to_pad << 13) | (in_from_pad << 17) | (irq_to_proc << 26)
    }
}

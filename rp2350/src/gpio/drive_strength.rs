/**
 * @file gpio/drive_strength.rs
 * @author Nguyen Le Duy
 * @date 14/04/2025
 * @brief Drive strength module for the RP2350
 */

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

impl From<DriveStrength> for u32 {
    fn from(value: DriveStrength) -> Self {
        match value {
            DriveStrength::_2mA => 0,
            DriveStrength::_4mA => 1,
            DriveStrength::_8mA => 2,
            DriveStrength::_12mA => 3,
        }
    }
}

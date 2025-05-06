/**
 * @file override.rs
 * @author Nguyen Le Duy
 * @date 14/04/2025
 * @brief Definition of the GPIO override
 */

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
    pub fn to_u32(&self) -> u32 {
        match self {
            Override::Normal => 0,
            Override::Invert => 1,
            Override::Low => 2,
            Override::High => 3,
        }
    }

    pub fn apply(&self, value: f32) -> f32 {
        match self {
            Override::Normal => value,
            Override::Invert => 3.3 - value,
            Override::Low => 0.0,
            Override::High => 3.3,
        }
    }

    pub fn apply_bool(&self, value: bool) -> bool {
        match self {
            Override::Normal => value,
            Override::Invert => !value,
            Override::Low => false,
            Override::High => true,
        }
    }

    pub fn is_enabled(&self) -> bool {
        *self == Override::High
    }

    pub fn is_disabled(&self) -> bool {
        *self == Override::Low
    }
}

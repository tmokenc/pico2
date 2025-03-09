use super::*;

#[derive(Debug, Default, Clone, Copy)]
pub enum VoltageSelect {
    V1_8,
    #[default]
    V3_3,
}

pub const VOLTAGE_SELECT = 0x00;
pub const GPIO0 = 0x04;
pub const GPIO47 = 0xc0;
pub const SWCLK = 0xc4;
pub const SWD = 0xc8;

#[derive(Debug, Default)]
pub struct PadBank0 {
    voltage_select: VoltageSelect,
}

impl Peripheral for PadBank0 {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        match address {
            0x00 => Ok(self.voltage_select as u32),
            _ => Err(PeripheralError::OutOfBounds),
        }
    }

    fn write_raw(
        &mut self,
        address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        match address {
            0x00 => {
                self.voltage_select = match value & 1 {
                    0 => VoltageSelect::V3_3,
                    1 => VoltageSelect::V1_8,
                    _ => unreachable!(),
                };
            }
            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}

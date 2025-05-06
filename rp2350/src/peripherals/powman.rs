use super::*;
pub struct Powman {
    // TODO
}

impl Default for Powman {
    fn default() -> Self {
        Self {}
    }
}

impl Peripheral for Powman {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        log::warn!(
            "Unimplemented peripheral read at address {:#X}",
            ctx.address
        );
        Ok(0)
    }

    fn write_raw(
        &mut self,
        address: u16,
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

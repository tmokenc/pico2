#[derive(Debug, Clone)]
pub struct ExecutedInstruction {
    pub instruction: u32,
    pub address: u32,
    pub name: String,
    pub operands: Vec<u32>,
}

pub trait Inspector {
    fn trng_generated(&self, value: u32) {
        log::info!("TRNG generated value: {:#X}", value);
    }

    fn executed_instruction(&self, core: u8, value: ExecutedInstruction) {
        log::info!(
            "Core {} executed instruction {:#X} at address {:#X} - {} {:?}",
            core,
            value.instruction,
            value.address,
            value.name,
            value.operands
        );
    }

    fn tick(&self, core: u8) {
        log::trace!("Ticking core {}", core);
    }

    fn wake_core(&self, core: u8) {
        log::info!("Waking core {}", core);
    }
}

pub struct DummyInspector;

impl Inspector for DummyInspector {}

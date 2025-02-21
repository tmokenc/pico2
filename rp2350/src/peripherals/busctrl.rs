use super::*;

// Priority
const PRIORITY_PROC0: u32 = 1 << 0;
const PRIORITY_PROC1: u32 = 1 << 4;
const PRIORITY_DMA_R: u32 = 1 << 8;
const PRIORITY_DMA_W: u32 = 1 << 12;

#[derive(Default, Clone, Copy)]
enum PerformanceEventType {
    // count cycles where any master stalled for any reason
    StallUpstream = 0,
    // count cycles where any master stalled due to a stall on the downstream bus
    StallDownstream = 1,
    // An event took place that previouslsy stalled
    AccessContested = 2,
    // An event took place
    #[default]
    Access = 3,
}

#[derive(Default, Clone, Copy)]
enum PerformanceEventSource {
    #[default]
    SiobProc1,
    SiobProc0,
    Apb,
    Fastperi,
    Sram9,
    Sram8,
    Sram7,
    Sram6,
    Sram5,
    Sram4,
    Sram3,
    Sram2,
    Sram1,
    Sram0,
    XipMain0,
    XipMain1,
    Rom,
    Reserved,
}

#[derive(Default, Clone, Copy)]
struct PerformanceEvent {
    source: PerformanceEventSource,
    event: PerformanceEventType,
}

impl From<PerformanceEvent> for u32 {
    fn from(perfsel: PerformanceEvent) -> u32 {
        let source = perfsel.source as u8 as u32;
        let event = perfsel.event as u8 as u32;
        event + (source << 2)
    }
}

impl From<u32> for PerformanceEvent {
    fn from(value: u32) -> PerformanceEvent {
        let source = match value >> 2 & 0b11111 {
            0 => PerformanceEventSource::SiobProc1,
            1 => PerformanceEventSource::SiobProc0,
            2 => PerformanceEventSource::Apb,
            3 => PerformanceEventSource::Fastperi,
            4 => PerformanceEventSource::Sram9,
            5 => PerformanceEventSource::Sram8,
            6 => PerformanceEventSource::Sram7,
            7 => PerformanceEventSource::Sram6,
            8 => PerformanceEventSource::Sram5,
            9 => PerformanceEventSource::Sram4,
            10 => PerformanceEventSource::Sram3,
            11 => PerformanceEventSource::Sram2,
            12 => PerformanceEventSource::Sram1,
            13 => PerformanceEventSource::Sram0,
            14 => PerformanceEventSource::XipMain0,
            15 => PerformanceEventSource::XipMain1,
            16 => PerformanceEventSource::Rom,
            _ => PerformanceEventSource::Reserved,
        };

        let event = match value & 0b11 {
            0 => PerformanceEventType::StallUpstream,
            1 => PerformanceEventType::StallDownstream,
            2 => PerformanceEventType::AccessContested,
            3 => PerformanceEventType::Access,
            _ => unreachable!(),
        };
        PerformanceEvent { source, event }
    }
}

#[derive(Default)]
pub struct BusCtrl {
    priority: u32,
    perfctr_en: bool,
    perfctr: [u32; 4],
    perfsel: [PerformanceEvent; 4],
}

impl Peripheral for BusCtrl {
    fn read(&self, addr: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        match addr & 0xFFF {
            0x00 => Ok(self.priority as u32),
            0x04 => Ok(1), // Priority Ack, rarely be 0
            0x08 => Ok(self.perfctr_en as u32),
            0x0C => Ok(self.perfctr[0]),
            0x10 => Ok(self.perfsel[0].into()),
            0x14 => Ok(self.perfctr[1]),
            0x18 => Ok(self.perfsel[1].into()),
            0x1C => Ok(self.perfctr[2]),
            0x20 => Ok(self.perfsel[2].into()),
            0x24 => Ok(self.perfctr[3]),
            0x28 => Ok(self.perfsel[3].into()),
            _ => Err(PeripheralError::OutOfBounds),
        }
    }

    fn write_raw(
        &mut self,
        addr: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        match addr & 0xFFF {
            0x00 => self.priority = value,
            0x04 => (), /* Priority Ack, ignore */
            0x08 => self.perfctr_en = value & 1 == 1,
            0x0C => self.perfctr[0] = value,
            0x10 => self.perfsel[0] = value.into(),
            0x14 => self.perfctr[1] = value,
            0x18 => self.perfsel[1] = value.into(),
            0x1C => self.perfctr[2] = value,
            0x20 => self.perfsel[2] = value.into(),
            0x24 => self.perfctr[3] = value,
            0x28 => self.perfsel[3] = value.into(),
            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}

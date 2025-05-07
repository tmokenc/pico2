/**
 * @file peripherals/busctrl.rs
 * @author Nguyen Le Duy
 * @date 06/02/2025
 * @brief BusCtrl peripheral implementation
 */
use super::*;
use crate::bus::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
// default to 0x1f => SRAM6 Access
pub enum PerformanceEventType {
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
// default to 0x1f => SRAM6 Access
enum PerformanceEventSource {
    SiobProc1,
    SiobProc0,
    Apb,
    Fastperi,
    Sram9,
    Sram8,
    Sram7,
    #[default]
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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

// Map address to the performance event source
impl PerformanceEventSource {
    fn from_address(address: u32, master: Requestor) -> PerformanceEventSource {
        let base_address = address & 0xF000_0000;
        match base_address {
            Bus::ROM => PerformanceEventSource::Rom,
            Bus::SIO => match master {
                Requestor::Proc0 => PerformanceEventSource::SiobProc0,
                Requestor::Proc1 => PerformanceEventSource::SiobProc1,
                _ => PerformanceEventSource::Reserved,
            },
            Bus::ABP => PerformanceEventSource::Apb,
            Bus::AHB => PerformanceEventSource::Fastperi,
            Bus::SRAM => match (address, (address >> 2) & 0b11) {
                (0x2000_0000..=0x2003_FFFF, 0) => PerformanceEventSource::Sram0,
                (0x2000_0000..=0x2003_FFFF, 1) => PerformanceEventSource::Sram1,
                (0x2000_0000..=0x2003_FFFF, 2) => PerformanceEventSource::Sram2,
                (0x2000_0000..=0x2003_FFFF, 3) => PerformanceEventSource::Sram3,
                (0x2004_0000..=0x2007_FFFF, 0) => PerformanceEventSource::Sram4,
                (0x2004_0000..=0x2007_FFFF, 1) => PerformanceEventSource::Sram5,
                (0x2004_0000..=0x2007_FFFF, 2) => PerformanceEventSource::Sram6,
                (0x2004_0000..=0x2007_FFFF, 3) => PerformanceEventSource::Sram7,
                (0x2008_0000, _) => PerformanceEventSource::Sram8,
                (0x2008_1000, _) => PerformanceEventSource::Sram9,
                _ => PerformanceEventSource::Reserved,
            },
            Bus::XIP => match master {
                Requestor::Proc0 => PerformanceEventSource::XipMain0,
                Requestor::Proc1 => PerformanceEventSource::XipMain1,
                _ => PerformanceEventSource::Reserved,
            },
            _ => PerformanceEventSource::Reserved,
        }
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

impl BusCtrl {
    // Priority
    pub const PRIORITY_PROC0: u32 = 1 << 0;
    pub const PRIORITY_PROC1: u32 = 1 << 4;
    pub const PRIORITY_DMA_R: u32 = 1 << 8;
    pub const PRIORITY_DMA_W: u32 = 1 << 12;

    pub fn has_priority(&self, master: Requestor) -> bool {
        match master {
            Requestor::Proc0 => self.priority & Self::PRIORITY_PROC0 != 0,
            Requestor::Proc1 => self.priority & Self::PRIORITY_PROC1 != 0,
            Requestor::DmaR => self.priority & Self::PRIORITY_DMA_R != 0,
            Requestor::DmaW => self.priority & Self::PRIORITY_DMA_W != 0,
        }
    }

    pub fn count(&mut self, address: u32, master: Requestor, event: PerformanceEventType) {
        // Disable by default
        if !self.perfctr_en {
            return;
        }

        let event = PerformanceEvent {
            source: PerformanceEventSource::from_address(address, master),
            event,
        };

        for i in 0..4 {
            if self.perfsel[i] == event {
                // Saturation to 24 bits
                if self.perfctr[i] < 0x00FF_FFFF {
                    self.perfctr[i] += 1;
                }
            }
        }
    }
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
            0x0C => self.perfctr[0] = 0, // Reset counter
            0x10 => self.perfsel[0] = value.into(),
            0x14 => self.perfctr[1] = 0, // Reset counter
            0x18 => self.perfsel[1] = value.into(),
            0x1C => self.perfctr[2] = 0, // Reset counter
            0x20 => self.perfsel[2] = value.into(),
            0x24 => self.perfctr[3] = 0, // Reset counter
            0x28 => self.perfsel[3] = value.into(),
            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    #![allow(unused_variables)]
    // allow unchecked result
    #![allow(unused_must_use)]
    use super::*;
    use crate::bus::*;
    use crate::common::*;
    use crate::peripherals::*;

    #[test]
    fn priority() {
        let mut ctrl = BusCtrl::default();

        // Default
        assert!(!ctrl.has_priority(Requestor::Proc0));
        assert!(!ctrl.has_priority(Requestor::Proc1));
        assert!(!ctrl.has_priority(Requestor::DmaR));
        assert!(!ctrl.has_priority(Requestor::DmaW));

        // Set priority
        ctrl.write(
            0,
            BusCtrl::PRIORITY_PROC0 | BusCtrl::PRIORITY_DMA_W,
            &Default::default(),
        );

        assert!(ctrl.has_priority(Requestor::Proc0));
        assert!(!ctrl.has_priority(Requestor::Proc1));
        assert!(!ctrl.has_priority(Requestor::DmaR));
        assert!(ctrl.has_priority(Requestor::DmaW));

        // Set all to one
        ctrl.write(0, u32::MAX, &Default::default());
        assert!(ctrl.has_priority(Requestor::Proc0));
        assert!(ctrl.has_priority(Requestor::Proc1));
        assert!(ctrl.has_priority(Requestor::DmaR));
        assert!(ctrl.has_priority(Requestor::DmaW));
    }

    #[test]
    fn counter() {
        let mut ctrl = BusCtrl::default();

        // default
        assert_eq!(ctrl.read(0x08, &Default::default()), Ok(0)); // perfctr_en
        assert_eq!(ctrl.read(0x10, &Default::default()), Ok(0x1f)); // default to 0x1f
        assert_eq!(ctrl.read(0x18, &Default::default()), Ok(0x1f));
        assert_eq!(ctrl.read(0x20, &Default::default()), Ok(0x1f));
        assert_eq!(ctrl.read(0x28, &Default::default()), Ok(0x1f));

        ctrl.write(0x10, 0x43, &Default::default()); // Rom Access

        ctrl.count(Bus::SRAM, Requestor::Proc0, PerformanceEventType::Access);
        ctrl.count(
            Bus::SIO,
            Requestor::Proc1,
            PerformanceEventType::StallUpstream,
        );
        ctrl.count(Bus::ROM, Requestor::Proc0, PerformanceEventType::Access);
        ctrl.count(Bus::ROM, Requestor::Proc1, PerformanceEventType::Access);
        ctrl.count(
            Bus::ROM,
            Requestor::Proc1,
            PerformanceEventType::AccessContested,
        );
        ctrl.count(
            Bus::ROM,
            Requestor::DmaR,
            PerformanceEventType::StallUpstream,
        );

        // counter is disabled by default
        assert_eq!(ctrl.read(0x08, &Default::default()), Ok(0));
        assert_eq!(ctrl.read(0x0C, &Default::default()), Ok(0));
        assert_eq!(ctrl.read(0x14, &Default::default()), Ok(0));
        assert_eq!(ctrl.read(0x1C, &Default::default()), Ok(0));
        assert_eq!(ctrl.read(0x24, &Default::default()), Ok(0));

        // Enable counter
        ctrl.write(0x08, 1, &Default::default());
        assert_eq!(ctrl.read(0x08, &Default::default()), Ok(1));

        ctrl.count(
            0x2004_0000 + 0b1010, // Sram6
            Requestor::Proc0,
            PerformanceEventType::Access,
        );
        ctrl.count(
            Bus::SIO,
            Requestor::Proc1,
            PerformanceEventType::StallUpstream,
        );
        ctrl.count(Bus::ROM, Requestor::Proc0, PerformanceEventType::Access);
        ctrl.count(Bus::ROM, Requestor::Proc1, PerformanceEventType::Access);
        ctrl.count(
            Bus::ROM,
            Requestor::Proc1,
            PerformanceEventType::AccessContested,
        );
        ctrl.count(
            Bus::ROM,
            Requestor::DmaR,
            PerformanceEventType::StallUpstream,
        );

        // Rom access was 2
        assert_eq!(ctrl.read(0x0C, &Default::default()), Ok(2));
        // Sram6 access for the rest and was 1
        assert_eq!(ctrl.read(0x14, &Default::default()), Ok(1));
        assert_eq!(ctrl.read(0x1C, &Default::default()), Ok(1));
        assert_eq!(ctrl.read(0x24, &Default::default()), Ok(1));
    }
}

use crate::clock::Clock;
use crate::common::*;
use crate::gpio::GpioController;
use crate::interrupts::Interrupts;
use crate::memory::*;
use crate::peripherals::*;
use crate::utils::*;
use std::cell::RefCell;
use std::rc::Rc;

// TODO - counter

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusError {
    BusFault,
    ConcurrentAccess,
    LoadError,
    StoreError,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// Requestor for the bus (Core0, Core1, DMA)
/// This is used to identify the source of the request
/// As well as how it should be handled
pub struct BusAccessContext {
    pub secure: bool,
    pub requestor: Requestor,
    pub size: DataSize,
    pub signed: bool,
    pub exclusive: bool,
    pub architecture: ArchitectureType,
}

type BusResult<T> = Result<T, BusError>;

impl From<crate::memory::MemoryOutOfBoundsError> for BusError {
    fn from(_e: crate::memory::MemoryOutOfBoundsError) -> Self {
        BusError::BusFault
    }
}

/// Status of a load transaction
/// this will be wrapped in a RC<RefCell<>> to allow for mutable access to the status
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoadStatus {
    #[default]
    Waiting,
    Done(u32),
    ExclusiveDone(u32), // exclusive access
    Error(BusError),
}

/// Status of a store transaction
/// this will be wrapped in a RC<RefCell<>> to allow for mutable access to the status
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoreStatus {
    #[default]
    Waiting,
    Done,
    ExclusiveDone, // exclusive access
    Error(BusError),
}

enum StatusType {
    Load(Rc<RefCell<LoadStatus>>),
    Store(u32, Rc<RefCell<StoreStatus>>),
}

struct Status {
    address: u32,
    wait_cycles: u8,
    ctx: BusAccessContext,
    status: StatusType,
}

pub struct Bus {
    pub sram: GenericMemory<{ 520 * KB }>,
    pub rom: GenericMemory<{ 32 * KB }>,
    // pub xip: GenericMemory<{ 64 * KB }>,
    pub flash: GenericMemory<{ 4 * MB }>,

    pub peripherals: Peripherals,

    // Internal states
    dma_access: Option<Status>,
    core0_access: Option<Status>,
    core1_access: Option<Status>,

    core0_exclusive: Option<u32>, // address
    core1_exclusive: Option<u32>, // address
    dma_exclusive: Option<u32>,   // ?? not sure if this is needed, added just in case
}

impl Default for Bus {
    fn default() -> Self {
        let mut res = Self {
            sram: GenericMemory::default(),
            rom: GenericMemory::default(),
            flash: GenericMemory::default(),
            peripherals: Peripherals::default(),
            dma_access: None,
            core0_access: None,
            core1_access: None,
            core0_exclusive: None,
            core1_exclusive: None,
            dma_exclusive: None,
        };

        res.set_rom(*include_bytes!("../bootrom-combined.bin"));
        res
    }
}

impl Bus {
    // Address Map
    pub const ROM: u32 = 0x0000_0000;
    pub const XIP: u32 = 0x1000_0000;
    pub const SRAM: u32 = 0x2000_0000;
    pub const ABP: u32 = 0x4000_0000;
    pub const AHB: u32 = 0x5000_0000;
    pub const SIO: u32 = 0xd000_0000;
    pub const CORTEX_M33_PRIVATE_REGISTERS: u32 = 0xe0000000;

    pub fn new(
        gpio: Rc<RefCell<GpioController>>,
        interrupts: Rc<RefCell<Interrupts>>,
        clock: Rc<RefCell<Clock>>,
    ) -> Self {
        Self {
            peripherals: Peripherals::new(gpio, interrupts, clock),
            ..Default::default()
        }
    }

    pub fn set_rom(&mut self, data: [u8; 32 * KB]) {
        self.rom = GenericMemory::new(&data);
    }

    pub fn set_sram(&mut self, data: [u8; 520 * KB]) {
        self.sram = GenericMemory::new(&data);
    }

    pub fn tick(&mut self) {
        let mut core0_access = self.core0_access.take();
        let mut core1_access = self.core1_access.take();
        let mut dma_access = self.dma_access.take();

        self.update_status(&mut core0_access);
        self.update_status(&mut core1_access);
        self.update_status(&mut dma_access);
        self.core0_access = core0_access;
        self.core1_access = core1_access;
        self.dma_access = dma_access;
    }

    fn update_status(&mut self, target_status: &mut Option<Status>) {
        let Some(mut status) = target_status.take() else {
            return;
        };

        if status.wait_cycles > 1 {
            status.wait_cycles -= 1;
            *target_status = Some(status);
            return;
        }

        match status.status {
            StatusType::Load(load_status) => {
                let result = match status.ctx.size {
                    DataSize::Byte => self.read_u8(status.address, status.ctx).map(|v| {
                        if status.ctx.signed {
                            sign_extend(v as u32, 7)
                        } else {
                            v as u32
                        }
                    }),
                    DataSize::HalfWord => self.read_u16(status.address, status.ctx).map(|v| {
                        if status.ctx.signed {
                            sign_extend(v as u32, 15)
                        } else {
                            v as u32
                        }
                    }),

                    DataSize::Word => self.read_u32(status.address, status.ctx),
                };

                *load_status.borrow_mut() = match result {
                    Ok(v) if status.ctx.exclusive => LoadStatus::ExclusiveDone(v),
                    Ok(v) => LoadStatus::Done(v),
                    Err(BusError::ConcurrentAccess) => LoadStatus::Waiting,
                    Err(_e) => LoadStatus::Error(BusError::LoadError),
                };
            }

            StatusType::Store(value, store_status) => {
                let result = match status.ctx.size {
                    DataSize::Byte => self.write_u8(status.address, value, status.ctx),
                    DataSize::HalfWord => self.write_u16(status.address, value, status.ctx),
                    DataSize::Word => self.write_u32(status.address, value, status.ctx),
                };
                *store_status.borrow_mut() = match result {
                    Ok(_) if status.ctx.exclusive => StoreStatus::ExclusiveDone,
                    Ok(_) => StoreStatus::Done,
                    Err(BusError::ConcurrentAccess) => StoreStatus::Waiting,
                    Err(_e) => StoreStatus::Error(BusError::StoreError),
                };
            }
        }
    }

    pub fn fetch(&mut self, address: u32) -> BusResult<u32> {
        let base_address = address & 0xF000_0000;

        if (base_address != Self::ROM)
            && (base_address != Self::SRAM)
            && (base_address != Self::XIP)
        {
            return Err(BusError::BusFault);
        }

        let result = match address & 0xF000_0000 {
            Self::ROM => self.rom.read_u32(address),
            Self::SRAM => self.sram.read_u32(address - Self::SRAM),
            Self::XIP => self.flash.read_u32(address - Self::XIP),
            _ => return Err(BusError::BusFault),
        };

        result.map_err(|_| BusError::BusFault)
    }

    /// Call by a load instruction
    pub fn load(
        &mut self,
        address: u32,
        ctx: BusAccessContext,
    ) -> BusResult<Rc<RefCell<LoadStatus>>> {
        // TODO: counter

        // check for address correctness
        if !self.is_valid_address(address, &ctx) {
            return Err(BusError::BusFault);
        }

        let load_status = Rc::new(RefCell::new(LoadStatus::Waiting));

        let status = Status {
            ctx,
            address,
            wait_cycles: self.address_cycle(address).0,
            status: StatusType::Load(Rc::clone(&load_status)),
        };

        match ctx.requestor {
            Requestor::Proc0 => self.core0_access = Some(status),
            Requestor::Proc1 => self.core1_access = Some(status),
            Requestor::DmaR | Requestor::DmaW => self.dma_access = Some(status),
        }

        Ok(load_status)
    }

    /// Call by a store instruction
    pub fn store(
        &mut self,
        address: u32,
        value: u32,
        ctx: BusAccessContext,
    ) -> BusResult<Rc<RefCell<StoreStatus>>> {
        // TODO: counter

        // check for address correctness
        if !self.is_valid_address(address, &ctx) {
            return Err(BusError::BusFault);
        }

        let store_status = Rc::new(RefCell::new(StoreStatus::Waiting));

        let status = Status {
            ctx,
            address,
            wait_cycles: self.address_cycle(address).1,
            status: StatusType::Store(value, Rc::clone(&store_status)),
        };

        match ctx.requestor {
            Requestor::Proc0 => self.core0_access = Some(status),
            Requestor::Proc1 => self.core1_access = Some(status),
            Requestor::DmaR | Requestor::DmaW => self.dma_access = Some(status),
        }

        Ok(store_status)
    }

    /// Cycle required for read and write access
    fn address_cycle(&self, address: u32) -> (u8, u8) {
        match address & 0xF000_0000 {
            Self::ROM | Self::SRAM | Self::SIO | Self::XIP => (1, 1),
            _ => (3, 4),
        }
    }

    fn is_valid_address(&self, address: u32, ctx: &BusAccessContext) -> bool {
        // Rough address decode is first performed on bits 31:28 of the address
        let base_address = address & 0xF000_0000;

        if ctx.exclusive && base_address != Self::SRAM {
            // Exclusive access is only allowed for SRAM
            return false;
        }

        match address & 0xF000_0000 {
            Self::ROM | Self::SRAM | Self::XIP => true,
            _ => self.peripherals.find(address, ctx.requestor).is_some(),
        }
    }

    // Check if the address is free for access
    // This hold the assumption that the exclusive access
    // is done in the sequence of read -> modify -> write
    fn is_address_free(&self, address: u32, ctx: &BusAccessContext) -> bool {
        match ctx.requestor {
            Requestor::Proc0 => self
                .core1_exclusive
                .filter(|addr| *addr == address)
                .or_else(|| self.dma_exclusive)
                .filter(|addr| *addr == address)
                .is_none(),
            Requestor::Proc1 => self
                .core0_exclusive
                .filter(|addr| *addr == address)
                .or_else(|| self.dma_exclusive)
                .filter(|addr| *addr == address)
                .is_none(),
            Requestor::DmaR | Requestor::DmaW => self
                .core0_exclusive
                .filter(|addr| *addr == address)
                .or_else(|| self.core1_exclusive)
                .filter(|addr| *addr == address)
                .is_none(),
        }
    }

    fn read_u32(&mut self, address: u32, ctx: BusAccessContext) -> BusResult<u32> {
        if !self.is_address_free(address, &ctx) {
            return Err(BusError::ConcurrentAccess);
        }

        // Exclusive read will lock the address for that requestor
        if ctx.exclusive {
            match ctx.requestor {
                Requestor::Proc0 => self.core0_exclusive = Some(address),
                Requestor::Proc1 => self.core1_exclusive = Some(address),
                Requestor::DmaR | Requestor::DmaW => self.dma_exclusive = Some(address),
            }
        }

        match address & 0xF000_0000 {
            Self::ROM => Ok(self.rom.read_u32(address)?),
            Self::SRAM => Ok(self.sram.read_u32(address - Self::SRAM)?),
            Self::XIP => Ok(self.flash.read_u32(address - Self::XIP)?),
            _ => {
                let peri_ctx = PeripheralAccessContext {
                    secure: ctx.secure,
                    requestor: ctx.requestor,
                };

                self.peripherals
                    .find(address, ctx.requestor)
                    .ok_or(BusError::BusFault)?
                    .read(address as u16, &peri_ctx)
                    .map_err(|_| BusError::BusFault)
            }
        }
    }

    fn write_u32(&mut self, address: u32, value: u32, ctx: BusAccessContext) -> BusResult<()> {
        if !self.is_address_free(address, &ctx) {
            return Err(BusError::ConcurrentAccess);
        }

        // Exclusive write will unlock the address of that requestor
        // normal write will not unlock the address even if exclusive is set for that address
        if ctx.exclusive {
            match ctx.requestor {
                Requestor::Proc0 => self.core0_exclusive = None,
                Requestor::Proc1 => self.core1_exclusive = None,
                Requestor::DmaR | Requestor::DmaW => self.dma_exclusive = None,
            }
        }

        match address & 0xF000_0000 {
            Self::ROM => (),
            Self::SRAM => self.sram.write_u32(address - Self::SRAM, value)?,
            Self::XIP => self.flash.write_u32(address - Self::XIP, value)?,
            _ => {
                let peri_ctx = PeripheralAccessContext {
                    secure: ctx.secure,
                    requestor: ctx.requestor,
                };

                self.peripherals
                    .find_mut(address, ctx.requestor)
                    .ok_or(BusError::BusFault)?
                    .write(address as u16, value, &peri_ctx)
                    .map_err(|_| BusError::BusFault)?
            }
        }

        Ok(())
    }

    fn read_u16(&mut self, address: u32, ctx: BusAccessContext) -> BusResult<u16> {
        match address & 0xF000_0000 {
            Self::ROM => Ok(self.rom.read_u16(address)?),
            Self::SRAM => Ok(self.sram.read_u16(address - Self::SRAM)?),
            Self::XIP => Ok(self.flash.read_u16(address - Self::XIP)?),
            _ => {
                let value = self.read_u32(address & !0b11, ctx)?;
                if (address & 0b11) == 0 {
                    Ok(value as u16)
                } else {
                    Ok((value >> 16) as u16)
                }
            }
        }
    }

    fn write_u16(&mut self, address: u32, value: u32, ctx: BusAccessContext) -> BusResult<()> {
        match address & 0xF000_0000 {
            Self::ROM => (),
            Self::SRAM => self.sram.write_u16(address - Self::SRAM, value as u16)?,
            Self::XIP => self.flash.write_u16(address - Self::XIP, value as u16)?,
            _ => {
                let value = if (address & 0b11) == 0 {
                    value & 0x0000_FFFF
                } else {
                    (value as u32) << 16
                };

                self.write_u32(address & !0b11, value, ctx)?
            }
        }

        Ok(())
    }

    fn read_u8(&mut self, address: u32, ctx: BusAccessContext) -> BusResult<u8> {
        match address & 0xF000_0000 {
            Self::ROM => Ok(self.rom.read_u8(address)?),
            Self::SRAM => Ok(self.sram.read_u8(address - Self::SRAM)?),
            Self::XIP => Ok(self.flash.read_u8(address - Self::XIP)?),
            _ => {
                let value = self.read_u32(address & !0b11, ctx)?;
                let index = address as usize & 0b11;
                Ok(value.to_le_bytes()[index])
            }
        }
    }

    fn write_u8(&mut self, address: u32, value: u32, ctx: BusAccessContext) -> BusResult<()> {
        match address & 0xF000_0000 {
            Self::ROM => (),
            Self::SRAM => self.sram.write_u8(address - Self::SRAM, value as u8)?,
            Self::XIP => self.flash.write_u8(address - Self::XIP, value as u8)?,
            _ => {
                let value = value & 0xFF;
                let value = match address & 0b11 {
                    0 => value << 0,
                    1 => value << 8,
                    2 => value << 16,
                    3 => value << 24,
                    _ => unreachable!(),
                };

                self.write_u32(address & !0b11, value, ctx)?
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! setup {
        ($bus:ident) => {
            let mut $bus = Bus::new(
                Rc::new(RefCell::new(GpioController::default())),
                Rc::new(RefCell::new(Interrupts::default())),
                Rc::new(RefCell::new(Clock::new(150_000_000))),
            );
        };
    }

    #[test]
    fn fetch() {
        setup!(bus);
        let address = Bus::SRAM;
        let value = 0x1234_5678;
        bus.write_u32(address, value, Default::default()).unwrap();

        assert_eq!(bus.fetch(address), Ok(value));
    }

    #[test]
    fn fetch_error() {
        setup!(bus);
        let address = 0x4000_0000;
        assert_eq!(bus.fetch(address), Err(BusError::BusFault));
    }

    #[test]
    fn load() {
        setup!(bus);
        let address = Bus::SRAM;
        let value = 0x1234_5678;
        bus.write_u32(address, value, Default::default()).unwrap();

        let status = bus.load(address, Default::default()).unwrap();
        assert_eq!(*status.borrow(), LoadStatus::Waiting);

        bus.tick();
        assert_eq!(*status.borrow(), LoadStatus::Done(value));
    }
}

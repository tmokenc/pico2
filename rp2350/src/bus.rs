use crate::common::*;
use crate::memory::*;
use crate::peripherals::*;
use crate::utils::*;
use num_derive::FromPrimitive;
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

#[derive(Default)]
pub struct Bus {
    pub sram: GenericMemory<{ 520 * KB }>,
    rom: GenericMemory<{ 32 * KB }>,
    xip: GenericMemory<{ 64 * MB }>,

    sio: Sio,

    // APB peripherals
    sysinfo: UnimplementedPeripheral,
    syscfg: UnimplementedPeripheral,
    clocks: Clocks,
    psm: UnimplementedPeripheral,
    resets: UnimplementedPeripheral,
    io_bank0: UnimplementedPeripheral,
    io_qspi: UnimplementedPeripheral,
    pads_bank0: UnimplementedPeripheral,
    pads_qspi: UnimplementedPeripheral,
    xosc: UnimplementedPeripheral,
    pll_sys: UnimplementedPeripheral,
    pll_usb: UnimplementedPeripheral,
    accessctrl: UnimplementedPeripheral,
    busctrl: BusCtrl,
    uart: [Uart; 2],
    spi: [UnimplementedPeripheral; 2],
    i2c: [UnimplementedPeripheral; 2],
    adc: UnimplementedPeripheral,
    pwm: UnimplementedPeripheral,
    timer: [UnimplementedPeripheral; 2],
    hstx_ctrl: UnimplementedPeripheral,
    xip_ctrl: UnimplementedPeripheral,
    xip_qmi: UnimplementedPeripheral,
    watch_dog: UnimplementedPeripheral,
    bootram: BootRam, // only allow secure access
    rosc: UnimplementedPeripheral,
    trng: UnimplementedPeripheral,
    sha256: UnimplementedPeripheral,
    powman: UnimplementedPeripheral,
    ticks: UnimplementedPeripheral,
    otp: UnimplementedPeripheral,
    otp_data: UnimplementedPeripheral,
    otp_data_raw: UnimplementedPeripheral,
    otp_data_guarded: UnimplementedPeripheral,
    otp_data_raw_guarded: UnimplementedPeripheral,
    coresight_periph: UnimplementedPeripheral,
    coresight_romtable: UnimplementedPeripheral,
    coresight_ahb_ap: [UnimplementedPeripheral; 2],
    coresight_timestamp_gen: UnimplementedPeripheral,
    coresight_atb_funnel: UnimplementedPeripheral,
    coresight_tpiu: UnimplementedPeripheral,
    coresight_cti: UnimplementedPeripheral,
    coresight_apb_ap_riscv: UnimplementedPeripheral,
    glitch_detector: UnimplementedPeripheral,
    tbman: UnimplementedPeripheral,

    // AHB peripherals
    dma: UnimplementedPeripheral,
    usbctrl: UnimplementedPeripheral,
    usbctrl_dpram: UnimplementedPeripheral,
    usbctrl_regs: UnimplementedPeripheral,
    pio: [UnimplementedPeripheral; 3],
    xip_aux: UnimplementedPeripheral,
    hstx_fifo: UnimplementedPeripheral,
    coresight_trace: UnimplementedPeripheral,

    // Internal states
    dma_access: Option<Status>,
    core0_access: Option<Status>,
    core1_access: Option<Status>,

    core0_exclusive: Option<u32>, // address
    core1_exclusive: Option<u32>, // address
    dma_exclusive: Option<u32>,   // ?? not sure if this is needed, added just in case
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

    pub fn new() -> Self {
        let mut result = Self::default();
        result.set_rom(*include_bytes!("../bootrom-combined.bin"));
        result
    }

    pub fn set_rom(&mut self, data: [u8; 32 * KB]) {
        self.rom = GenericMemory::new(data);
    }

    pub fn set_sram(&mut self, data: [u8; 520 * KB]) {
        self.sram = GenericMemory::new(data);
    }

    pub fn tick(&mut self) {
        if let Some(status) = self.dma_access.take() {
            self.dma_access = self.update_status(status);
        }

        if let Some(status) = self.core0_access.take() {
            self.core0_access = self.update_status(status);
        }

        if let Some(status) = self.core1_access.take() {
            self.core1_access = self.update_status(status);
        }
    }

    fn update_status(&mut self, mut status: Status) -> Option<Status> {
        if status.wait_cycles > 1 {
            status.wait_cycles -= 1;
            return Some(status);
        }

        match status.status {
            StatusType::Load(load_status) => {
                let result = match status.ctx.size {
                    DataSize::Byte => self.read_u8(status.address, &status.ctx).map(|v| {
                        if status.ctx.signed {
                            sign_extend(v as u32, 7)
                        } else {
                            v as u32
                        }
                    }),
                    DataSize::HalfWord => self.read_u16(status.address, &status.ctx).map(|v| {
                        if status.ctx.signed {
                            sign_extend(v as u32, 15)
                        } else {
                            v as u32
                        }
                    }),
                    DataSize::Word => self.read_u32(status.address, &status.ctx),
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
                    DataSize::Byte => self.write_u8(status.address, value, &status.ctx),
                    DataSize::HalfWord => self.write_u16(status.address, value, &status.ctx),
                    DataSize::Word => self.write_u32(status.address, value, &status.ctx),
                };
                *store_status.borrow_mut() = match result {
                    Ok(_) if status.ctx.exclusive => StoreStatus::ExclusiveDone,
                    Ok(_) => StoreStatus::Done,
                    Err(BusError::ConcurrentAccess) => StoreStatus::Waiting,
                    Err(_e) => StoreStatus::Error(BusError::StoreError),
                };
            }
        }

        None
    }

    pub fn fetch(&mut self, address: u32) -> BusResult<u32> {
        let base_address = address & 0xF000_0000;

        if (base_address != Self::ROM)
            && (base_address != Self::SRAM)
            && (base_address != Self::XIP)
        {
            return Err(BusError::BusFault);
        }

        self.read_u32(address, &Default::default())
    }

    /// Call by a load instruction
    pub fn load(
        &mut self,
        address: u32,
        ctx: BusAccessContext,
    ) -> BusResult<Rc<RefCell<LoadStatus>>> {
        // TODO: counter

        // check for address correctness
        self.find_peripheral(address, &ctx)?;

        let load_status = Rc::new(RefCell::new(LoadStatus::Waiting));

        let status = Status {
            address,
            wait_cycles: self.address_cycle(address).0,
            ctx: ctx,
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
        self.find_peripheral(address, &ctx)?;
        let store_status = Rc::new(RefCell::new(StoreStatus::Waiting));

        let status = Status {
            address,
            wait_cycles: self.address_cycle(address).1,
            ctx: ctx,
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

    fn find_peripheral(
        &self,
        address: u32,
        ctx: &BusAccessContext,
    ) -> BusResult<Option<&dyn Peripheral>> {
        // Rough address decode is first performed on bits 31:28 of the address
        let base_address = address & 0xF000_0000;

        if ctx.exclusive && base_address != Self::SRAM {
            // Exclusive access is only allowed for SRAM
            return Err(BusError::BusFault);
        }

        let result: &dyn Peripheral = match base_address {
            // base address
            Self::ROM | Self::SRAM | Self::XIP => return Ok(None),
            Self::ABP => match address & 0xFFFF_F000 {
                0x4000_0000 => &self.sysinfo as &dyn Peripheral,
                0x4000_8000 => &self.syscfg as &dyn Peripheral,
                0x4001_0000 => &self.clocks as &dyn Peripheral,
                0x4001_8000 => &self.psm as &dyn Peripheral,
                0x4002_0000 => &self.resets as &dyn Peripheral,
                0x4002_8000 => &self.io_bank0 as &dyn Peripheral,
                0x4003_0000 => &self.io_qspi as &dyn Peripheral,
                0x4003_8000 => &self.pads_bank0 as &dyn Peripheral,
                0x4004_0000 => &self.pads_qspi as &dyn Peripheral,
                0x4004_8000 => &self.xosc as &dyn Peripheral,
                0x4005_0000 => &self.pll_sys as &dyn Peripheral,
                0x4005_8000 => &self.pll_usb as &dyn Peripheral,
                0x4006_0000 => &self.accessctrl as &dyn Peripheral,
                0x4006_8000 => &self.busctrl as &dyn Peripheral,
                0x4007_0000 => &self.uart[0] as &dyn Peripheral,
                0x4007_8000 => &self.uart[1] as &dyn Peripheral,
                0x4008_0000 => &self.spi[0] as &dyn Peripheral,
                0x4008_8000 => &self.spi[1] as &dyn Peripheral,
                0x4009_0000 => &self.i2c[0] as &dyn Peripheral,
                0x4009_8000 => &self.i2c[1] as &dyn Peripheral,
                0x400A_0000 => &self.adc as &dyn Peripheral,
                0x400A_8000 => &self.pwm as &dyn Peripheral,
                0x400B_0000 => &self.timer[0] as &dyn Peripheral,
                0x400B_8000 => &self.timer[1] as &dyn Peripheral,
                0x400C_0000 => &self.hstx_ctrl as &dyn Peripheral,
                0x400C_8000 => &self.xip_ctrl as &dyn Peripheral,
                0x400D_0000 => &self.xip_qmi as &dyn Peripheral,
                0x400D_8000 => &self.watch_dog as &dyn Peripheral,
                0x400E_0000 => &self.bootram as &dyn Peripheral,
                0x400E_8000 => &self.rosc as &dyn Peripheral,
                0x400F_0000 => &self.trng as &dyn Peripheral,
                0x400F_8000 => &self.sha256 as &dyn Peripheral,
                0x4010_0000 => &self.powman as &dyn Peripheral,
                0x4010_8000 => &self.ticks as &dyn Peripheral,
                0x4012_0000 => &self.otp as &dyn Peripheral,
                0x4013_0000 => &self.otp_data as &dyn Peripheral,
                0x4013_4000 => &self.otp_data_raw as &dyn Peripheral,
                0x4013_8000 => &self.otp_data_guarded as &dyn Peripheral,
                0x4013_C000 => &self.otp_data_raw_guarded as &dyn Peripheral,
                0x4014_0000 => &self.coresight_periph as &dyn Peripheral,
                // 0x4014_0000 => Some(&self.coresight_romtable as &dyn Peripheral,
                0x4014_2000 => &self.coresight_ahb_ap[0] as &dyn Peripheral,
                0x4014_4000 => &self.coresight_ahb_ap[1] as &dyn Peripheral,
                0x4014_6000 => &self.coresight_timestamp_gen as &dyn Peripheral,
                0x4014_7000 => &self.coresight_atb_funnel as &dyn Peripheral,
                0x4014_8000 => &self.coresight_tpiu as &dyn Peripheral,
                0x4014_9000 => &self.coresight_cti as &dyn Peripheral,
                0x4014_A000 => &self.coresight_apb_ap_riscv as &dyn Peripheral,
                0x4015_8000 => &self.glitch_detector as &dyn Peripheral,
                0x4016_0000 => &self.tbman as &dyn Peripheral,
                _ => return Err(BusError::BusFault),
            },

            Self::AHB => match address & 0xFFFF_F000 {
                0x5000_0000 => &self.dma as &dyn Peripheral,
                0x5010_0000 => &self.usbctrl as &dyn Peripheral,
                // 0x5010_0000 => Some(&self.usbctrl_dpram as &dyn Peripheral,
                0x5011_0000 => &self.usbctrl_regs as &dyn Peripheral,
                0x5020_0000 => &self.pio[0] as &dyn Peripheral,
                0x5030_8000 => &self.pio[1] as &dyn Peripheral,
                0x5040_0000 => &self.pio[2] as &dyn Peripheral,
                0x5050_0000 => &self.xip_aux as &dyn Peripheral,
                0x5060_0000 => &self.hstx_fifo as &dyn Peripheral,
                0x5070_0000 => &self.coresight_trace as &dyn Peripheral,
                _ => return Err(BusError::BusFault),
            },

            Self::SIO => match ctx.requestor {
                Requestor::Proc0 | Requestor::Proc1 => &self.sio as &dyn Peripheral,
                _ => return Err(BusError::BusFault),
            },

            Self::CORTEX_M33_PRIVATE_REGISTERS => match ctx.architecture {
                ArchitectureType::Hazard3 => return Err(BusError::BusFault),
                ArchitectureType::CortexM33 => todo!(),
            },

            _ => return Err(BusError::BusFault),
        };

        Ok(Some(result))
    }

    // Version for mutable access
    fn find_peripheral_mut(
        &mut self,
        address: u32,
        ctx: &BusAccessContext,
    ) -> BusResult<Option<&mut dyn Peripheral>> {
        // Rough address decode is first performed on bits 31:28 of the address
        let result: &mut dyn Peripheral = match address & 0xF000_0000 {
            // base address
            Self::ROM | Self::SRAM | Self::XIP => return Ok(None),
            Self::ABP => match address & 0xFFFF_F000 {
                0x4000_0000 => &mut self.sysinfo as &mut dyn Peripheral,
                0x4000_8000 => &mut self.syscfg as &mut dyn Peripheral,
                0x4001_0000 => &mut self.clocks as &mut dyn Peripheral,
                0x4001_8000 => &mut self.psm as &mut dyn Peripheral,
                0x4002_0000 => &mut self.resets as &mut dyn Peripheral,
                0x4002_8000 => &mut self.io_bank0 as &mut dyn Peripheral,
                0x4003_0000 => &mut self.io_qspi as &mut dyn Peripheral,
                0x4003_8000 => &mut self.pads_bank0 as &mut dyn Peripheral,
                0x4004_0000 => &mut self.pads_qspi as &mut dyn Peripheral,
                0x4004_8000 => &mut self.xosc as &mut dyn Peripheral,
                0x4005_0000 => &mut self.pll_sys as &mut dyn Peripheral,
                0x4005_8000 => &mut self.pll_usb as &mut dyn Peripheral,
                0x4006_0000 => &mut self.accessctrl as &mut dyn Peripheral,
                0x4006_8000 => &mut self.busctrl as &mut dyn Peripheral,
                0x4007_0000 => &mut self.uart[0] as &mut dyn Peripheral,
                0x4007_8000 => &mut self.uart[1] as &mut dyn Peripheral,
                0x4008_0000 => &mut self.spi[0] as &mut dyn Peripheral,
                0x4008_8000 => &mut self.spi[1] as &mut dyn Peripheral,
                0x4009_0000 => &mut self.i2c[0] as &mut dyn Peripheral,
                0x4009_8000 => &mut self.i2c[1] as &mut dyn Peripheral,
                0x400A_0000 => &mut self.adc as &mut dyn Peripheral,
                0x400A_8000 => &mut self.pwm as &mut dyn Peripheral,
                0x400B_0000 => &mut self.timer[0] as &mut dyn Peripheral,
                0x400B_8000 => &mut self.timer[1] as &mut dyn Peripheral,
                0x400C_0000 => &mut self.hstx_ctrl as &mut dyn Peripheral,
                0x400C_8000 => &mut self.xip_ctrl as &mut dyn Peripheral,
                0x400D_0000 => &mut self.xip_qmi as &mut dyn Peripheral,
                0x400D_8000 => &mut self.watch_dog as &mut dyn Peripheral,
                0x400E_0000 => &mut self.bootram as &mut dyn Peripheral,
                0x400E_8000 => &mut self.rosc as &mut dyn Peripheral,
                0x400F_0000 => &mut self.trng as &mut dyn Peripheral,
                0x400F_8000 => &mut self.sha256 as &mut dyn Peripheral,
                0x4010_0000 => &mut self.powman as &mut dyn Peripheral,
                0x4010_8000 => &mut self.ticks as &mut dyn Peripheral,
                0x4012_0000 => &mut self.otp as &mut dyn Peripheral,
                0x4013_0000 => &mut self.otp_data as &mut dyn Peripheral,
                0x4013_4000 => &mut self.otp_data_raw as &mut dyn Peripheral,
                0x4013_8000 => &mut self.otp_data_guarded as &mut dyn Peripheral,
                0x4013_C000 => &mut self.otp_data_raw_guarded as &mut dyn Peripheral,
                0x4014_0000 => &mut self.coresight_periph as &mut dyn Peripheral,
                // 0x4014_0000 => Some(&mut self.coresight_romtable as &mut dyn Peripheral,
                0x4014_2000 => &mut self.coresight_ahb_ap[0] as &mut dyn Peripheral,
                0x4014_4000 => &mut self.coresight_ahb_ap[1] as &mut dyn Peripheral,
                0x4014_6000 => &mut self.coresight_timestamp_gen as &mut dyn Peripheral,
                0x4014_7000 => &mut self.coresight_atb_funnel as &mut dyn Peripheral,
                0x4014_8000 => &mut self.coresight_tpiu as &mut dyn Peripheral,
                0x4014_9000 => &mut self.coresight_cti as &mut dyn Peripheral,
                0x4014_A000 => &mut self.coresight_apb_ap_riscv as &mut dyn Peripheral,
                0x4015_8000 => &mut self.glitch_detector as &mut dyn Peripheral,
                0x4016_0000 => &mut self.tbman as &mut dyn Peripheral,
                _ => return Err(BusError::BusFault),
            },

            Self::AHB => match address & 0xFFFF_F000 {
                0x5000_0000 => &mut self.dma as &mut dyn Peripheral,
                0x5010_0000 => &mut self.usbctrl as &mut dyn Peripheral,
                // 0x5010_0000 => Some(&mut self.usbctrl_dpram as &mut dyn Peripheral,
                0x5011_0000 => &mut self.usbctrl_regs as &mut dyn Peripheral,
                0x5020_0000 => &mut self.pio[0] as &mut dyn Peripheral,
                0x5030_8000 => &mut self.pio[1] as &mut dyn Peripheral,
                0x5040_0000 => &mut self.pio[2] as &mut dyn Peripheral,
                0x5050_0000 => &mut self.xip_aux as &mut dyn Peripheral,
                0x5060_0000 => &mut self.hstx_fifo as &mut dyn Peripheral,
                0x5070_0000 => &mut self.coresight_trace as &mut dyn Peripheral,
                _ => return Err(BusError::BusFault),
            },

            Self::SIO => match ctx.requestor {
                Requestor::Proc0 | Requestor::Proc1 => &mut self.sio as &mut dyn Peripheral,
                _ => return Err(BusError::BusFault),
            },
            Self::CORTEX_M33_PRIVATE_REGISTERS => match ctx.architecture {
                ArchitectureType::Hazard3 => return Err(BusError::BusFault),
                ArchitectureType::CortexM33 => todo!(),
            },
            _ => return Err(BusError::BusFault),
        };

        Ok(Some(result))
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

    fn read_u32(&mut self, address: u32, ctx: &BusAccessContext) -> BusResult<u32> {
        if !self.is_address_free(address, ctx) {
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
            Self::XIP => Ok(self.xip.read_u32(address - Self::XIP)?),
            _ => {
                let mut peri_ctx = PeripheralAccessContext::new();
                peri_ctx.secure = ctx.secure;
                peri_ctx.requestor = ctx.requestor;

                self.find_peripheral(address, ctx)?
                    .ok_or(BusError::BusFault)?
                    .read(address as u16, &peri_ctx)
                    .map_err(|_| BusError::BusFault)
            }
        }
    }

    fn write_u32(&mut self, address: u32, value: u32, ctx: &BusAccessContext) -> BusResult<()> {
        if !self.is_address_free(address, ctx) {
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
            Self::ROM => self.rom.write_u32(address, value)?,
            Self::SRAM => self.sram.write_u32(address - Self::SRAM, value)?,
            Self::XIP => self.xip.write_u32(address - Self::XIP, value)?,
            _ => {
                let mut peri_ctx = PeripheralAccessContext::new();
                peri_ctx.secure = ctx.secure;
                peri_ctx.requestor = ctx.requestor;

                self.find_peripheral_mut(address, ctx)?
                    .ok_or(BusError::BusFault)?
                    .write(address as u16, value, &peri_ctx)
                    .map_err(|_| BusError::BusFault)?
            }
        }

        Ok(())
    }

    fn read_u16(&mut self, address: u32, ctx: &BusAccessContext) -> BusResult<u16> {
        match address & 0xF000_0000 {
            Self::ROM => Ok(self.rom.read_u16(address)?),
            Self::SRAM => Ok(self.sram.read_u16(address - Self::SRAM)?),
            Self::XIP => Ok(self.xip.read_u16(address - Self::XIP)?),
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

    fn write_u16(&mut self, address: u32, value: u32, ctx: &BusAccessContext) -> BusResult<()> {
        match address & 0xF000_0000 {
            Self::ROM => self.rom.write_u16(address, value as u16)?,
            Self::SRAM => self.sram.write_u16(address - Self::SRAM, value as u16)?,
            Self::XIP => self.xip.write_u16(address - Self::XIP, value as u16)?,
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

    fn read_u8(&mut self, address: u32, ctx: &BusAccessContext) -> BusResult<u8> {
        match address & 0xF000_0000 {
            Self::ROM => Ok(self.rom.read_u8(address)?),
            Self::SRAM => Ok(self.sram.read_u8(address - Self::SRAM)?),
            Self::XIP => Ok(self.xip.read_u8(address - Self::XIP)?),
            _ => {
                let value = self.read_u32(address & !0b11, ctx)?;
                let index = address as usize & 0b11;
                Ok(value.to_le_bytes()[index])
            }
        }
    }

    fn write_u8(&mut self, address: u32, value: u32, ctx: &BusAccessContext) -> BusResult<()> {
        match address & 0xF000_0000 {
            Self::ROM => self.rom.write_u8(address, value as u8)?,
            Self::SRAM => self.sram.write_u8(address - Self::SRAM, value as u8)?,
            Self::XIP => self.xip.write_u8(address - Self::XIP, value as u8)?,
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

    #[test]
    fn fetch() {
        let mut bus = Bus::default();
        let address = Bus::SRAM;
        let value = 0x1234_5678;
        bus.write_u32(address, value, &Default::default()).unwrap();

        assert_eq!(bus.fetch(address), Ok(value));
    }

    #[test]
    fn fetch_error() {
        let mut bus = Bus::default();
        let address = 0x4000_0000;
        assert_eq!(bus.fetch(address), Err(BusError::BusFault));
    }

    // #[test]
    // fn load() {
    //     let mut bus = Bus::default();
    //     let address = Bus::SRAM;
    //     let value = 0x1234_5678;
    //     bus.write_u32(address, value).unwrap();

    //     let status = bus.load(address, DataSize::Word).unwrap();
    //     assert_eq!(*status.borrow(), LoadStatus::Waiting);

    //     bus.tick();
    //     assert_eq!(*status.borrow(), LoadStatus::Word(value));
    // }
}

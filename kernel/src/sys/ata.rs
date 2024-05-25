// https://github.com/vinc/moros/blob/trunk/src/sys/ata.rs

use crate::{
    api::fs::{FileIO, IO},
    log, sys,
};
use alloc::{boxed::Box, fmt, string::String, vec::Vec};
use bit_field::BitField as _;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

pub const BLOCK_SIZE: usize = 512;

lazy_static! {
    pub static ref BUSES: Mutex<Vec<Bus>> = Mutex::new(Vec::new());
}

pub fn init() {
    {
        let mut buses = BUSES.lock();
        buses.push(Bus::new(0, 0x1F0, 0x3F6, 14));
        buses.push(Bus::new(1, 0x170, 0x376, 15));
    }

    for drive in list() {
        log!("ATA {}:{} {}", drive.bus, drive.dsk, drive);
    }
}

// Keep track of the last selected bus and drive pair to speed up operations
pub static LAST_SELECTED: Mutex<Option<(u8, u8)>> = Mutex::new(None);

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
enum Command {
    Read = 0x20,
    Write = 0x30,
    Identify = 0xEC,
}

enum IdentifyResponse {
    Ata(Box<[u16; 256]>),
    Atapi,
    Sata,
    None,
}

#[allow(dead_code)]
#[repr(usize)]
#[derive(Debug, Clone, Copy)]
enum Status {
    Err = 0,  // Error
    Idx = 1,  // (obsolete)
    Corr = 2, // (obsolete)
    Drq = 3,  // Data Request
    Dsc = 4,  // (command dependant)
    Df = 5,   // (command dependant)
    Drdy = 6, // Device Ready
    Bsy = 7,  // Busy
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Bus {
    id: u8,
    irq: u8,

    data_register: Port<u16>,
    error_register: PortReadOnly<u8>,
    features_register: PortWriteOnly<u8>,
    sector_count_register: Port<u8>,
    lba0_register: Port<u8>,
    lba1_register: Port<u8>,
    lba2_register: Port<u8>,
    drive_register: Port<u8>,
    status_register: PortReadOnly<u8>,
    command_register: PortWriteOnly<u8>,

    alternate_status_register: PortReadOnly<u8>,
    control_register: PortWriteOnly<u8>,
    drive_blockess_register: PortReadOnly<u8>,
}

impl Bus {
    #[must_use]
    pub const fn new(id: u8, io_base: u16, ctrl_base: u16, irq: u8) -> Self {
        Self {
            id,
            irq,
            data_register: Port::new(io_base),
            error_register: PortReadOnly::new(io_base + 1),
            features_register: PortWriteOnly::new(io_base + 1),
            sector_count_register: Port::new(io_base + 2),
            lba0_register: Port::new(io_base + 3),
            lba1_register: Port::new(io_base + 4),
            lba2_register: Port::new(io_base + 5),
            drive_register: Port::new(io_base + 6),
            status_register: PortReadOnly::new(io_base + 7),
            command_register: PortWriteOnly::new(io_base + 7),
            alternate_status_register: PortReadOnly::new(ctrl_base),
            control_register: PortWriteOnly::new(ctrl_base),
            drive_blockess_register: PortReadOnly::new(ctrl_base + 1),
        }
    }

    fn check_floating_bus(&mut self) -> Result<(), ()> {
        match self.status() {
            0xFF | 0x7F => Err(()),
            _ => Ok(()),
        }
    }

    fn wait(ns: u64) {
        sys::time::nanowait(ns);
    }

    fn clear_interrupt(&mut self) -> u8 {
        unsafe { self.status_register.read() }
    }

    fn status(&mut self) -> u8 {
        unsafe { self.alternate_status_register.read() }
    }

    fn lba1(&mut self) -> u8 {
        unsafe { self.lba1_register.read() }
    }

    fn lba2(&mut self) -> u8 {
        unsafe { self.lba2_register.read() }
    }

    fn read_data(&mut self) -> u16 {
        unsafe { self.data_register.read() }
    }

    fn write_data(&mut self, data: u16) {
        unsafe { self.data_register.write(data) }
    }

    fn is_error(&mut self) -> bool {
        self.status().get_bit(Status::Err as usize)
    }

    fn poll(&mut self, bit: Status, val: bool) -> Option<()> {
        let start = sys::clock::uptime();
        while self.status().get_bit(bit as usize) != val {
            if sys::clock::uptime() - start > 1.0 {
                log!("ATA hanged while polling {:?} bit in status register", bit);
                self.debug();
                return None;
            }
            core::hint::spin_loop();
        }
        Some(())
    }

    fn select_drive(&mut self, drive: u8) -> Option<()> {
        self.poll(Status::Bsy, false)?;
        self.poll(Status::Drq, false)?;

        // Skip the rest if this drive was already selected
        if *LAST_SELECTED.lock() == Some((self.id, drive)) {
            return Some(());
        }

        *LAST_SELECTED.lock() = Some((self.id, drive));

        unsafe {
            // Bit 4 => DEV
            // Bit 5 => 1
            // Bit 7 => 1
            self.drive_register.write(0xA0 | (drive << 4));
        }
        sys::time::nanowait(400); // Wait at least 400 ns
        self.poll(Status::Bsy, false)?;
        self.poll(Status::Drq, false)?;
        Some(())
    }

    fn write_command_params(&mut self, drive: u8, block: u32) {
        let lba = true;
        let mut bytes = block.to_le_bytes();
        bytes[3].set_bit(4, drive > 0);
        bytes[3].set_bit(5, true);
        bytes[3].set_bit(6, lba);
        bytes[3].set_bit(7, true);

        unsafe { self.sector_count_register.write(1) };
        unsafe { self.lba0_register.write(bytes[0]) };
        unsafe { self.lba1_register.write(bytes[1]) };
        unsafe { self.lba2_register.write(bytes[2]) };
        unsafe { self.drive_register.write(bytes[3]) };
    }

    fn write_command(&mut self, cmd: Command) -> Option<()> {
        unsafe { self.command_register.write(cmd as u8) }
        Self::wait(400); // Wait at least 400 ns
        self.status(); // Ignore results of first read
        self.clear_interrupt();
        if self.status() == 0 {
            // Drive does not exist
            return None;
        }
        if self.is_error() {
            //debug!("ATA {:?} command errored", cmd);
            //self.debug();
            return None;
        }
        self.poll(Status::Bsy, false)?;
        self.poll(Status::Drq, true)?;
        Some(())
    }

    fn setup_pio(&mut self, drive: u8, block: u32) -> Option<()> {
        self.select_drive(drive)?;
        self.write_command_params(drive, block);
        Some(())
    }

    fn read(&mut self, drive: u8, block: u32, buf: &mut [u8]) -> Option<()> {
        debug_assert!(buf.len() == BLOCK_SIZE);
        self.setup_pio(drive, block)?;
        self.write_command(Command::Read)?;
        for chunk in buf.chunks_mut(2) {
            let data = self.read_data().to_le_bytes();
            chunk.clone_from_slice(&data);
        }
        if self.is_error() {
            log!("ATA read: data error");
            self.debug();
            None
        } else {
            Some(())
        }
    }

    fn write(&mut self, drive: u8, block: u32, buf: &[u8]) -> Option<()> {
        debug_assert!(buf.len() == BLOCK_SIZE);
        self.setup_pio(drive, block)?;
        self.write_command(Command::Write)?;
        for chunk in buf.chunks(2) {
            let data = u16::from_le_bytes(chunk.try_into().unwrap());
            self.write_data(data);
        }
        if self.is_error() {
            log!("ATA write: data error");
            self.debug();
            None
        } else {
            Some(())
        }
    }

    fn identify_drive(&mut self, drive: u8) -> Option<IdentifyResponse> {
        if self.check_floating_bus().is_err() {
            return Some(IdentifyResponse::None);
        }
        self.select_drive(drive)?;
        self.write_command_params(drive, 0);
        if self.write_command(Command::Identify).is_none() {
            if self.status() == 0 {
                return Some(IdentifyResponse::None);
            }

            return None;
        }
        match (self.lba1(), self.lba2()) {
            (0x00, 0x00) => Some(IdentifyResponse::Ata(Box::new(
                [(); 256].map(|()| self.read_data()),
            ))),
            (0x14, 0xEB) => Some(IdentifyResponse::Atapi),
            (0x3C, 0xC3) => Some(IdentifyResponse::Sata),
            (_, _) => None,
        }
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        unsafe { self.control_register.write(4) }; // Set SRST bit
        Self::wait(5); // Wait at least 5 ns
        unsafe { self.control_register.write(0) }; // Then clear it
        Self::wait(2000); // Wait at least 2 ms
    }

    #[allow(dead_code)]
    fn debug(&mut self) {
        log!(
            "ATA status register: 0b{:08b} <BSY|DRDY|#|#|DRQ|#|#|ERR>",
            unsafe { self.alternate_status_register.read() }
        );
        log!(
            "ATA error register:  0b{:08b} <#|#|#|#|#|ABRT|#|#>",
            unsafe { self.error_register.read() }
        );
    }
}

#[derive(Clone, Debug)]
pub struct Drive {
    pub bus: u8,
    pub dsk: u8,
    model: String,
    serial: String,
    block_count: u32,
    block_index: u32,
}

impl Drive {
    #[must_use]
    pub const fn size() -> usize {
        BLOCK_SIZE
    }

    pub fn open(bus: u8, dsk: u8) -> Option<Self> {
        let mut buses = BUSES.lock();
        let res = buses[bus as usize].identify_drive(dsk);
        if let Some(IdentifyResponse::Ata(res)) = res {
            let buffer = res.map(u16::to_be_bytes).concat();
            let model = String::from_utf8_lossy(&buffer[54..94]).trim().into();
            let serial = String::from_utf8_lossy(&buffer[20..40]).trim().into();
            let block_count =
                u32::from_be_bytes(buffer[120..124].try_into().unwrap()).rotate_left(16);
            let block_index = 0;

            Some(Self {
                bus,
                dsk,
                model,
                serial,
                block_count,
                block_index,
            })
        } else {
            None
        }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn block_size(&self) -> u32 {
        BLOCK_SIZE as u32
    }

    #[must_use]
    pub const fn block_count(&self) -> u32 {
        self.block_count
    }

    fn humanized_size(&self) -> (usize, String) {
        let size = self.block_size() as usize;
        let count = self.block_count() as usize;
        let bytes = size * count;
        if bytes >> 20 < 1000 {
            (bytes >> 20, String::from("MB"))
        } else {
            (bytes >> 30, String::from("GB"))
        }
    }
}

impl FileIO for Drive {
    fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        if self.block_index == self.block_count {
            return Some(0);
        }

        let mut buses = BUSES.lock();
        let _ = buses[self.bus as usize].read(self.dsk, self.block_index, buf);
        let n = buf.len();
        self.block_index += 1;
        Some(n)
    }

    fn write(&mut self, _buf: &[u8]) -> Option<usize> {
        unimplemented!();
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => false,
        }
    }
}

impl fmt::Display for Drive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (size, unit) = self.humanized_size();
        write!(f, "{} {} ({} {})", self.model, self.serial, size, unit)
    }
}

#[must_use]
pub fn list() -> Vec<Drive> {
    let mut res = Vec::new();
    for bus in 0..2 {
        for dsk in 0..2 {
            if let Some(drive) = Drive::open(bus, dsk) {
                res.push(drive);
            }
        }
    }
    res
}

pub fn read(bus: u8, drive: u8, block: u32, buffer: &mut [u8]) -> Option<()> {
    let mut buses = BUSES.lock();
    buses[bus as usize].read(drive, block, buffer)
}

#[must_use]
pub fn write(bus: u8, drive: u8, block: u32, buffer: &[u8]) -> Option<()> {
    let mut buses = BUSES.lock();
    buses[bus as usize].write(drive, block, buffer)
}

//! UF2 parser
//! Author: Nguyen Le Duy
//! Date: 08/04/2025
//!

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Uf2Block {
    // 32 byte header
    pub flags: u32,
    pub target_addr: u32,
    pub block_no: u32,
    pub num_blocks: u32,
    pub data: Vec<u8>,
    pub family_id: Option<u32>,
}

impl Uf2Block {
    /// Check if the block can be flashed to the target device
    pub fn is_flashable(&self) -> bool {
        self.flags & 1 != 0
    }
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    // Since we use `chunks_exact(512)` and the max offset is 508
    // we can safely assume that the data is at least 4 bytes long

    let mut value = 0u32;

    for i in 0..4 {
        // little endianness
        value |= (data[offset + i] as u32) << (i * 8);
    }

    value
}

#[derive(Debug, Error, Clone, Copy)]
pub enum Error {
    #[error("Invalid UF2 file")]
    InvalidUF2File,
}

pub fn read_uf2(data: &[u8]) -> Result<impl Iterator<Item = Uf2Block>, Error> {
    if data.len() % 512 != 0 {
        return Err(Error::InvalidUF2File);
    }

    Ok(data.chunks_exact(512).filter_map(|v| {
        let magic_start0 = read_u32(v, 0);
        let magic_start1 = read_u32(v, 4);
        let magic_end = read_u32(v, 508);

        if (magic_start0, magic_start1, magic_end) != (0x0A32_4655, 0x9E5D_5157, 0x0AB16F30) {
            return None;
        }

        let flags = read_u32(v, 8);
        let target_addr = read_u32(v, 12);
        let payload_size = read_u32(v, 16);
        let block_no = read_u32(v, 20);
        let num_blocks = read_u32(v, 24);

        let family_id = if flags & 0x2000 != 0 {
            Some(read_u32(v, 28))
        } else {
            None
        };

        let data = data[32..508]
            .into_iter()
            .take(payload_size as usize)
            .cloned()
            .collect::<Vec<u8>>();

        Some(Uf2Block {
            flags,
            target_addr,
            block_no,
            num_blocks,
            data,
            family_id,
        })
    }))
}

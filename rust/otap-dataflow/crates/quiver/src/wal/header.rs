use std::io::{Read, Seek, SeekFrom, Write};

use super::{WAL_MAGIC, WalError};

pub(crate) const WAL_VERSION: u16 = 1;
pub(crate) const WAL_HEADER_LEN: usize = WAL_MAGIC.len() + 2 + 2 + 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WalHeader {
    pub segment_cfg_hash: [u8; 16],
}

impl WalHeader {
    pub fn new(segment_cfg_hash: [u8; 16]) -> Self {
        Self { segment_cfg_hash }
    }

    pub fn encode(&self) -> [u8; WAL_HEADER_LEN] {
        let mut buf = [0u8; WAL_HEADER_LEN];
        let mut cursor = 0;
        buf[cursor..cursor + WAL_MAGIC.len()].copy_from_slice(WAL_MAGIC);
        cursor += WAL_MAGIC.len();

        buf[cursor..cursor + 2].copy_from_slice(&WAL_VERSION.to_le_bytes());
        cursor += 2;

        buf[cursor..cursor + 2].copy_from_slice(&0u16.to_le_bytes());
        cursor += 2;

        buf[cursor..cursor + 16].copy_from_slice(&self.segment_cfg_hash);
        buf
    }

    pub fn write_to(&self, file: &mut (impl Write + Seek)) -> Result<(), WalError> {
        let _ = file.seek(SeekFrom::Start(0))?;
        file.write_all(&self.encode())?;
        file.flush()?;
        Ok(())
    }

    pub fn read_from(file: &mut (impl Read + Seek)) -> Result<Self, WalError> {
        let _ = file.seek(SeekFrom::Start(0))?;
        let mut buf = [0u8; WAL_HEADER_LEN];
        file.read_exact(&mut buf)?;
        Self::decode(&buf)
    }

    pub fn decode(buf: &[u8]) -> Result<Self, WalError> {
        if buf.len() < WAL_HEADER_LEN {
            return Err(WalError::InvalidHeader("buffer too short"));
        }
        let mut cursor = 0;
        if &buf[cursor..cursor + WAL_MAGIC.len()] != WAL_MAGIC {
            return Err(WalError::InvalidHeader("magic mismatch"));
        }
        cursor += WAL_MAGIC.len();

        let version = u16::from_le_bytes([buf[cursor], buf[cursor + 1]]);
        cursor += 2;
        if version != WAL_VERSION {
            return Err(WalError::InvalidHeader("unsupported version"));
        }

        // Reserved field; must be zero today.
        if buf[cursor] != 0 || buf[cursor + 1] != 0 {
            return Err(WalError::InvalidHeader("reserved bits non-zero"));
        }
        cursor += 2;

        let mut segment_cfg_hash = [0u8; 16];
        segment_cfg_hash.copy_from_slice(&buf[cursor..cursor + 16]);

        Ok(Self { segment_cfg_hash })
    }
}

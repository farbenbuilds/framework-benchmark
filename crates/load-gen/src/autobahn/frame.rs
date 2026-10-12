use crate::error::LoadGenError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameOpcode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl TryFrom<u8> for FrameOpcode {
    type Error = LoadGenError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x0 => Ok(Self::Continuation),
            0x1 => Ok(Self::Text),
            0x2 => Ok(Self::Binary),
            0x8 => Ok(Self::Close),
            0x9 => Ok(Self::Ping),
            0xA => Ok(Self::Pong),
            _ => Err(LoadGenError::Protocol(format!("unknown opcode: {val:#x}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameHeader {
    pub fin: bool,
    pub rsv1: bool,
    pub rsv2: bool,
    pub rsv3: bool,
    pub opcode: FrameOpcode,
    pub masked: bool,
    pub payload_len: usize,
    pub masking_key: Option<[u8; 4]>,
}

pub fn parse_frame_header(buf: &[u8]) -> Result<Option<(FrameHeader, usize)>, LoadGenError> {
    if buf.len() < 2 {
        return Ok(None);
    }

    let b0 = buf[0];
    let b1 = buf[1];

    let fin = (b0 & 0x80) != 0;
    let rsv1 = (b0 & 0x40) != 0;
    let rsv2 = (b0 & 0x20) != 0;
    let rsv3 = (b0 & 0x10) != 0;
    let opcode = FrameOpcode::try_from(b0 & 0x0F)?;

    let masked = (b1 & 0x80) != 0;
    let basic_len = (b1 & 0x7F) as usize;

    let mut offset = 2;
    let payload_len = match basic_len {
        126 => {
            if buf.len() < offset + 2 {
                return Ok(None);
            }
            let len = u16::from_be_bytes([buf[offset], buf[offset + 1]]) as usize;
            offset += 2;
            len
        }
        127 => {
            if buf.len() < offset + 8 {
                return Ok(None);
            }
            let slice: [u8; 8] = match buf[offset..offset + 8].try_into() {
                Ok(arr) => arr,
                Err(_) => return Ok(None),
            };
            let len = u64::from_be_bytes(slice) as usize;
            offset += 8;
            len
        }
        len => len,
    };

    let masking_key = if masked {
        if buf.len() < offset + 4 {
            return Ok(None);
        }
        let mut key = [0u8; 4];
        key.copy_from_slice(&buf[offset..offset + 4]);
        offset += 4;
        Some(key)
    } else {
        None
    };

    Ok(Some((
        FrameHeader {
            fin,
            rsv1,
            rsv2,
            rsv3,
            opcode,
            masked,
            payload_len,
            masking_key,
        },
        offset,
    )))
}

pub fn apply_mask(payload: &mut [u8], mask: [u8; 4]) {
    for (i, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[i % 4];
    }
}

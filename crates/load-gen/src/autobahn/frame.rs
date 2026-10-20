use crate::error::LoadGenError;

pub const MAX_FRAME_PAYLOAD_BYTES: usize = 65_536;
pub const MAX_FRAME_HEADER_BYTES: usize = 14;

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
            if len < 126 {
                return Err(LoadGenError::Protocol(
                    "websocket frame uses noncanonical length".into(),
                ));
            }
            offset += 2;
            len
        }
        127 => {
            if buf.len() < offset + 8 {
                return Ok(None);
            }
            let len = u64::from_be_bytes(buf[offset..offset + 8].try_into().map_err(|_| {
                LoadGenError::Protocol("websocket frame length is truncated".into())
            })?);
            if len >> 63 != 0 {
                return Err(LoadGenError::Protocol(
                    "websocket frame length has high bit set".into(),
                ));
            }
            let len = usize::try_from(len).map_err(|_| {
                LoadGenError::Protocol("websocket frame length exceeds platform size".into())
            })?;
            if u16::try_from(len).is_ok() {
                return Err(LoadGenError::Protocol(
                    "websocket frame uses noncanonical length".into(),
                ));
            }
            offset += 8;
            len
        }
        len => len,
    };

    if payload_len > MAX_FRAME_PAYLOAD_BYTES {
        return Err(LoadGenError::Protocol(
            "websocket frame exceeds payload limit".into(),
        ));
    }

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

    let header = FrameHeader {
        fin,
        rsv1,
        rsv2,
        rsv3,
        opcode,
        masked,
        payload_len,
        masking_key,
    };
    validate_control_frame(&header)?;
    Ok(Some((header, offset)))
}

fn validate_control_frame(header: &FrameHeader) -> Result<(), LoadGenError> {
    if !matches!(
        header.opcode,
        FrameOpcode::Close | FrameOpcode::Ping | FrameOpcode::Pong
    ) {
        return Ok(());
    }
    if !header.fin || header.payload_len > 125 {
        return Err(LoadGenError::Protocol(
            "invalid websocket control frame".into(),
        ));
    }
    Ok(())
}

pub fn apply_mask(payload: &mut [u8], mask: [u8; 4]) {
    for chunk in payload.chunks_mut(4) {
        for (offset, byte) in chunk.iter_mut().enumerate() {
            *byte ^= mask[offset];
        }
    }
}

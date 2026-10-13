use crate::autobahn::frame::{FrameHeader, FrameOpcode, parse_frame_header};
use crate::error::LoadGenError;

pub fn validate_echo_frame(bytes: &[u8], expected: &[u8]) -> Result<Option<usize>, LoadGenError> {
    let Some((header, header_len)) = parse_frame_header(bytes)? else {
        return Ok(None);
    };
    validate_echo_header(&header, expected.len())?;
    let frame_len = header_len
        .checked_add(header.payload_len)
        .ok_or_else(|| LoadGenError::Protocol("websocket frame length overflows".into()))?;
    if bytes.len() < frame_len {
        return Ok(None);
    }
    if bytes.len() != frame_len {
        return Err(LoadGenError::Protocol(
            "unexpected bytes follow websocket echo".into(),
        ));
    }
    if &bytes[header_len..] != expected {
        return Err(LoadGenError::Protocol(
            "websocket echo payload differs".into(),
        ));
    }
    Ok(Some(frame_len))
}

fn validate_echo_header(header: &FrameHeader, expected_len: usize) -> Result<(), LoadGenError> {
    if !header.fin {
        return Err(LoadGenError::Protocol(
            "fragmented websocket echo is unsupported".into(),
        ));
    }
    if header.rsv1 || header.rsv2 || header.rsv3 {
        return Err(LoadGenError::Protocol(
            "websocket echo has reserved bits set".into(),
        ));
    }
    if header.opcode != FrameOpcode::Text {
        return Err(LoadGenError::Protocol(
            "websocket echo is not a text frame".into(),
        ));
    }
    if header.masked {
        return Err(LoadGenError::Protocol(
            "server websocket frame is masked".into(),
        ));
    }
    if header.payload_len != expected_len {
        return Err(LoadGenError::Protocol(
            "websocket echo payload length differs".into(),
        ));
    }
    Ok(())
}

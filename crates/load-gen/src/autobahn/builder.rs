use crate::autobahn::frame::{FrameOpcode, apply_mask};

#[derive(Debug)]
pub struct FrameBuilder;

impl FrameBuilder {
    pub fn build_client_frame(
        opcode: FrameOpcode,
        payload: &[u8],
        mask: [u8; 4],
        fin: bool,
    ) -> Vec<u8> {
        let mut frame = Vec::with_capacity(payload.len() + 14);

        let mut b0 = opcode as u8;
        if fin {
            b0 |= 0x80;
        }
        frame.push(b0);

        let len = payload.len();
        if len <= 125 {
            frame.push(0x80 | (len as u8));
        } else if len <= 65535 {
            frame.push(0x80 | 0x7E);
            frame.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            frame.push(0x80 | 0x7F);
            frame.extend_from_slice(&(len as u64).to_be_bytes());
        }

        frame.extend_from_slice(&mask);

        let mask_start = frame.len();
        frame.extend_from_slice(payload);
        apply_mask(&mut frame[mask_start..], mask);

        frame
    }

    pub fn build_text_message(text: &str, mask: [u8; 4]) -> Vec<u8> {
        Self::build_client_frame(FrameOpcode::Text, text.as_bytes(), mask, true)
    }

    pub fn build_binary_message(data: &[u8], mask: [u8; 4]) -> Vec<u8> {
        Self::build_client_frame(FrameOpcode::Binary, data, mask, true)
    }

    pub fn build_fragmented_text(chunks: &[&str], mask: [u8; 4]) -> Vec<Vec<u8>> {
        let mut frames = Vec::with_capacity(chunks.len());
        for (i, chunk) in chunks.iter().enumerate() {
            let is_first = i == 0;
            let is_last = i == chunks.len() - 1;
            let opcode = if is_first {
                FrameOpcode::Text
            } else {
                FrameOpcode::Continuation
            };
            frames.push(Self::build_client_frame(
                opcode,
                chunk.as_bytes(),
                mask,
                is_last,
            ));
        }
        frames
    }

    pub fn build_fragmented_binary(data: &[u8], chunks: &[usize], mask: [u8; 4]) -> Vec<Vec<u8>> {
        let mut frames = Vec::with_capacity(chunks.len());
        let mut offset = 0usize;
        for (index, size) in chunks.iter().enumerate() {
            let end = offset.saturating_add(*size).min(data.len());
            let opcode = if index == 0 {
                FrameOpcode::Binary
            } else {
                FrameOpcode::Continuation
            };
            frames.push(Self::build_client_frame(
                opcode,
                &data[offset..end],
                mask,
                end == data.len(),
            ));
            offset = end;
            if offset == data.len() {
                break;
            }
        }
        frames
    }

    pub fn build_close_frame(code: u16, mask: [u8; 4]) -> Vec<u8> {
        let payload = code.to_be_bytes();
        Self::build_client_frame(FrameOpcode::Close, &payload, mask, true)
    }
}

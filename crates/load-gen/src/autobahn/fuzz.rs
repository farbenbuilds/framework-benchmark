#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuzzType {
    UnmaskedClientFrame,
    ReservedBitRsv1Set,
    ReservedBitRsv2Set,
    ReservedBitRsv3Set,
    InvalidUtf8InTextFrame,
    FragmentedPingControlFrame,
    OversizedPayload,
    InvalidOpcode,
}

#[derive(Debug)]
pub struct FuzzInjector;

impl FuzzInjector {
    pub fn create_unmasked_frame(payload: &[u8]) -> Vec<u8> {
        let mut frame = Vec::with_capacity(payload.len() + 10);
        frame.push(0x81);
        let len = payload.len();
        if len <= 125 {
            frame.push(len as u8);
        } else {
            frame.push(126);
            frame.extend_from_slice(&(len as u16).to_be_bytes());
        }
        frame.extend_from_slice(payload);
        frame
    }

    pub fn create_reserved_bit_frame(rsv_bit: u8) -> Vec<u8> {
        let mut b0 = 0x81;
        match rsv_bit {
            1 => b0 |= 0x40,
            2 => b0 |= 0x20,
            3 => b0 |= 0x10,
            _ => (),
        }
        vec![b0, 0x80, 0x01, 0x02, 0x03, 0x04]
    }

    pub fn create_invalid_utf8_frame() -> Vec<u8> {
        let invalid_utf8 = [0xCE, 0xBA, 0xE1, 0x1F, 0xB7, 0xCF];
        let mask = [0x11, 0x22, 0x33, 0x44];
        let mut frame = vec![0x81, 0x86];
        frame.extend_from_slice(&mask);
        let mut payload = invalid_utf8;
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= mask[i % 4];
        }
        frame.extend_from_slice(&payload);
        frame
    }

    pub fn create_fragmented_ping() -> Vec<u8> {
        vec![0x09, 0x80, 0x00, 0x00, 0x00, 0x00]
    }
}

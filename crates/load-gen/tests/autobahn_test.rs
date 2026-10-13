use load_gen::autobahn::builder::FrameBuilder;
use load_gen::autobahn::echo::validate_echo_frame;
use load_gen::autobahn::frame::{FrameOpcode, apply_mask, parse_frame_header};
use load_gen::autobahn::fuzz::FuzzInjector;

#[test]
fn test_frame_builder_and_parser_roundtrip() {
    let mask = [0x12, 0x34, 0x56, 0x78];
    let payload = b"Hello, Autobahn!";
    let frame = FrameBuilder::build_text_message("Hello, Autobahn!", mask);

    let parsed = parse_frame_header(&frame).expect("parse header failed");
    assert!(parsed.is_some());
    let (header, offset) = parsed.unwrap();

    assert!(header.fin);
    assert_eq!(header.opcode, FrameOpcode::Text);
    assert!(header.masked);
    assert_eq!(header.payload_len, payload.len());
    assert_eq!(header.masking_key, Some(mask));

    let mut unmasked = frame[offset..offset + header.payload_len].to_vec();
    apply_mask(&mut unmasked, mask);
    assert_eq!(&unmasked, payload);
}

#[test]
fn test_fuzz_generators() {
    let unmasked = FuzzInjector::create_unmasked_frame(b"illegal unmasked payload");
    assert_eq!(unmasked[1] & 0x80, 0);

    let rsv1 = FuzzInjector::create_reserved_bit_frame(1);
    assert_ne!(rsv1[0] & 0x40, 0);

    let invalid_utf8 = FuzzInjector::create_invalid_utf8_frame();
    assert_eq!(invalid_utf8.len(), 12);

    let frag_ping = FuzzInjector::create_fragmented_ping();
    assert_eq!(frag_ping[0] & 0x80, 0);
}

#[test]
fn test_fragmentation_boundaries() {
    let data = vec![7; 64 * 1024];
    let frames =
        FrameBuilder::build_fragmented_binary(&data, &[64, 125, 126, 1024, 16384, 65536], [1; 4]);
    assert_eq!(frames.len(), 6);
    assert_eq!(frames[0][0] & 0x0f, FrameOpcode::Binary as u8);
    assert_eq!(frames[1][0] & 0x0f, FrameOpcode::Continuation as u8);
    assert_ne!(frames.last().unwrap()[0] & 0x80, 0);
}

#[test]
fn echo_validation_waits_for_complete_frame() {
    let frame = [0x81, 0x05, b'h', b'e', b'l'];
    assert_eq!(validate_echo_frame(&frame, b"hello").unwrap(), None);
}

#[test]
fn echo_validation_rejects_wrong_server_frame() {
    let masked = [0x81, 0x81, 1, 2, 3, 4, b'h' ^ 1];
    assert!(validate_echo_frame(&masked, b"h").is_err());
}

#[test]
fn echo_validation_accepts_exact_text_frame() {
    let frame = [0x81, 0x02, b'o', b'k'];
    assert_eq!(
        validate_echo_frame(&frame, b"ok").unwrap(),
        Some(frame.len())
    );
}

#[test]
fn frame_parser_rejects_oversize_and_noncanonical_lengths() {
    let oversize = [0x82, 0x7f, 0, 0, 0, 0, 0, 1, 0, 1];
    let noncanonical = [0x82, 0x7e, 0, 125];
    assert!(parse_frame_header(&oversize).is_err());
    assert!(parse_frame_header(&noncanonical).is_err());
}

#[test]
fn frame_parser_waits_for_truncated_extended_length() {
    assert_eq!(parse_frame_header(&[0x82, 0x7f, 0, 0]).unwrap(), None);
}

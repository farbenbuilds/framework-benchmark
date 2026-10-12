use load_gen::autobahn::builder::FrameBuilder;
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

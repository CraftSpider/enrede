//! Proc macro tests

use enrede_proc::str;

const SHIFTJIS_EXAMPLE: &[u8] = &[
    0x83, 0x6E, 0x83, 0x8D, 0x81, 0x5B, 0x83, 0x8F, 0x81, 0x5B, 0x83, 0x8B, 0x83, 0x68, 0x5C, 0x6E,
    0x82, 0x9F,
];

#[test]
fn test_utf8_literal() {
    let s = str!("HΣllo World!"utf8);

    assert_eq!(s.as_std(), "HΣllo World!");
}

#[test]
fn test_shiftjis_literal() {
    let s = str!("ハローワールド¥nぁ"shiftjis);

    assert_eq!(s.as_bytes(), SHIFTJIS_EXAMPLE);
}

#[test]
fn test_utf8_cstr() {
    let s = str!(c"HΣllo World!"utf8);

    assert_eq!(s.as_std(), "HΣllo World!");
    assert!(s.as_bytes_with_nul().ends_with(&[0]));
}

#[test]
fn test_shiftjis_cstr() {
    let s = str!(c"ハローワールド¥nぁ"shiftjis);

    assert_eq!(s.as_bytes(), SHIFTJIS_EXAMPLE);
    assert!(s.as_bytes_with_nul().ends_with(&[0]));
}

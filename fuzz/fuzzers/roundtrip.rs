#![no_main]

use libfuzzer_sys::fuzz_target;
use turbo_base64::{decode, encode};

fuzz_target!(|data: &[u8]| {
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();

    assert_eq!(data, decoded.as_slice());
});

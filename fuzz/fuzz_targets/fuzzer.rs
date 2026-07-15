#![no_main]

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use libfuzzer_sys::fuzz_target;
use turbo_base64::{decode, encode};

fuzz_target!(|data: &[u8]| {
    let turbo_encoded = encode(data);
    let standard_encoded = BASE64_STANDARD.encode(data);

    assert_eq!(
        turbo_encoded,
        standard_encoded.as_bytes(),
        "Encoding mismatch between turbo_base64 and standard base64"
    );

    let roundtrip_decoded = decode(&turbo_encoded).expect("Turbo failed to decode its own output");
    assert_eq!(roundtrip_decoded, data, "Roundtrip data mismatch");

    let turbo_decode_res = decode(data);
    let standard_decode_res = BASE64_STANDARD.decode(data);

    match (turbo_decode_res, standard_decode_res) {
        (Ok(t_val), Ok(s_val)) => {
            assert_eq!(t_val, s_val, "Decode result mismatch on valid base64 payload");
        }
        (Err(_), Err(_)) => {}
        (Ok(_), Err(_)) => {
            panic!("Turbo accepted invalid base64 that the standard crate rejected");
        }
        (Err(_), Ok(_)) => {
            panic!("Turbo rejected valid base64 that the standard crate accepted");
        }
    }
});

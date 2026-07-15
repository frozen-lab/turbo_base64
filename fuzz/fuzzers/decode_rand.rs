#![no_main]

use libfuzzer_sys::fuzz_target;
use turbo_base64::decode;

fuzz_target!(|data: &[u8]| {
    // NOTE: The data most probably is invalid base64 input, but as long as the function returns
    // a decode error instead of a panic or a crash, that's the correct and expected behaviour.
    let _ = decode(data);
});

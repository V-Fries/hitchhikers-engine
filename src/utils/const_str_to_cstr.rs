#[macro_export]
macro_rules! const_str_to_cstr {
    ($s:expr) => {{
        // Append null byte at compile time
        const LEN: usize = $s.len() + 1;
        const BYTES: &[u8; LEN] = &{
            let mut bytes = [0u8; LEN];
            let mut i = 0;
            while i < $s.len() {
                bytes[i] = $s.as_bytes()[i];
                i += 1;
            }
            bytes[$s.len()] = 0;
            bytes
        };

        unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(BYTES) }
    }};
}

#[macro_export]
macro_rules! const_str_to_cstr {
    ($s:expr) => {{
        // Append null byte at compile time
        const S: &str = $s;
        const LEN: usize = S.len() + 1;
        const BYTES: &[u8; LEN] = &{
            let mut bytes = [0u8; LEN];
            let mut i = 0;
            while i < S.len() {
                bytes[i] = S.as_bytes()[i];
                i += 1;
            }
            bytes[S.len()] = 0;
            bytes
        };

        unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(BYTES) }
    }};
}

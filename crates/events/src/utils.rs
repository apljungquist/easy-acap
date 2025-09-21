use std::ffi::CString;

pub fn cstring_from_str_lossy(s: &str) -> CString {
    match CString::new(s) {
        Ok(s) => s,
        Err(e) => {
            let pos = e.nul_position();
            let mut v = e.into_vec();
            v.truncate(pos + 1);
            CString::from_vec_with_nul(v).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstring_from_str_lossy_truncates_invalid_string() {
        let actual = cstring_from_str_lossy("hello\0world");
        assert_eq!(actual.as_c_str(), c"hello");
    }
}

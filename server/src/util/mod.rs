pub mod channel;
pub mod env;
pub mod helix;
pub mod tracing;

use std::arch::asm;

/// Performs `&str` comparisons in constant time in an attempt to close any and all side-channels
/// that might leak information about our key
pub fn constant_time_cmp(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a = a.as_bytes();
    let b = b.as_bytes();
    let mut res = 0u8;

    // convoluted attempt to avoid optimizations that might do some
    // bullshit to this iterator
    //
    // TODO:
    //  using pointers is probably not amongst god's most efficient methods
    //  for equality testing but i am not so smart and i don't think perfect
    //  efficiency is of utmost importance for this function at present.
    //
    //  ... plus we want to check out decompilation for this function anyway, right?
    for i in 0..a.len() {
        let left = std::hint::black_box(&a[i]) as *const u8;
        let right = std::hint::black_box(&b[i]) as *const u8;

        unsafe {
            asm!(
                "mov {tmp}, [{a_ptr}]",
                "xor {tmp}, [{b_ptr}]",
                "or {res}, {tmp}",
                a_ptr = in(reg) left,
                b_ptr = in(reg) right,
                tmp = out(reg_byte) _,
                res = inout(reg_byte) res,
                options(nostack)
            );
        }
    }

    res == 0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_const_time_cmp() {
        let expects = "test_string";
        let passing = "test_string";

        let bad_start = "__st_string";
        let bad_end = "test_str___";

        let short = "test_strin";
        let long = "test_string_";

        assert!(constant_time_cmp(expects, passing));
        assert!(!constant_time_cmp(expects, bad_start));
        assert!(!constant_time_cmp(expects, bad_end));
        assert!(!constant_time_cmp(expects, short));
        assert!(!constant_time_cmp(expects, long));
    }
}

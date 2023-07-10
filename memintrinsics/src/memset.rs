use core::intrinsics::likely;

// #[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    set_bytes(s, c as u8, n)
}

const WORD_SIZE: usize = 4;
const WORD_MASK: usize = 3;
const WORD_COPY_THRESHOLD: usize = 8;

#[inline(always)]
pub unsafe fn set_bytes(mut s: *mut u8, c: u8, n: usize) -> *mut u8 {
    #[inline(always)]
    pub unsafe fn set_bytes_bytes(mut s: *mut u8, c: u8, e: *mut u8) -> *mut u8 {
        while s != e {
            *s = c;
            s = s.add(1);
        }
        s
    }

    #[inline(always)]
    pub unsafe fn set_bytes_words(s: *mut u8, c: u8, e: *mut u8) -> *mut u8 {
        let mut broadcast = c as usize;
        if WORD_SIZE == 4 {
            broadcast = broadcast | (broadcast << 8);
            broadcast = broadcast | (broadcast << 16);
        } else {
            let mut bits = 8;
            while bits < WORD_SIZE * 8 {
                broadcast |= broadcast << bits;
                bits *= 2;
            }
        }

        let mut s_usize = s as *mut usize;
        let end = e as *mut usize;

        loop {
            *s_usize = broadcast;
            s_usize = s_usize.add(1);
            if s_usize == end {
                break;
            }
        }
        s_usize as *mut u8
    }

    let end = s.add(n);
    if likely(n >= WORD_COPY_THRESHOLD) {
        // Align s
        // Because of n >= 2 * WORD_SIZE, dst_misalignment < n
        // let misalignment = (s as usize).wrapping_neg() & WORD_MASK;
        // let e = s.add(misalignment);
        let e = (((s as usize) + WORD_MASK) & !WORD_MASK) as *mut u8;
        set_bytes_bytes(s, c, e);
        s = e;

        s = set_bytes_words(s, c, ((end as usize) & !WORD_MASK) as *mut u8);
    }
    set_bytes_bytes(s, c, end)
}

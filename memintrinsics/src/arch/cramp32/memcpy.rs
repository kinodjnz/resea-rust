use core::intrinsics::likely;

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    copy_forward(dest, src, n).0
}

const WORD_SIZE: usize = 4;
const WORD_MASK: usize = 3;
const WORD_COPY_THRESHOLD: usize = 8;

#[inline(always)]
pub unsafe fn copy_forward(
    mut dest: *mut u8,
    mut src: *const u8,
    n: usize,
) -> (*mut u8, *const u8) {
    #[inline(always)]
    unsafe fn copy_forward_bytes(
        mut dest: *mut u8,
        mut src: *const u8,
        dest_end: *mut u8,
    ) -> (*mut u8, *const u8) {
        while dest != dest_end {
            *dest = *src;
            dest = dest.add(1);
            src = src.add(1);
        }
        (dest, src)
    }

    #[inline(always)]
    unsafe fn copy_forward_aligned_words(
        dest: *mut u8,
        src: *const u8,
        dest_end: *mut u8,
    ) -> (*mut u8, *const u8) {
        let mut dest_usize = dest as *mut usize;
        let mut src_usize = src as *mut usize;
        let dest_end = dest_end as *mut usize;

        loop {
            // while dest_usize != dest_end {
            *dest_usize = *src_usize;
            dest_usize = dest_usize.add(1);
            src_usize = src_usize.add(1);
            if dest_usize == dest_end {
                break;
            }
        }
        (dest_usize as *mut u8, src_usize as *mut u8)
    }

    #[inline(always)]
    unsafe fn copy_forward_misaligned_words(
        dest: *mut u8,
        src: *const u8,
        dest_end: *mut u8,
    ) -> (*mut u8, *const u8) {
        let mut dest_usize = dest as *mut usize;
        let dest_end = dest_end as *mut usize;

        // Calculate the misalignment offset and shift needed to reassemble value.
        let offset = src as usize & WORD_MASK;
        let shift = offset * 8;

        // Realign src
        let mut src_aligned = (src as usize & !WORD_MASK) as *mut usize;
        // This will read (but won't use) bytes out of bound.
        // cfg needed because not all targets will have atomic loads that can be lowered
        // (e.g. BPF, MSP430), or provided by an external library (e.g. RV32I)
        // #[cfg(target_has_atomic_load_store = "ptr")]
        // let mut prev_word = core::intrinsics::atomic_load_unordered(src_aligned);
        // #[cfg(not(target_has_atomic_load_store = "ptr"))]
        let mut prev_word = core::ptr::read_volatile(src_aligned);

        loop {
            // while dest_usize != dest_end {
            src_aligned = src_aligned.add(1);
            let cur_word = *src_aligned;
            // #[cfg(target_endian = "little")]
            let resembled = prev_word >> shift | cur_word << (WORD_SIZE * 8 - shift);
            // #[cfg(target_endian = "big")]
            // let resembled = prev_word << shift | cur_word >> (WORD_SIZE * 8 - shift);
            prev_word = cur_word;

            *dest_usize = resembled;
            dest_usize = dest_usize.add(1);
            if dest_usize == dest_end {
                break;
            }
        }
        (dest_usize as *mut u8, (src_aligned as *mut u8).add(offset))
    }

    let dest_end = dest.add(n);
    if n >= WORD_COPY_THRESHOLD {
        // Align dest
        // Because of n >= 2 * WORD_SIZE, dst_misalignment < n
        // let dest_misalignment = (dest as usize).wrapping_neg() & WORD_MASK;
        let dest_align_end = (((dest as usize) + WORD_MASK) & !WORD_MASK) as *mut u8;
        (dest, src) = copy_forward_bytes(dest, src, dest_align_end);

        let src_misalignment = src as usize & WORD_MASK;
        let dest_end_word = ((dest_end as usize) & !WORD_MASK) as *mut u8;
        (dest, src) = if likely(src_misalignment == 0) {
            copy_forward_aligned_words(dest, src, dest_end_word)
        } else {
            copy_forward_misaligned_words(dest, src, dest_end_word)
        };
    }
    copy_forward_bytes(dest, src, dest_end)
}

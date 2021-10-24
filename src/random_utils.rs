use libc::c_int;

// Port of the exact random algorithm used for dithering, in order to make
// the hashes match.  After porting, consider replacing this with the rand crate.
const VP8_RANDOM_TABLE_SIZE: usize = 55;

#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8Random {
    index1: c_int,
    index2: c_int,
    tab: [u32; VP8_RANDOM_TABLE_SIZE],
    amp: c_int,
}
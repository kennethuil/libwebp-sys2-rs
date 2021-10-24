use libc::c_int;

use crate::{dsp::WEBP_FILTER_TYPE, vp8_dec::VP8Io};

#[repr(C)]
pub(crate) struct VP8LDecoder {

}
#[repr(C)]
pub(crate) struct ALPHDecoder {
    width: c_int,
    height: c_int,
    method: c_int,
    filter: WEBP_FILTER_TYPE,
    pre_processing: c_int,
    vp8l_dec: *mut VP8LDecoder,
    io: VP8Io,
    use_8b_decode: c_int,
    output: *mut u8,
    prev_line: *mut u8,
}
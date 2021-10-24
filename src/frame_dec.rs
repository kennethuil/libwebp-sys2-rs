use core::slice;
use crate::{array::{to_array_ref, to_array_ref_mut}, dec::{transform_ac3, transform_dc, transform_dc_uv, transform_two, transform_uv}};
use crate::dsp::{UBPS};

//------------------------------------------------------------------------------
// Main reconstruction function.
pub(crate) static K_SCAN: [usize; 16] = [
    0 +  0 * UBPS,  4 +  0 * UBPS, 8 +  0 * UBPS, 12 +  0 * UBPS,
    0 +  4 * UBPS,  4 +  4 * UBPS, 8 +  4 * UBPS, 12 +  4 * UBPS,
    0 +  8 * UBPS,  4 +  8 * UBPS, 8 +  8 * UBPS, 12 +  8 * UBPS,
    0 + 12 * UBPS,  4 + 12 * UBPS, 8 + 12 * UBPS, 12 + 12 * UBPS
];

pub(crate) fn do_transform(bits: u32, src: &[i16], dst: &mut[u8]) {
    match bits >> 30 {
        3 => transform_two(to_array_ref(src), to_array_ref_mut(dst), false),
        2 => transform_ac3(to_array_ref(src), to_array_ref_mut(dst)),
        1 => transform_dc(src[0], to_array_ref_mut(dst)),
        _ => {}
    }
}

pub(crate) fn do_uv_transform(bits: u32, src: &[i16; 64], dst: &mut [u8; 132 + 4 * UBPS]) {
    if bits & 0xff != 0 {       // any non-zero coeff at all?
        if bits & 0xaa != 0 {   // any non-zero AC coefficient?
            transform_uv(src, dst); // note we don't use the AC3 variant for U/V
        } else {
            transform_dc_uv(src, dst);
        }
    }
}


//------------------------------------------------------------------------------------------
// Temporary extern wrappers

#[no_mangle]
unsafe extern "C" fn DoUVTransform(bits: u32, src: *const i16, dst: *mut u8) {
    let arr_src = &*(src as *const [i16; 64]);
    let arr_dst = &mut *(dst as *mut [u8; 132 + 4 * UBPS]);
    do_uv_transform(bits, arr_src, arr_dst);
}

#[no_mangle]
unsafe extern "C" fn DoTransform(bits: u32, src: *const i16, dst: *mut u8) {
    let arr_src = slice::from_raw_parts(src, 32);
    let arr_dst = slice::from_raw_parts_mut(dst, 132);
    do_transform(bits, arr_src, arr_dst);
}

use core::slice;
use crate::{array::{to_array_ref, to_array_ref_mut}, dec::{transform_ac3, transform_dc, transform_dc_uv, transform_two, transform_uv}};
use crate::dsp::{BPS, UBPS};

fn do_transform(bits: u32, src: &[i16], dst: &mut[u8]) {
    match bits >> 30 {
        3 => transform_two(to_array_ref(src), to_array_ref_mut(dst), false),
        2 => transform_ac3(to_array_ref(src), to_array_ref_mut(dst)),
        1 => transform_dc(src[0], to_array_ref_mut(dst)),
        _ => {}
    }
}

fn do_uv_transform(bits: u32, src: &[i16; 64], dst: &mut [u8; 132 + 4 * UBPS]) {
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

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DoUVTransform(bits: u32, src: *const i16, dst: *mut u8) {
    let arr_src = &*(src as *const [i16; 64]);
    let arr_dst = &mut *(dst as *mut [u8; 132 + 4 * UBPS]);
    do_uv_transform(bits, arr_src, arr_dst);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DoTransform(bits: u32, src: *const i16, dst: *mut u8) {
    let arr_src = slice::from_raw_parts(src, 32);
    let arr_dst = slice::from_raw_parts_mut(dst, 132);
    do_transform(bits, arr_src, arr_dst);
}
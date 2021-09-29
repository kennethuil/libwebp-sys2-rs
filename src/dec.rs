use std::os::raw::*;
use std::convert::TryInto;
use bytemuck::TransparentWrapper;

use crate::offsetref::{OffsetArray};
use crate::dec_clip_tables::VP8_KCLIP1;

const BPS: isize = 32;

// DST_SIZE = BPS * SIZE_PARAM + BPS + 1
fn true_motion<const DST_SIZE: usize, const SIZE_PARAM: isize>(dst: &mut OffsetArray<u8, DST_SIZE, {BPS+1}>) {
    // min index: -33 (-BPS - 1)
    // max index: BPS * size
    let mut top = dst.with_offset(-BPS);
    let mut dst_offset = BPS;
    let clip0 = VP8_KCLIP1.with_offset(-(top[-1] as isize));
    for _ in 0..SIZE_PARAM {
        let clip = clip0.with_offset(top[dst_offset-1].into());
        for x in 0..SIZE_PARAM {
            top[x + dst_offset] = clip[top[x].into()];
        }
        dst_offset += BPS;
    }
}

fn tm4(dst: &mut OffsetArray<u8, {((BPS+1)*4+1) as usize}, {BPS+1}>) {
    true_motion::<{((BPS+1)*4+1) as usize}, 4>(dst);
}

fn tm8uv(dst: &mut OffsetArray<u8, {((BPS+1)*8+1) as usize}, {BPS+1}>) {
    true_motion::<{((BPS+1)*8+1) as usize}, 8>(dst);
}

fn tm16(dst: &mut OffsetArray<u8, {((BPS+1)*16+1) as usize}, {BPS+1}>) {
    true_motion::<{((BPS+1)*16+1) as usize}, 16>(dst);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TM4_C(dst: *mut u8) {
    let begin = dst.offset(-BPS-1);
    let dst_arr = &mut *(begin as *mut[u8; ((BPS+1)*4+1) as usize]);
    let dst_arr = OffsetArray::<u8, {((BPS+1)*4+1) as usize}, {BPS+1}>::wrap_mut(dst_arr);
    tm4(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TM8uv_C(dst: *mut u8) {
    let begin = dst.offset(-BPS-1);
    let dst_arr = &mut *(begin as *mut[u8; ((BPS+1)*8+1) as usize]);
    let dst_arr = OffsetArray::<u8, {((BPS+1)*8+1) as usize}, {BPS+1}>::wrap_mut(dst_arr);
    tm8uv(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TM16_C(dst: *mut u8) {
    let begin = dst.offset(-BPS-1);
    let dst_arr = &mut *(begin as *mut[u8; ((BPS+1)*16+1) as usize]);
    let dst_arr = OffsetArray::<u8, {((BPS+1)*16+1) as usize}, {BPS+1}>::wrap_mut(dst_arr);
    tm16(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
fn clip_8b(
    v: c_int
) -> u8 {
    if v & !0xff == 0 {
        v as u8
    } else if v < 0 {
        0
    } else {
        255
    }
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TransformOne_C(data: *const i16, dst: *mut u8) {
    let input_arr = &*(data as *const[i16; 16]);
    let output_arr = &mut *(dst as *mut[u8; 128]);
    transformone(input_arr, output_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TransformTwo_C(data: *const i16, dst: *mut u8, do_two: c_int) {

    let input_arr = &*(data as *const[i16; 32]);
    let output_arr = &mut *(dst as *mut[u8; 132]);
    transformtwo(input_arr, output_arr, do_two != 0);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TransformAC3_C(r#in: *const i16, dst: *mut u8) {
    let input_arr = &*(r#in as *const[i16; 5]);
    let output_arr = &mut *(dst as *mut[u8; 128]);
    transform_ac3(input_arr, output_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DitherCombine8x8_C(dither: *const u8, dst: *mut u8, dst_stride: i32) {
    let dither = &*(dither as *const[u8; 64]);
    let dst_stride = dst_stride as usize;
    let dst = std::slice::from_raw_parts_mut(dst, dst_stride * 8);
    dither_combine_8x8(dither, dst, dst_stride);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TransformDC_C(r#in: *const i16, dst: *mut u8) {
    let input = *r#in;
    let output_arr = &mut *(dst as *mut[u8; 128]);
    transform_dc(input, output_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TransformWHT_C(r#in: *const i16, out: *mut i16) {
    let input = &*(r#in as *const[i16; 16]);
    let output = &mut *(out as *mut[i16; 256]);
    transform_wht(input, output);
}

fn mul1(a: i32) -> i32 {
    ((a * 20091) >> 16) + a
}

fn mul2(a: i32) -> i32 {
    (a * 35468) >> 16
}

fn store(dst: &mut [u8], x: usize, y: usize, v: i32) {
    dst[x + y * 32] = clip_8b(dst[x + y * 32] as i32 + (v >> 3));
}

fn store2(dst: &mut [u8], y: usize, dc: i32, d: i32, c: i32) {
    store(dst, 0, y, dc + d);
    store(dst, 1, y, dc + c);
    store(dst, 2, y, dc - c);
    store(dst, 3, y, dc - d);
}

fn transform_dc(r#in: i16, dst: &mut [u8; 128]) {
    let dc = (r#in + 4).into();
    for i in 0..4 {
        for j in 0..4 {
            store(dst, i, j, dc);
        }
    }
}

fn transformtwo(r#in: &[i16; 32], dst: &mut [u8; 132], do_two: bool) {
    transformone((&r#in[0..16]).try_into().unwrap(),
        (&mut dst[0..128]).try_into().unwrap());
    if do_two {
        transformone((&r#in[16..]).try_into().unwrap(),
        (&mut dst[4..]).try_into().unwrap());
    }
}

fn transformone(data: &[i16; 16], dst: &mut [u8; 128]) {
    let mut c = [0; 4*4];
    let mut tmp = &mut c[..];
    let mut data = &data[..];
    for _ in 0..4 {    // vertical pass
        let a = data[0] as i32 + data[8] as i32;    // [-4096, 4094]
        let b = data[0] as i32 - data[8] as i32;    // [-4095, 4095]
        let c = mul2(data[4].into()) - mul1(data[12].into());   // [-3783, 3783]
        let d = mul1(data[4].into()) + mul2(data[12].into());   // [-3785, 3781]
        tmp[0] = a + d;   // [-7881, 7875]
        tmp[1] = b + c;   // [-7878, 7878]
        tmp[2] = b - c;   // [-7878, 7878]
        tmp[3] = a - d;   // [-7877, 7879]
        tmp = &mut tmp[4..];
        data = &data[1..];
    }
    // Each pass is expanding the dynamic range by ~3.85 (upper bound).
    // The exact value is (2. + (20091 + 35468) / 65536).
    // After the second pass, maximum interval is [-3794, 3794], assuming
    // an input in [-2048, 2047] interval. We then need to add a dst value
    // in the [0, 255] range.
    // In the worst case scenario, the input to clip_8b() can be as large as
    // [-60713, 60968].
    let mut tmp = &c[..];
    let mut dst = &mut dst[..];
    for _ in 0..4 {    // horizontal pass
        let dc = tmp[0] + 4;
        let a = dc + tmp[8];
        let b = dc - tmp[8];
        let c = mul2(tmp[4]) - mul1(tmp[12]);
        let d = mul1(tmp[4]) + mul2(tmp[12]);
        store(dst, 0, 0, a + d);
        store(dst, 1, 0, b + c);
        store(dst, 2, 0, b - c);
        store(dst, 3, 0, a - d);
        tmp = &tmp[1..];
        dst = &mut dst[32..];
    }
}

// Simplified transform when only in[0], in[1] and in[4] are non-zero
fn transform_ac3(r#in: &[i16; 5], dst: &mut [u8; 128]) {
    let a: i32 = r#in[0] as i32 + 4;
    let c4 = mul2(r#in[4].into());
    let d4 = mul1(r#in[4].into());
    let c1 = mul2(r#in[1].into());
    let d1 = mul1(r#in[1].into());
    store2(dst, 0, a + d4, d1, c1);
    store2(dst, 1, a + c4, d1, c1);
    store2(dst, 2, a - c4, d1, c1);
    store2(dst, 3, a - d4, d1, c1);
}

fn transform_wht(r#in: &[i16; 16], out: &mut [i16; 256]) {
    let mut out = &mut out[..];
    let mut tmp = [0; 16];
    for i in 0..4 {
        let a0 = r#in[0 + i] + r#in[12 + i];
        let a1 = r#in[4 + i] + r#in[ 8 + i];
        let a2 = r#in[4 + i] - r#in[ 8 + i];
        let a3 = r#in[0 + i] - r#in[12 + i];
        tmp[ 0 + i] = a0 + a1;
        tmp[ 8 + i] = a0 - a1;
        tmp[ 4 + i] = a3 + a2;
        tmp[12 + i] = a3 - a2;
    }
    for i in 0..4 {
        let dc = tmp[0 + i * 4] + 3; // w/ rounder
        let a0 = dc             + tmp[3 + i * 4];
        let a1 = tmp[1 + i * 4] + tmp[2 + i * 4];
        let a2 = tmp[1 + i * 4] - tmp[2 + i * 4];
        let a3 = dc             - tmp[3 + i * 4];
        out[ 0] = (a0 + a1) >> 3;
        out[16] = (a3 + a2) >> 3;
        out[32] = (a0 - a1) >> 3;
        out[48] = (a3 - a2) >> 3;
        out = &mut out[64..];
    }
}


const VP8_DITHER_AMP_CENTER: i32 = 1 << 7;
const VP8_DITHER_DESCALE_ROUNDER: i32 = 1 << (4 - 1);
const VP8_DITHER_DESCALE: i32 = 4;

fn dither_combine_8x8(dither: &[u8; 64], mut dst: &mut [u8], dst_stride: usize) {
    let mut dither = &dither[..];
    for _ in 0..8 {
        for i in 0..8 {
            let delta0: i32 = dither[i] as i32 - VP8_DITHER_AMP_CENTER;
            let delta1 = (delta0 + VP8_DITHER_DESCALE_ROUNDER) >> VP8_DITHER_DESCALE;
            dst[i] = clip_8b(dst[i] as i32 + delta1);
        }
        dst = &mut dst[dst_stride..];
        dither = &dither[8..];
    }
}
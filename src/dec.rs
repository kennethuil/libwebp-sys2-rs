use std::os::raw::*;
use std::convert::TryInto;
use byteorder::{ByteOrder, LittleEndian};
use crate::offsetref::{OffsetArray};
use crate::dec_clip_tables::VP8_KCLIP1;

const BPS: isize = 32;
const UBPS: usize = BPS as usize;

//------------------------------------------------------------------------------
// Transforms (Paragraph 14.4)

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

//------------------------------------------------------------------------------
// Paragraph 14.3

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

//------------------------------------------------------------------------------
// Intra predictions

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

//------------------------------------------------------------------------------
// 16x16

fn ve16(dst: &mut OffsetArray<u8, {UBPS*16+16}, {BPS}>) { // vertical
    let (src, dst) = dst.split_at_mut(0);
    let src = &src[0..16];
    for dst_line in dst.chunks_exact_mut(UBPS/2).step_by(2) {
        dst_line.copy_from_slice(src);
    }
}

fn he16(dst: &mut OffsetArray<u8, {(BPS*15+17) as usize}, 1>) { // horizontal
    // TODO: Do we really get full BPS-sized chunks or not?
    for chunk in dst.chunks_mut(BPS as usize) {
        let v = chunk[0];
        chunk[1..17].fill(v);
    }
}

fn put16(v: u8, dst: &mut [u8; 15*UBPS+16]) {
    for chunk in dst.chunks_exact_mut(UBPS/2).step_by(2) {
        chunk.fill(v);
    }
}

fn dc16(dst: &mut OffsetArray<u8, {UBPS*16+16}, {BPS}>) {  // DC
    let mut dc:u32 = 16;
    for j in 0..16 {
        let first = dst[-1 + j * BPS] as u32;
        let second = dst[j-BPS] as u32;
        let sum = first + second;
        dc = dc.wrapping_add(sum);
    }

    // Parentheses around &mut dst[0..BPS*16] needed because otherwise try_into will
    // make a temporary array (not array ref) and then &mut will make a reference to the temporary
    // and put16 will then write to the temporary.
    put16(((dc >> 5) & 0xff) as u8, (&mut dst[0..BPS*15+16]).try_into().unwrap());
}

fn dc16_no_top(dst: &mut OffsetArray<u8, {UBPS*15+17}, 1>) {  // DC with top samples not available
    let mut dc:u32 = 8;
    for j in 0..16 {
        dc = dc.wrapping_add(dst[-1 + j * BPS] as u32);
    }
    put16(((dc >> 4) & 0xff) as u8, (&mut dst[0..BPS*15+16]).try_into().unwrap());
}

fn dc16_no_left(dst: &mut OffsetArray<u8, {UBPS*16+16}, {BPS}>) {  // DC with left samples not available
    let mut dc:u32 = 8;
    for i in 0..16 {
        dc = dc.wrapping_add(dst[i - BPS] as u32);
    }
    put16(((dc >> 4) & 0xff) as u8, (&mut dst[0..BPS*15+16]).try_into().unwrap());
}

fn dc16_no_top_left(dst: &mut [u8; UBPS*15+16]) {  // DC with no top and left samples
    put16(0x80, dst);
}

//------------------------------------------------------------------------------
// 4x4
fn avg3(p1: u8, p2: u8, p3: u8) -> u8 {
    (((p1 as u32) + 2 * (p2 as u32) + (p3 as u32) + 2) >> 2) as u8
}

fn avg2(p1: u8, p2: u8) -> u8 {
    (((p1 as u32) + (p2 as u32) + 1) >> 1) as u8
}

fn ve4(dst: &mut OffsetArray<u8, {5*UBPS+1}, {BPS+1}>) { // vertical
    let top = dst.with_offset(-BPS);
    let vals = [
        avg3(top[-1], top[0], top[1]),
        avg3(top[ 0], top[1], top[2]),
        avg3(top[ 1], top[2], top[3]),
        avg3(top[ 2], top[3], top[4]),
    ];
    for chunk in dst[0..].chunks_exact_mut(UBPS) {
        chunk[0..4].copy_from_slice(&vals);
    }
}

// #define DST(x, y) dst[(x) + (y) * BPS]
fn he4(dst: &mut OffsetArray<u8, {5*UBPS+1}, {BPS+1}>) {  // horizontal
    // (-1-BPS)..(4*BPS)
    let a = dst[-1 - BPS];
    let b = dst[-1];
    let c = dst[-1 + BPS];
    let d = dst[-1 + 2 * BPS];
    let e = dst[-1 + 3 * BPS];
    LittleEndian::write_u32(&mut dst[0 * BPS..], 0x01010101 * avg3(a, b, c) as u32);
    LittleEndian::write_u32(&mut dst[1 * BPS..], 0x01010101 * avg3(b, c, d) as u32);
    LittleEndian::write_u32(&mut dst[2 * BPS..], 0x01010101 * avg3(c, d, e) as u32);
    LittleEndian::write_u32(&mut dst[3 * BPS..], 0x01010101 * avg3(d, e, e) as u32);
}


fn dc4(dst: &mut OffsetArray<u8, {5*UBPS}, {BPS}>) { // DC
    let mut dc = 4;

    for i in 0..4 {
        dc += (dst[i - BPS] as u32) + (dst[-1 + i * BPS] as u32);
    }
    let dc = (dc >> 3) as u8;
    for chunk in dst[0..].chunks_exact_mut(UBPS) {
        chunk[0..4].fill(dc);
    }
}

fn assign<const NUM_COORDS: usize>(dst: &mut [u8], xy: [(usize,usize); NUM_COORDS], val: u8) {
    for (x,y) in xy {
        dst[x + y * UBPS] = val;
    }
}

fn rd4(dst: &mut OffsetArray<u8, {4*UBPS+5}, {1+BPS}>) { // Down-right
    let i = dst[-1 + 0 * BPS];
    let j = dst[-1 + 1 * BPS];
    let k = dst[-1 + 2 * BPS];
    let l = dst[-1 + 3 * BPS];
    let x = dst[-1 - BPS];
    let a = dst[ 0 - BPS];
    let b = dst[ 1 - BPS];
    let c = dst[ 2 - BPS];
    let d = dst[ 3 - BPS];
    assign(&mut dst[0..], [(0, 3)],                         avg3(j, k, l));
    assign(&mut dst[0..], [(1, 3), (0, 2)],                 avg3(i, j, k));
    assign(&mut dst[0..], [(2, 3), (1, 2), (0, 1)],         avg3(x, i, j));
    assign(&mut dst[0..], [(3, 3), (2, 2), (1, 1), (0, 0)], avg3(a, x, i));
    assign(&mut dst[0..], [        (3, 2), (2, 1), (1, 0)], avg3(b, a, x));
    assign(&mut dst[0..], [                (3, 1), (2, 0)], avg3(c, b, a));
    assign(&mut dst[0..], [                        (3, 0)], avg3(d, c, b));
}

fn ld4(dst: &mut OffsetArray<u8, {4*UBPS+4}, {BPS}>) { // Down-Left
    let a = dst[0 - BPS];
    let b = dst[1 - BPS];
    let c = dst[2 - BPS];
    let d = dst[3 - BPS];
    let e = dst[4 - BPS];
    let f = dst[5 - BPS];
    let g = dst[6 - BPS];
    let h = dst[7 - BPS];
    assign(&mut dst[0..], [(0, 0)],                         avg3(a, b, c));
    assign(&mut dst[0..], [(1, 0), (0, 1)],                 avg3(b, c, d));
    assign(&mut dst[0..], [(2, 0), (1, 1), (0, 2)],         avg3(c, d, e));
    assign(&mut dst[0..], [(3, 0), (2, 1), (1, 2), (0, 3)], avg3(d, e, f));
    assign(&mut dst[0..], [        (3, 1), (2, 2), (1, 3)], avg3(e, f, g));
    assign(&mut dst[0..], [                (3, 2), (2, 3)], avg3(f, g, h));
    assign(&mut dst[0..], [                        (3, 3)], avg3(g, h, h));
}

fn vr4(dst: &mut OffsetArray<u8, {4*UBPS+5}, {1+BPS}>) { // Vertical-Right
    let i = dst[-1 + 0 * BPS];
    let j = dst[-1 + 1 * BPS];
    let k = dst[-1 + 2 * BPS];
    let x = dst[-1 - BPS];
    let a = dst[0 - BPS];
    let b = dst[1 - BPS];
    let c = dst[2 - BPS];
    let d = dst[3 - BPS];
    assign(&mut dst[0..], [(0, 0), (1, 2)], avg2(x, a));
    assign(&mut dst[0..], [(1, 0), (2, 2)], avg2(a, b));
    assign(&mut dst[0..], [(2, 0), (3, 2)], avg2(b, c));
    assign(&mut dst[0..], [(3, 0)],         avg2(c, d));

    assign(&mut dst[0..], [(0, 3)],         avg3(k, j, i));
    assign(&mut dst[0..], [(0, 2)],         avg3(j, i, x));
    assign(&mut dst[0..], [(0, 1), (1, 3)], avg3(i, x, a));
    assign(&mut dst[0..], [(1, 1), (2, 3)], avg3(x, a, b));
    assign(&mut dst[0..], [(2, 1), (3, 3)], avg3(a, b, c));
    assign(&mut dst[0..], [(3, 1)],         avg3(b, c, d));
}

fn vl4(dst: &mut OffsetArray<u8, {4*UBPS+4}, {BPS}>) {  // Vertical-Left
    let a = dst[0 - BPS];
    let b = dst[1 - BPS];
    let c = dst[2 - BPS];
    let d = dst[3 - BPS];
    let e = dst[4 - BPS];
    let f = dst[5 - BPS];
    let g = dst[6 - BPS];
    let h = dst[7 - BPS];
    assign(&mut dst[0..], [(0, 0)],         avg2(a, b));
    assign(&mut dst[0..], [(1, 0), (0, 2)], avg2(b, c));
    assign(&mut dst[0..], [(2, 0), (1, 2)], avg2(c, d));
    assign(&mut dst[0..], [(3, 0), (2, 2)], avg2(d, e));

    assign(&mut dst[0..], [(0, 1)],         avg3(a, b, c));
    assign(&mut dst[0..], [(1, 1), (0, 3)], avg3(b, c, d));
    assign(&mut dst[0..], [(2, 1), (1, 3)], avg3(c, d, e));
    assign(&mut dst[0..], [(3, 1), (2, 3)], avg3(d, e, f));
    assign(&mut dst[0..], [        (3, 2)], avg3(e, f, g));
    assign(&mut dst[0..], [        (3, 3)], avg3(f, g, h));
}

fn hu4(dst: &mut OffsetArray<u8, {4*UBPS+5}, {1+BPS}>) { // Horizontal-Up
    let i = dst[-1 + 0 * BPS];
    let j = dst[-1 + 1 * BPS];
    let k = dst[-1 + 2 * BPS];
    let l = dst[-1 + 3 * BPS];
    assign(&mut dst[0..], [(0, 0)],         avg2(i, j));
    assign(&mut dst[0..], [(2, 0), (0, 1)], avg2(j, k));
    assign(&mut dst[0..], [(2, 1), (0, 2)], avg2(k, l));
    assign(&mut dst[0..], [(1, 0)],         avg3(i, j, k));
    assign(&mut dst[0..], [(3, 0), (1, 1)], avg3(j, k, l));
    assign(&mut dst[0..], [(3, 1), (1, 2)], avg3(k, l, l));
    assign(&mut dst[0..], [(3, 2), (2, 2), 
        (0, 3), (1, 3), (2, 3), (3, 3)], l);
}

fn hd4(dst: &mut OffsetArray<u8, {4*UBPS+5}, {1+BPS}>) { // Horizontal-Down
    let i = dst[-1 + 0 * BPS];
    let j = dst[-1 + 1 * BPS];
    let k = dst[-1 + 2 * BPS];
    let l = dst[-1 + 3 * BPS];
    let x = dst[-1 - BPS];
    let a = dst[0 - BPS];
    let b = dst[1 - BPS];
    let c = dst[2 - BPS];

    assign(&mut dst[0..], [(0, 0), (2, 1)], avg2(i, x));
    assign(&mut dst[0..], [(0, 1), (2, 2)], avg2(j, i));
    assign(&mut dst[0..], [(0, 2), (2, 3)], avg2(k, j));
    assign(&mut dst[0..], [(0, 3)],         avg2(l, k));

    assign(&mut dst[0..], [(3, 0)],         avg3(a, b, c));
    assign(&mut dst[0..], [(2, 0)],         avg3(x, a, b));
    assign(&mut dst[0..], [(1, 0), (3, 1)], avg3(i, x, a));
    assign(&mut dst[0..], [(1, 1), (3, 2)], avg3(j, i, x));
    assign(&mut dst[0..], [(1, 2), (3, 3)], avg3(k, j, i));
    assign(&mut dst[0..], [(1, 3)],         avg3(l, k, j));
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn HD4_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    hd4(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn HU4_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    hu4(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn VL4_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    vl4(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn RD4_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    rd4(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn VR4_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    vr4(dst_arr);
}


#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn LD4_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    ld4(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn VE4_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    ve4(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DC4_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    dc4(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn HE4_C(dst: *mut u8) {
    let dst_arr = OffsetArray::<u8, {5*UBPS+1}, {BPS+1}>::from_zero_mut_ptr(dst);
    he4(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn HE16_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    he16(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DC16_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    dc16(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DC16NoTop_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    dc16_no_top(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DC16NoLeft_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    dc16_no_left(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DC16NoTopLeft_C(dst: *mut u8) {
    let dst_arr = &mut *(dst as *mut [u8; UBPS*15+16]);
    dc16_no_top_left(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn VE16_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    ve16(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TM4_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    tm4(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TM8uv_C(dst: *mut u8) {
     let dst_arr = OffsetArray::<u8, {((BPS+1)*8+1) as usize}, {BPS+1}>::from_zero_mut_ptr(dst);
    tm8uv(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TM16_C(dst: *mut u8) {
    let dst_arr = OffsetArray::<u8, {((BPS+1)*16+1) as usize}, {BPS+1}>::from_zero_mut_ptr(dst);
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
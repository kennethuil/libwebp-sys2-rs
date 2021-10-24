use std::os::raw::*;
use byteorder::{ByteOrder, LittleEndian};
use crate::dsp::{BPS, UBPS};
use crate::offsetref::{OffsetArray, OffsetSliceRefMut};
use crate::dec_clip_tables::{VP8_KABS0, VP8_KCLIP1, VP8_KSCLIP1, VP8_KSCLIP2};
use crate::array::{to_array_ref, to_array_ref_mut};



//------------------------------------------------------------------------------
// Transforms (Paragraph 14.4)
//  VP8Transform = TransformTwo_C;
//  VP8TransformDC = TransformDC_C;

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

pub(crate) fn transform_dc(r#in: i16, dst: &mut [u8; 128]) {
    let dc = (r#in + 4).into();
    for i in 0..4 {
        for j in 0..4 {
            store(dst, i, j, dc);
        }
    }
}

pub(crate) fn transform_dc_uv(r#in: &[i16; 64], dst: &mut [u8; 128+4*UBPS+4]) {
    if r#in[0 * 16] != 0 {
        transform_dc(r#in[0*16], to_array_ref_mut(dst));
    }
    if r#in[1 * 16] != 0 {
        transform_dc(r#in[1*16], to_array_ref_mut(&mut dst[4..]));
    }
    if r#in[2 * 16] != 0 {
        transform_dc(r#in[2*16], to_array_ref_mut(&mut dst[4*UBPS..]));
    }
    if r#in[3 * 16] != 0 {
        transform_dc(r#in[3*16], to_array_ref_mut(&mut dst[4*UBPS+4..]));
    }
}

pub(crate) fn transform_two(r#in: &[i16; 32], dst: &mut [u8; 132], do_two: bool) {
    transform_one(to_array_ref(r#in), to_array_ref_mut(dst));
    if do_two {
        transform_one(to_array_ref(&r#in[16..]), to_array_ref_mut(&mut dst[4..]));
    }
}

pub(crate) fn transform_uv(r#in: &[i16; 64], dst: &mut [u8; 132 + 4 * UBPS]) {
    transform_two(to_array_ref(r#in), to_array_ref_mut(dst), true);
    transform_two(to_array_ref(&r#in[32..]), to_array_ref_mut(&mut dst[4*UBPS..]), true);
}

fn transform_one(data: &[i16; 16], dst: &mut [u8; 128]) {
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
pub(crate) fn transform_ac3(r#in: &[i16; 5], dst: &mut [u8; 128]) {
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

pub(crate) fn transform_wht(r#in: &[i16; 16], out: &mut [i16; 256]) {
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

pub(crate) fn tm4(dst: &mut OffsetArray<u8, {((BPS+1)*4+1) as usize}, {BPS+1}>) {
    true_motion::<{((BPS+1)*4+1) as usize}, 4>(dst);
}

pub(crate) fn tm8uv(dst: &mut OffsetArray<u8, {((BPS+1)*8+1) as usize}, {BPS+1}>) {
    true_motion::<{((BPS+1)*8+1) as usize}, 8>(dst);
}

pub(crate) fn tm16(dst: &mut OffsetArray<u8, {((BPS+1)*16+1) as usize}, {BPS+1}>) {
    true_motion::<{((BPS+1)*16+1) as usize}, 16>(dst);
}

//------------------------------------------------------------------------------
// 16x16

pub(crate) fn ve16(dst: &mut OffsetArray<u8, {UBPS*16+16}, {BPS}>) { // vertical
    let (src, dst) = dst.split_at_mut(0);
    let src = &src[0..16];
    for dst_line in dst.chunks_exact_mut(UBPS/2).step_by(2) {
        dst_line.copy_from_slice(src);
    }
}

pub(crate) fn he16(dst: &mut OffsetArray<u8, {(BPS*15+17) as usize}, 1>) { // horizontal
    // TODO: Do we really get full BPS-sized chunks or not?
    for chunk in dst.chunks_mut(BPS as usize) {
        let v = chunk[0];
        chunk[1..17].fill(v);
    }
}

pub(crate) fn put16(v: u8, dst: &mut [u8; 15*UBPS+16]) {
    for chunk in dst.chunks_exact_mut(UBPS/2).step_by(2) {
        chunk.fill(v);
    }
}

pub(crate) fn dc16(dst: &mut OffsetArray<u8, {UBPS*16+16}, {BPS}>) {  // DC
    let mut dc:u32 = 16;
    for j in 0..16 {
        let first = dst[-1 + j * BPS] as u32;
        let second = dst[j-BPS] as u32;
        let sum = first + second;
        dc = dc.wrapping_add(sum);
    }

    put16(((dc >> 5) & 0xff) as u8, to_array_ref_mut(&mut dst[0..]));
}

pub(crate) fn dc16_no_top(dst: &mut OffsetArray<u8, {UBPS*15+17}, 1>) {  // DC with top samples not available
    let mut dc:u32 = 8;
    for j in 0..16 {
        dc = dc.wrapping_add(dst[-1 + j * BPS] as u32);
    }
    put16(((dc >> 4) & 0xff) as u8, to_array_ref_mut(&mut dst[0..]));
}

pub(crate) fn dc16_no_left(dst: &mut OffsetArray<u8, {UBPS*16+16}, {BPS}>) {  // DC with left samples not available
    let mut dc:u32 = 8;
    for i in 0..16 {
        dc = dc.wrapping_add(dst[i - BPS] as u32);
    }
    put16(((dc >> 4) & 0xff) as u8, to_array_ref_mut(&mut dst[0..]));
}

pub(crate) fn dc16_no_top_left(dst: &mut [u8; UBPS*15+16]) {  // DC with no top and left samples
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

pub(crate) fn ve4(dst: &mut OffsetArray<u8, {5*UBPS+1}, {BPS+1}>) { // vertical
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
pub(crate) fn he4(dst: &mut OffsetArray<u8, {5*UBPS+1}, {BPS+1}>) {  // horizontal
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


pub(crate) fn dc4(dst: &mut OffsetArray<u8, {5*UBPS}, {BPS}>) { // DC
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

pub(crate) fn rd4(dst: &mut OffsetArray<u8, {4*UBPS+5}, {1+BPS}>) { // Down-right
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

pub(crate) fn ld4(dst: &mut OffsetArray<u8, {4*UBPS+4}, {BPS}>) { // Down-Left
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

pub(crate) fn vr4(dst: &mut OffsetArray<u8, {4*UBPS+5}, {1+BPS}>) { // Vertical-Right
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

pub(crate) fn vl4(dst: &mut OffsetArray<u8, {4*UBPS+4}, {BPS}>) {  // Vertical-Left
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

pub(crate) fn hu4(dst: &mut OffsetArray<u8, {4*UBPS+5}, {1+BPS}>) { // Horizontal-Up
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

pub(crate) fn hd4(dst: &mut OffsetArray<u8, {4*UBPS+5}, {1+BPS}>) { // Horizontal-Down
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

//------------------------------------------------------------------------------
// Chroma

pub(crate) fn ve8_uv(dst: &mut OffsetArray<u8, {8*UBPS+8}, {BPS}>) {    // vertical
    let (src, dst) = dst.split_at_mut(0);
    let src = &src[0..8];
    for chunk in dst.chunks_exact_mut(8).step_by(4) {
        chunk.copy_from_slice(src);
    }
}

pub(crate) fn he8_uv(dst: &mut OffsetArray<u8, {7*UBPS+9},1>) {    // horizontal
    for chunk in dst.chunks_exact_mut(UBPS) {
        let v = chunk[0];
        chunk[1..9].fill(v);
    }
    let v = dst[7*BPS-1];
    dst[7*BPS..].fill(v);
}

fn put_8x8_uv(v: u8, dst: &mut [u8; UBPS*7+8]) {
    for chunk in dst.chunks_exact_mut(8).step_by(4) {
        chunk.fill(v);
    }
}

pub(crate) fn dc8_uv(dst: &mut OffsetArray<u8, {UBPS*8+8}, BPS>) {  // DC
    let mut dc0 = 8;
    for i in 0..8 {
        dc0 += (dst[i-BPS] as u32) + (dst[-1 + i * BPS] as u32);
    }
    put_8x8_uv((dc0 >> 4) as u8, to_array_ref_mut(&mut dst[0..]));
}

pub(crate) fn dc8_uv_no_left(dst: &mut OffsetArray<u8, {UBPS*8+8}, BPS>) {   // DC with no left samples
    let mut dc0 = 4;
    for i in 0..8 {
        dc0 += dst[i - BPS] as u32;
    }
    put_8x8_uv((dc0 >> 3) as u8, to_array_ref_mut(&mut dst[0..]));
}

pub(crate) fn dc8_uv_no_top(dst: &mut OffsetArray<u8, {UBPS*7+9}, 1>) {  // DC with no top samples
    let mut dc0 = 4;
    for i in 0..8 {
        dc0 += dst[-1 + i * BPS] as u32;
    }
    put_8x8_uv((dc0 >> 3) as u8, to_array_ref_mut(&mut dst[0..]));
}

pub(crate) fn dc8_uv_no_top_left(dst: &mut [u8; UBPS*7+8]) {  // DC with nothing
    put_8x8_uv(0x80, dst);
}

//------------------------------------------------------------------------------
// Edge filtering functions

// 4 pixels in, 2 pixels out
// p: (-2*step)..(step+1)
fn do_filter_2(p: &mut OffsetSliceRefMut<u8>, step: isize) {
    let p1 = p[-2*step] as isize;
    let p0 = p[-step] as isize;
    let q0 = p[0] as isize;
    let q1 = p[step] as isize;
    let a = 3 * (q0 - p0) + VP8_KSCLIP1[p1 - q1] as isize; // in [-893,892]
    let a1 = VP8_KSCLIP2[(a + 4) >> 3] as isize;           // in [-16,15]
    let a2 = VP8_KSCLIP2[(a + 3) >> 3] as isize;
    p[-step] = VP8_KCLIP1[p0 + a2];
    p[    0] = VP8_KCLIP1[q0 - a1];
}

// 4 pixels in, 4 pixels out
// p: (-2*step)..(step+1)
fn do_filter_4(p: &mut OffsetSliceRefMut<u8>, step: isize) {
    let p1 = p[-2*step] as isize;
    let p0 = p[-step] as isize;
    let q0 = p[0] as isize;
    let q1 = p[step] as isize;
    let a = 3 * (q0 - p0);
    let a1 = VP8_KSCLIP2[(a + 4) >> 3] as isize;
    let a2 = VP8_KSCLIP2[(a + 3) >> 3] as isize;
    let a3 = (a1 + 1) >> 1;
    p[-2*step] = VP8_KCLIP1[p1 + a3];
    p[-  step] = VP8_KCLIP1[p0 + a2];
    p[      0] = VP8_KCLIP1[q0 - a1];
    p[   step] = VP8_KCLIP1[q1 - a3];
}

// 6 pixels in, 6 pixels out
// p: (-3*step)..(2*step+1)
fn do_filter_6(p: &mut OffsetSliceRefMut<u8>, step: isize) {
    let p2 = p[-3*step] as isize;
    let p1 = p[-2*step] as isize;
    let p0 = p[-step] as isize;
    let q0 = p[0] as isize;
    let q1 = p[step] as isize;
    let q2 = p[2*step] as isize;
    let a = VP8_KSCLIP1[3 * (q0 - p0) + (VP8_KSCLIP1[p1 - q1] as isize)] as isize;
    // a is in [-128,127], a1 in [-27,27], a2 in [-18,18] and a3 in [-9,9]
    let a1 = (27 * a + 63) >> 7;  // eq. to ((3 * a + 7) * 9) >> 7
    let a2 = (18 * a + 63) >> 7;  // eq. to ((2 * a + 7) * 9) >> 7
    let a3 = ( 9 * a + 63) >> 7;  // eq. to ((1 * a + 7) * 9) >> 7
    p[-3*step] = VP8_KCLIP1[p2 + a3];
    p[-2*step] = VP8_KCLIP1[p1 + a2];
    p[-  step] = VP8_KCLIP1[p0 + a1];
    p[      0] = VP8_KCLIP1[q0 - a1];
    p[   step] = VP8_KCLIP1[q1 - a2];
    p[ 2*step] = VP8_KCLIP1[q2 - a3];
}

// p: (-2*step)..(step+1)
fn hev(p: &mut OffsetSliceRefMut<u8>, step: isize, thresh: u8) -> bool {
    let p1 = p[-2*step] as isize;
    let p0 = p[-step] as isize;
    let q0 = p[0] as isize;
    let q1 = p[step] as isize;
    VP8_KABS0[p1 - p0] > thresh || VP8_KABS0[q1 - q0] > thresh
}

// p: (-2*step)..(step+1)
fn needs_filter(p: &mut OffsetSliceRefMut<u8>, step: isize, t: u32) -> bool {
    let p1 = p[-2 * step] as isize;
    let p0 = p[-step] as isize;
    let q0 = p[0] as isize;
    let q1 = p[step] as isize;
    (4 * (VP8_KABS0[p0 - q0] as u32) + (VP8_KABS0[p1 - q1] as u32)) <= t
}

// p: (-4*step)..(3*step+1)
fn needs_filter_2(p: &mut OffsetSliceRefMut<u8>, step: isize, t: u32, it: u8) -> bool {
    let p3 = p[-4 * step] as isize;
    let p2 = p[-3 * step] as isize;
    let p1 = p[-2 * step] as isize;
    let p0 = p[-step] as isize;
    let q0 = p[0] as isize;
    let q1 = p[step] as isize;
    let q2 = p[2 * step] as isize;
    let q3 = p[3 * step] as isize;
    if (4 * (VP8_KABS0[p0 - q0] as u32) + (VP8_KABS0[p1 - q1] as u32)) > t {
        return false;
    }
    VP8_KABS0[p3 - p2] <= it && VP8_KABS0[p2 - p1] <= it &&
        VP8_KABS0[p1 - p0] <= it && VP8_KABS0[q3 - q2] <= it &&
        VP8_KABS0[q2 - q1] <= it && VP8_KABS0[q1 - q0] <= it
}

//------------------------------------------------------------------------------
// Simple In-loop filtering (Paragraph 15.2)

// p: (-2*stride)..(stride+16)
pub(crate) fn simple_v_filter_16(p: &mut OffsetSliceRefMut<u8>, stride: isize, thresh: u32) {
    let thresh2 = 2 * thresh + 1;
    for i in 0..16 {
        if needs_filter(&mut (p.with_offset(i)), stride, thresh2) {
            do_filter_2(&mut (p.with_offset(i)), stride);
        }
    }
}

// p: (2*stride)..(13*stride+16)
pub(crate) fn simple_v_filter_16i(p: &mut OffsetSliceRefMut<u8>, stride: isize, thresh: u32) {
    let mut p = p.with_offset(0);

    for _ in 0..3 {
        p.move_zero(4 * stride);
        simple_v_filter_16(&mut p, stride, thresh);
    }
}

// p: (-2)..(15*stride+2)
pub(crate) fn simple_h_filter_16(p: &mut OffsetSliceRefMut<u8>, stride: isize, thresh: u32) {
    let thresh2 = 2 * thresh + 1;
    for i in 0..16 {
        if needs_filter(&mut (p.with_offset(i * stride)), 1, thresh2) {
            do_filter_2(&mut (p.with_offset(i * stride)), 1);
        }
    }
}

// p: 4..(15*stride+12)
pub(crate) fn simple_h_filter_16i(p: &mut OffsetSliceRefMut<u8>, stride: isize, thresh: u32) {
    let mut p = p.with_offset(0);

    for _ in 0..3 {
        p.move_zero(4);
        simple_h_filter_16(&mut p, stride, thresh);
    }
}

//------------------------------------------------------------------------------
// Complex In-loop filtering (Paragraph 15.3)

// p: (-4*hstride)..((3*hstride) + vstride*(size-1))
fn filter_loop_26(p: &mut OffsetSliceRefMut<u8>,
                hstride: isize, vstride: isize, size: u32,
                thresh: u32, ithresh: u8, hev_thresh:u8) {
    let thresh2 = 2 * thresh + 1;
    for _ in 0..size {
        // p: (-4*step)..(3*step+1)
        if needs_filter_2(p, hstride, thresh2, ithresh) {
            // p: (-2*step)..(step+1)
            if hev(p, hstride, hev_thresh) {
                // p: (-2*step)..(step+1)
                do_filter_2(p, hstride);
            } else {
                // p: (-3*step)..(2*step+1)
                do_filter_6(p, hstride);
            }
        }
        p.move_zero(vstride);
    }
}

// p: (-4*hstride)..((3*hstride) + vstride*(size-1))
fn filter_loop_24(p: &mut OffsetSliceRefMut<u8>,
                hstride: isize, vstride: isize, size: u32,
                thresh: u32, ithresh: u8, hev_thresh:u8) {
    let thresh2 = 2 * thresh + 1;
    for _ in 0..size {
        // p: (-4*step)..(3*step+1)
        if needs_filter_2(p, hstride, thresh2, ithresh) {
            // p: (-2*step)..(step+1)
            if hev(p, hstride, hev_thresh) {
                // p: (-2*step)..(step+1)
                do_filter_2(p, hstride);
            } else {
                // p: (-2*step)..(step+1)
                do_filter_4(p, hstride);
            }
        }
        p.move_zero(vstride);
    }
}

// on macroblock edges
// p: (-4*stride)..(3*stride+16)
pub(crate) fn v_filter_16(p: &mut OffsetSliceRefMut<u8>, stride: isize, thresh: u32, ithresh: u8, hev_thresh: u8) {
    filter_loop_26(p, stride, 1, 16, thresh, ithresh, hev_thresh);
}

// p: -4..4+stride*15
pub(crate) fn h_filter_16(p: &mut OffsetSliceRefMut<u8>, stride: isize, thresh: u32, ithresh: u8, hev_thresh: u8) {
    filter_loop_26(p, 1, stride, 16, thresh, ithresh, hev_thresh);
}

// on three inner edges
// p: 0..15*stride+16
pub(crate) fn v_filter_16i(p: &mut OffsetSliceRefMut<u8>, stride: isize, thresh: u32, ithresh: u8, hev_thresh: u8) {
    for k in 1..4 {
        let mut new_p = p.with_offset(k * 4 * stride);
        filter_loop_24(&mut new_p, stride, 1, 16, thresh, ithresh, hev_thresh);
    }
}

// p: 0..16+stride*15
pub(crate) fn h_filter_16i(p: &mut OffsetSliceRefMut<u8>, stride: isize, thresh: u32, ithresh: u8, hev_thresh: u8) {
    for k in 1..4 {
        let mut new_p = p.with_offset(k * 4);
        filter_loop_24(&mut new_p, 1, stride, 16, thresh, ithresh, hev_thresh);
    }
}

// 8-pixels wide variant, for chroma filtering
// u, v: (-4*stride)..(3*stride + 8)
pub(crate) fn v_filter_8(u: &mut OffsetSliceRefMut<u8>, v: &mut OffsetSliceRefMut<u8>, stride: isize,
                thresh: u32, ithresh: u8, hev_thresh: u8) {
    filter_loop_26(u, stride, 1, 8, thresh, ithresh, hev_thresh);
    filter_loop_26(v, stride, 1, 8, thresh, ithresh, hev_thresh);
}

// u, v: (-4)..stride*7+4
pub(crate) fn h_filter_8(u: &mut OffsetSliceRefMut<u8>, v: &mut OffsetSliceRefMut<u8>, stride: isize,
                thresh: u32, ithresh: u8, hev_thresh: u8) {
    filter_loop_26(u, 1, stride, 8, thresh, ithresh, hev_thresh);
    filter_loop_26(v, 1, stride, 8, thresh, ithresh, hev_thresh);
}

// u, v: 0..(7*stride+8)
pub(crate) fn v_filter_8i(u: &mut OffsetSliceRefMut<u8>, v: &mut OffsetSliceRefMut<u8>, stride: isize,
    thresh: u32, ithresh: u8, hev_thresh: u8) {
    filter_loop_24(&mut (u.with_offset(4*stride)), stride, 1, 8, thresh, ithresh, hev_thresh);
    filter_loop_24(&mut (v.with_offset(4*stride)), stride, 1, 8, thresh, ithresh, hev_thresh);
}

// u, v: 0..stride*7+8
pub(crate) fn h_filter_8i(u: &mut OffsetSliceRefMut<u8>, v: &mut OffsetSliceRefMut<u8>, stride: isize,
    thresh: u32, ithresh: u8, hev_thresh: u8) {
    filter_loop_24(&mut (u.with_offset(4)), 1, stride, 8, thresh, ithresh, hev_thresh);
    filter_loop_24(&mut (v.with_offset(4)), 1, stride, 8, thresh, ithresh, hev_thresh);
}

const VP8_DITHER_AMP_CENTER: i32 = 1 << 7;
const VP8_DITHER_DESCALE_ROUNDER: i32 = 1 << (4 - 1);
const VP8_DITHER_DESCALE: i32 = 4;

pub(crate) fn dither_combine_8x8(dither: &[u8; 64], mut dst: &mut [u8], dst_stride: usize) {
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

//------------------------------------------------------------------------------------------
// Temporary extern wrappers

// p: 0..16+stride*15
#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn HFilter16i_C(p: *mut u8, stride: c_int, thresh: c_uint, ithresh: c_uint,
    hev_thresh: c_uint) {
    let mut p_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, 0 as isize,
        (15*stride+16) as isize);
    h_filter_16i(&mut p_arr, stride as isize, thresh, ithresh as u8, hev_thresh as u8);        
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn VFilter16i_C(p: *mut u8, stride: c_int, thresh: c_uint, ithresh: c_uint,
    hev_thresh: c_uint) {
    let mut p_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, 0 as isize,
        (15*stride+16) as isize);
    v_filter_16i(&mut p_arr, stride as isize, thresh, ithresh as u8, hev_thresh as u8);        
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn HFilter16_C(p: *mut u8, stride: c_int, thresh: c_uint, ithresh: c_uint,
    hev_thresh: c_uint) {
    let mut p_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -4,
        (4+stride*15) as isize);
    h_filter_16(&mut p_arr, stride as isize, thresh, ithresh as u8, hev_thresh as u8);        
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn VFilter16_C(p: *mut u8, stride: c_int, thresh: c_uint, ithresh: c_uint,
    hev_thresh: c_uint) {
    let mut p_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, (-4*stride) as isize,
        (3*stride+16) as isize);
    v_filter_16(&mut p_arr, stride as isize, thresh, ithresh as u8, hev_thresh as u8);        
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn HFilter8i_C(u: *mut u8, v: *mut u8, stride: c_int, thresh: c_uint,
    ithresh: c_uint, hev_thresh: c_uint) {
    let mut u_arr = OffsetSliceRefMut::from_zero_mut_ptr(u, 0, 
        (stride*7+8) as isize);
    let mut v_arr = OffsetSliceRefMut::from_zero_mut_ptr(v,  0, 
        (stride*7+8) as isize);

    let u_range = u_arr.as_ptr_range();
    let v_range = v_arr.as_ptr_range();
    let overlaps = u_range.contains(&v_range.start) || v_range.contains(&u_range.start);
    assert!(!overlaps);

    h_filter_8i(&mut u_arr, &mut v_arr, stride as isize, thresh, ithresh as u8, hev_thresh as u8);
}


#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn VFilter8i_C(u: *mut u8, v: *mut u8, stride: c_int, thresh: c_uint,
    ithresh: c_uint, hev_thresh: c_uint) {
    let mut u_arr = OffsetSliceRefMut::from_zero_mut_ptr(u, 0, 
        (7*stride + 8) as isize);
    let mut v_arr = OffsetSliceRefMut::from_zero_mut_ptr(v,  0, 
        (7*stride + 8) as isize);

    let u_range = u_arr.as_ptr_range();
    let v_range = v_arr.as_ptr_range();
    let overlaps = u_range.contains(&v_range.start) || v_range.contains(&u_range.start);
    assert!(!overlaps);

    v_filter_8i(&mut u_arr, &mut v_arr, stride as isize, thresh, ithresh as u8, hev_thresh as u8);
}


#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn HFilter8_C(u: *mut u8, v: *mut u8, stride: c_int, thresh: c_uint,
    ithresh: c_uint, hev_thresh: c_uint) {
    let mut u_arr = OffsetSliceRefMut::from_zero_mut_ptr(u, -4, 
        (stride*7+4) as isize);
    let mut v_arr = OffsetSliceRefMut::from_zero_mut_ptr(v,  -4, 
        (stride*7+4) as isize);

    let u_range = u_arr.as_ptr_range();
    let v_range = v_arr.as_ptr_range();
    let overlaps = u_range.contains(&v_range.start) || v_range.contains(&u_range.start);
    assert!(!overlaps);

    h_filter_8(&mut u_arr, &mut v_arr, stride as isize, thresh, ithresh as u8, hev_thresh as u8);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn VFilter8_C(u: *mut u8, v: *mut u8, stride: c_int, thresh: c_uint,
    ithresh: c_uint, hev_thresh: c_uint) {
    let mut u_arr = OffsetSliceRefMut::from_zero_mut_ptr(u, -4*stride as isize, 
        (3*stride + 8) as isize);
    let mut v_arr = OffsetSliceRefMut::from_zero_mut_ptr(v,  -4*stride as isize, 
        (3*stride + 8) as isize);

    let u_range = u_arr.as_ptr_range();
    let v_range = v_arr.as_ptr_range();
    let overlaps = u_range.contains(&v_range.start) || v_range.contains(&u_range.start);
    assert!(!overlaps);

    v_filter_8(&mut u_arr, &mut v_arr, stride as isize, thresh, ithresh as u8, hev_thresh as u8);
}


#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn FilterLoop26_C(p: *mut u8,
                                    hstride: c_int, vstride: c_int, size: c_uint,
                                    thresh: c_uint, ithresh: c_uint, hev_thresh: c_uint) {
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -4*hstride as isize, 
        (3*hstride + vstride*(size as i32)) as isize);
    filter_loop_26(&mut dst_arr, hstride as isize, vstride as isize, 
        size, thresh, ithresh as u8, hev_thresh as u8);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn FilterLoop24_C(p: *mut u8,
                                    hstride: c_int, vstride: c_int, size: c_uint,
                                    thresh: c_uint, ithresh: c_uint, hev_thresh: c_uint) {
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -4*hstride as isize, 
        (3*hstride + vstride*(size as i32)) as isize);
    filter_loop_24(&mut dst_arr, hstride as isize, vstride as isize, 
        size, thresh, ithresh as u8, hev_thresh as u8);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn SimpleVFilter16i_C(p: *mut u8, stride: c_int, thresh: c_uint) {
    let stride = stride as isize;
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, 2*stride, 
        13*stride+16);
    simple_v_filter_16i(&mut dst_arr, stride, thresh);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn SimpleHFilter16_C(p: *mut u8, stride: c_int, thresh: c_uint) {
    let stride = stride as isize;
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -2, 
        15*stride+2);
    simple_h_filter_16(&mut dst_arr, stride, thresh);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn SimpleHFilter16i_C(p: *mut u8, stride: c_int, thresh: c_uint) {
    let stride = stride as isize;
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, 2, 
        15*stride+14);
    simple_h_filter_16i(&mut dst_arr, stride, thresh);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn SimpleVFilter16_C(p: *mut u8, stride: c_int, thresh: c_uint) {
    let stride = stride as isize;
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -2*stride, 
        stride+16);
    simple_v_filter_16(&mut dst_arr, stride, thresh);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn NeedsFilter2_C(p: *mut u8, step: c_int, t: c_uint, it: c_uint) -> c_int {
    let step = step as isize;
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -4*step, 3*step+1);
    if needs_filter_2(&mut dst_arr, step, t as u32, it as u8) {
        1
    } else {
        0
    }
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn NeedsFilter_C(p: *mut u8, step: c_int, t: c_uint) -> c_int {
    let step = step as isize;
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -2*step, step+1);
    if needs_filter(&mut dst_arr, step, t as u32) {
        1
    } else {
        0
    }
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn Hev(p: *mut u8, step: c_int, thresh: c_int) -> c_int {
    let step = step as isize;
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -2*step, step+1);
    if hev(&mut dst_arr, step, thresh as u8) {
        1
    } else {
        0
    }
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DoFilter6_C(p: *mut u8, step: c_int) {
    let step = step as isize;
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -3*step, 2*step+1);
    do_filter_6(&mut dst_arr, step);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DoFilter4_C(p: *mut u8, step: c_int) {
    let step = step as isize;
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -2*step, step+1);
    do_filter_4(&mut dst_arr, step);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DoFilter2_C(p: *mut u8, step: c_int) {
    let step = step as isize;
    let mut dst_arr = OffsetSliceRefMut::from_zero_mut_ptr(p, -2*step, step+1);
    do_filter_2(&mut dst_arr, step);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DC8uvNoTopLeft_C(dst: *mut u8) {
    let dst_arr = &mut *(dst as *mut [u8; UBPS*7+8]);
    dc8_uv_no_top_left(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DC8uvNoTop_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    dc8_uv_no_top(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DC8uvNoLeft_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    dc8_uv_no_left(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn DC8uv_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    dc8_uv(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn HE8uv_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    he8_uv(dst_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn VE8uv_C(dst: *mut u8) {
    let dst_arr = OffsetArray::from_zero_mut_ptr(dst);
    ve8_uv(dst_arr);
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
    transform_one(input_arr, output_arr);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TransformTwo_C(data: *const i16, dst: *mut u8, do_two: c_int) {

    let input_arr = &*(data as *const[i16; 32]);
    let output_arr = &mut *(dst as *mut[u8; 132]);
    transform_two(input_arr, output_arr, do_two != 0);
}

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
unsafe extern "C" fn TransformUV_C(data: *const i16, dst: *mut u8) {

    let input_arr = &*(data as *const[i16; 64]);
    let output_arr = &mut *(dst as *mut[u8; 260]);
    transform_uv(input_arr, output_arr);
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
unsafe extern "C" fn TransformDCUV_C(r#in: *const i16, dst: *mut u8) {
    let input_arr = &*(r#in as *const[i16; 64]);
    let output_arr = &mut *(dst as *mut[u8; 260]);
    transform_dc_uv(input_arr, output_arr);
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


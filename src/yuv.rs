use std::convert::TryInto;

//------------------------------------------------------------------------------
// YUV -> RGB conversion
const YUV_FIX: i32 = 16;                    // fixed-point precision for RGB->YUV
const YUV_HALF: i32 = 1 << (YUV_FIX - 1);

const YUV_FIX2: i32 = 6;                    // fixed-point precision for YUV->RGB
const YUV_MASK2: i32 = (256 << YUV_FIX2) - 1;

fn mult_hi(v: i32, coeff: i32) -> i32 {
    (v * coeff) >> 8
}

fn vp8_clip8(v: i32) -> u8 {
    if v & !YUV_MASK2 == 0 {
        ((v >> YUV_FIX2) & 0xff) as u8
    } else if v < 0 {
        0
    } else {
        255
    }
}

fn vp8_yuv_to_r(y: u8, v: u8) -> u8 {
    vp8_clip8(mult_hi(y as i32, 19077) + mult_hi(v as i32, 26149) - 14234)
}

fn vp8_yuv_to_g(y: u8, u: u8, v: u8) -> u8 {
    vp8_clip8(mult_hi(y as i32, 19077) - mult_hi(u as i32, 6419) - mult_hi(v as i32, 13320)
    + 8708)
}

fn vp8_yuv_to_b(y: u8, u: u8) -> u8 {
    vp8_clip8(mult_hi(y as i32, 19077) + mult_hi(u as i32, 33050) - 17685)
}

pub(crate) fn vp8_yuv_to_rgb(y: u8, u: u8, v: u8, rgb: &mut [u8; 3]) {
    rgb[0] = vp8_yuv_to_r(y, v);
    rgb[1] = vp8_yuv_to_g(y, u, v);
    rgb[2] = vp8_yuv_to_b(y, u);
}

pub(crate) fn vp8_yuv_to_bgr(y: u8, u: u8, v: u8, bgr: &mut [u8; 3]) {
    bgr[0] = vp8_yuv_to_b(y, u);
    bgr[1] = vp8_yuv_to_g(y, u, v);
    bgr[2] = vp8_yuv_to_r(y, v);
}

pub(crate) fn vp8_yuv_to_rgb_565(y: u8, u: u8, v: u8, out: &mut [u8; 2]) {
    let r = vp8_yuv_to_r(y, v);     // 5 usable bits
    let g = vp8_yuv_to_g(y, u, v);  // 6 usable bits
    let b = vp8_yuv_to_b(y, u);     // 5 usable bits
    let rg = (r & 0xf8) | (g >> 5);
    let gb = ((g << 3) & 0xe0) | (b >> 3);
    out[0] = rg;
    out[1] = gb;
}

pub(crate) fn vp8_yuv_to_rgba_4444(y: u8, u: u8, v: u8, out: &mut[u8; 2]) {
    let r = vp8_yuv_to_r(y, v);     // 4 usable bits
    let g = vp8_yuv_to_g(y, u, v);  // 4 usable bits
    let b = vp8_yuv_to_b(y, u);     // 4 usable bits
    let rg = (r & 0xf0) | (g >> 4);
    let ba = (b & 0xf0) | 0x0f;     // overwrite the lower 4 bits
    out[0] = rg;
    out[1] = ba;
}

// ...

//-----------------------------------------------------------------------------
// Alpha handling variants

pub(crate) fn vp8_yuv_to_rgba(y: u8, u: u8, v: u8, rgba: &mut [u8; 4]) {
    vp8_yuv_to_rgb(y, u, v, (&mut rgba[0..3]).try_into().unwrap());
    rgba[3] = 0xff;
}

pub(crate) fn vp8_yuv_to_bgra(y: u8, u: u8, v: u8, bgra: &mut [u8; 4]) {
    vp8_yuv_to_bgr(y, u, v, (&mut bgra[0..3]).try_into().unwrap());
    bgra[3] = 0xff;
}

pub(crate) fn vp8_yuv_to_argb(y: u8, u: u8, v: u8, argb: &mut [u8; 4]) {
    argb[0] = 0xff;
    vp8_yuv_to_rgb(y, u, v, (&mut argb[1..]).try_into().unwrap());
}

// ...
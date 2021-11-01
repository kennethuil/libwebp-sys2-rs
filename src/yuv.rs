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

// ...

//-----------------------------------------------------------------------------
// Alpha handling variants

pub(crate) fn vp8_yuv_to_rgba(y: u8, u: u8, v: u8, rgba: &mut [u8; 4]) {
    vp8_yuv_to_rgb(y, u, v, (&mut rgba[0..3]).try_into().unwrap());
    rgba[3] = 0xff;
}

// ...
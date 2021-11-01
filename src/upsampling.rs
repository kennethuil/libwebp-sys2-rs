use core::slice;
use std::{convert::{TryFrom, TryInto}, fmt::Debug, ptr::{slice_from_raw_parts, slice_from_raw_parts_mut}};

use libc::c_uint;

use crate::{MODE_ARGB, MODE_Argb, MODE_BGR, MODE_BGRA, MODE_RGB, MODE_RGBA, MODE_RGBA_4444, MODE_RGB_565, MODE_bgrA, MODE_rgbA, MODE_rgbA_4444, yuv::vp8_yuv_to_rgba};

// Given samples laid out in a square as:
//  [a b]
//  [c d]
// we interpolate u/v as:
//  ([9*a + 3*b + 3*c +   d    3*a + 9*b + 3*c +   d] + [8 8]) / 16
//  ([3*a +   b + 9*c + 3*d      a + 3*b + 3*c + 9*d]   [8 8]) / 16

pub(crate) trait FancyUpsampler {
    // min_const_generics doesn't let you accept &mut [u8; Self::XSTEP] as a parameter :(
    // TODO: Try that again when the next chunk of const generics lands in stable, that way we can
    // ditch UpsampleDest and have upsample directly take a &mut [u8; XSTEP]
    type UpsampleDest;
    const XSTEP: usize = std::mem::size_of::<Self::UpsampleDest>();

    fn upsample(y: u8, u: u8, v: u8, out: &mut Self::UpsampleDest);

    fn upsample_line_pair(&self, top_y: &[u8], bottom: Option<(&[u8], &mut [u8])>,
        top_u: &[u8], top_v: &[u8],
        cur_u: &[u8], cur_v: &[u8],
        top_dst: &mut [u8], len: usize)   where for <'a> &'a mut <Self as FancyUpsampler>::UpsampleDest: TryFrom<&'a mut [u8]> {
        Self::do_upsample_line_pair(top_y, bottom, top_u, top_v, cur_u, cur_v,
            top_dst, len);
    }

    // We process u and v together stashed into 32bit (16bit each).
    fn load_uv(u: u8, v: u8) -> u32 {
        (u as u32) | ((v as u32) << 16)
    }

    fn do_upsample_line_pair(top_y: &[u8], mut bottom: Option<(&[u8], &mut [u8])>,
        top_u: &[u8], top_v: &[u8],
        cur_u: &[u8], cur_v: &[u8],
        top_dst: &mut [u8], len: usize)  where for <'a> &'a mut <Self as FancyUpsampler>::UpsampleDest: TryFrom<&'a mut [u8]> {
        

        let last_pixel_pair = (len - 1) >> 1;
        let mut tl_uv = Self::load_uv(top_u[0], top_v[0]);  // top-left sample
        let mut l_uv = Self::load_uv(cur_u[0], cur_v[0]);   // left-sample

        {
            let uv0 = (3 * tl_uv + l_uv + 0x00020002) >> 2;
            let dst = (&mut top_dst[..Self::XSTEP]).try_into().unwrap_or_else(|_| panic!("nope"));
            Self::upsample(top_y[0], (uv0 & 0xff) as u8, (uv0 >> 16) as u8, dst);
        }
      
        bottom.as_mut().and_then(|(bottom_y, bottom_dst)| {
            let uv0 = (3 * l_uv + tl_uv + 0x00020002) >> 2;
            Self::upsample(bottom_y[0], (uv0 & 0xff) as u8, (uv0 >> 16) as u8, 
            (&mut bottom_dst[..Self::XSTEP]).try_into().unwrap_or_else(|_| panic!("nope")));
            Some(())        
        });
        
        for x in 1..=last_pixel_pair {
            let t_uv = Self::load_uv(top_u[x], top_v[x]);     // top sample
            let uv = Self::load_uv(cur_u[x], cur_v[x]);       // sample 
            // precompute invariant values associated with first and second diagonals
            let avg = tl_uv + t_uv + l_uv + uv + 0x00080008;
            let diag_12 = (avg + 2 * (t_uv + l_uv)) >> 3;
            let diag_03 = (avg + 2 * (tl_uv + uv)) >> 3;
            {
                let uv0 = (diag_12 + tl_uv) >> 1;
                let uv1 = (diag_03 + t_uv) >> 1;
                Self::upsample(top_y[2 * x - 1], (uv0 & 0xff) as u8, (uv0 >> 16) as u8,
                    (&mut top_dst[(2 * x - 1)*Self::XSTEP..][..Self::XSTEP]).try_into().unwrap_or_else(|_| panic!("nope")));
                Self::upsample(top_y[2 * x - 0], (uv1 & 0xff) as u8, (uv1 >> 16) as u8,
                (&mut top_dst[(2 * x - 0)*Self::XSTEP..][..Self::XSTEP]).try_into().unwrap_or_else(|_| panic!("nope")));              
            }
            bottom.as_mut().and_then(|(bottom_y, bottom_dst)| {
                let uv0 = (diag_03 + l_uv) >> 1;
                let uv1 = (diag_12 + uv) >> 1;
                Self::upsample(bottom_y[2 * x - 1], (uv0 & 0xff) as u8, (uv0 >> 16) as u8,
                    (&mut bottom_dst[(2 * x - 1)*Self::XSTEP..][..Self::XSTEP]).try_into().unwrap_or_else(|_| panic!("nope")));
                Self::upsample(bottom_y[2 * x + 0], (uv1 & 0xff) as u8, (uv1 >> 16) as u8,
                (&mut bottom_dst[(2 * x + 0)*Self::XSTEP..][..Self::XSTEP]).try_into().unwrap_or_else(|_| panic!("nope")));              
                Some(())
            });
            tl_uv = t_uv;
            l_uv = uv;
        }
        
        if len & 1 == 0 {
            {
                let uv0 = (3 * tl_uv + l_uv + 0x00020002) >> 2;
                Self::upsample(top_y[len - 1], (uv0 & 0xff) as u8, (uv0 >> 16) as u8,
                    (&mut top_dst[(len - 1)*Self::XSTEP..][..Self::XSTEP]).try_into().unwrap_or_else(|_| panic!("nope")));
            }
            bottom.and_then(|(bottom_y, bottom_dst)| {
                let uv0 = (3 * l_uv + tl_uv + 0x00020002) >> 2;
                Self::upsample(bottom_y[len - 1], (uv0 & 0xff) as u8, (uv0 >> 16) as u8,
                (&mut bottom_dst[(len - 1)*Self::XSTEP..][..Self::XSTEP]).try_into().unwrap_or_else(|_| panic!("nope")));              
                Some(())
            });
        }
    }

    unsafe extern "C" fn ffi_upsample_line_pair(top_y: *const u8, bottom_y: *const u8,
        top_u: *const u8, top_v: *const u8, cur_u: *const u8, cur_v: *const u8,
        top_dst: *mut u8, bottom_dst: *mut u8, len: c_uint) where for <'a> &'a mut <Self as FancyUpsampler>::UpsampleDest: TryFrom<&'a mut [u8]> {
        let s_len = len as usize;
        assert_ne!(s_len, 0);
        let last_pixel_pair = (s_len - 1) >> 1;

        let s_top_y = slice::from_raw_parts(top_y, s_len);
        let s_top_u = slice::from_raw_parts(top_u, last_pixel_pair + 1);
        let s_top_v = slice::from_raw_parts(top_v, last_pixel_pair + 1);
        let s_cur_u = slice::from_raw_parts(cur_u, last_pixel_pair + 1);
        let s_cur_v = slice::from_raw_parts(cur_v, last_pixel_pair + 1);
        let s_top_dst = slice::from_raw_parts_mut(top_dst, s_len * Self::XSTEP);

        let s_bottom = if bottom_y.is_null() || bottom_dst.is_null() {
            None
        } else {
            Some((slice::from_raw_parts(bottom_y, s_len), slice::from_raw_parts_mut(bottom_dst, s_len * Self::XSTEP)))
        };
        Self::do_upsample_line_pair(s_top_y, s_bottom, s_top_u, s_top_v, s_cur_u, s_cur_v, s_top_dst, s_len);
    }
}

// All variants implemented

struct YuvToRgbaUpsampler {}
impl FancyUpsampler for YuvToRgbaUpsampler {
    type UpsampleDest = [u8; 4];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 4]) {
        vp8_yuv_to_rgba(y, u, v, out)
    }
}

struct YuvToBgraUpsampler {}
impl FancyUpsampler for YuvToBgraUpsampler {
    type UpsampleDest = [u8; 4];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 4]) {
        todo!()
    }
}

struct YuvToRgbaPremultipliedUpsampler {}
impl FancyUpsampler for YuvToRgbaPremultipliedUpsampler {
    type UpsampleDest = [u8; 4];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 4]) {
        todo!()
    }
}

struct YuvToBgraPremultipliedUpsampler {}
impl FancyUpsampler for YuvToBgraPremultipliedUpsampler {
    type UpsampleDest = [u8; 4];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 4]) {
        todo!()
    }
}

struct YuvToRgbUpsampler {}
impl FancyUpsampler for YuvToRgbUpsampler {
    type UpsampleDest = [u8; 3];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 3]) {
        todo!()
    }
}

struct YuvToBgrUpsampler {}
impl FancyUpsampler for YuvToBgrUpsampler {
    type UpsampleDest = [u8; 3];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 3]) {
        todo!()
    }
}

struct YuvToBgrPremultipliedUpsampler {}
impl FancyUpsampler for YuvToBgrPremultipliedUpsampler {
    type UpsampleDest = [u8; 3];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 3]) {
        todo!()
    }
}

struct YuvToArgbUpsampler {}
impl FancyUpsampler for YuvToArgbUpsampler {
    type UpsampleDest = [u8; 4];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 4]) {
        todo!()
    }
}

struct YuvToRgba4444Upsampler {}
impl FancyUpsampler for YuvToRgba4444Upsampler {
    type UpsampleDest = [u8; 2];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 2]) {
        todo!()
    }
}

struct YuvToRgb565Upsampler {}
impl FancyUpsampler for YuvToRgb565Upsampler {
    type UpsampleDest = [u8; 2];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 2]) {
        todo!()
    }
}

struct YuvToArgbPremultipledUpsampler {}
impl FancyUpsampler for YuvToArgbPremultipledUpsampler {
    type UpsampleDest = [u8; 4];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 4]) {
        todo!()
    }
}

struct YuvToRgbaPremultipled4444Upsampler {}
impl FancyUpsampler for YuvToRgbaPremultipled4444Upsampler {
    type UpsampleDest = [u8; 2];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 2]) {
        todo!()
    }
}

struct NotARealUpsampler {}
impl FancyUpsampler for NotARealUpsampler {
    type UpsampleDest = [u8; 4];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 4]) {
        todo!()
    } 
}


extern "C" {
    fn UpsampleRgbLinePair_C(top_y: *const u8, bottom_y: *const u8,
        top_u: *const u8, top_v: *const u8, cur_u: *const u8, cur_v: *const u8,
        top_dst: *mut u8, bottom_dst: *mut u8, len: c_uint);

    fn UpsampleBgrLinePair_C(top_y: *const u8, bottom_y: *const u8,
        top_u: *const u8, top_v: *const u8, cur_u: *const u8, cur_v: *const u8,
        top_dst: *mut u8, bottom_dst: *mut u8, len: c_uint);

    fn UpsampleBgraLinePair_C(top_y: *const u8, bottom_y: *const u8,
        top_u: *const u8, top_v: *const u8, cur_u: *const u8, cur_v: *const u8,
        top_dst: *mut u8, bottom_dst: *mut u8, len: c_uint);

    fn UpsampleArgbLinePair_C(top_y: *const u8, bottom_y: *const u8,
        top_u: *const u8, top_v: *const u8, cur_u: *const u8, cur_v: *const u8,
        top_dst: *mut u8, bottom_dst: *mut u8, len: c_uint);

    fn UpsampleRgba4444LinePair_C(top_y: *const u8, bottom_y: *const u8,
        top_u: *const u8, top_v: *const u8, cur_u: *const u8, cur_v: *const u8,
        top_dst: *mut u8, bottom_dst: *mut u8, len: c_uint);

    fn UpsampleRgb565LinePair_C(top_y: *const u8, bottom_y: *const u8,
        top_u: *const u8, top_v: *const u8, cur_u: *const u8, cur_v: *const u8,
        top_dst: *mut u8, bottom_dst: *mut u8, len: c_uint);

    fn UpsampleRgbaLinePair_C(top_y: *const u8, bottom_y: *const u8,
        top_u: *const u8, top_v: *const u8, cur_u: *const u8, cur_v: *const u8,
        top_dst: *mut u8, bottom_dst: *mut u8, len: c_uint);
}

/*
extern "C" {
    pub static mut WebPUpsamplers: [unsafe extern "C" fn(*const u8, *const u8, *const u8, *const u8, *const u8, *const u8, *mut u8, *mut u8, u32); 13];
}
*/

#[no_mangle]
unsafe extern "C" fn WebPInitUpsamplers() {
/* 
    WebPUpsamplers[MODE_RGBA as usize]      = YuvToRgbaUpsampler::ffi_upsample_line_pair;
    WebPUpsamplers[MODE_BGRA as usize]      = YuvToBgraUpsampler::ffi_upsample_line_pair;
    WebPUpsamplers[MODE_rgbA as usize]      = YuvToRgbaUpsampler::ffi_upsample_line_pair;
    WebPUpsamplers[MODE_bgrA as usize]      = YuvToBgraUpsampler::ffi_upsample_line_pair;
    WebPUpsamplers[MODE_RGB as usize]       = YuvToRgbUpsampler::ffi_upsample_line_pair;
    WebPUpsamplers[MODE_BGR as usize]       = YuvToBgrUpsampler::ffi_upsample_line_pair;
    WebPUpsamplers[MODE_ARGB as usize]      = YuvToArgbUpsampler::ffi_upsample_line_pair;
    WebPUpsamplers[MODE_RGBA_4444 as usize] = YuvToRgba4444Upsampler::ffi_upsample_line_pair;
    WebPUpsamplers[MODE_RGB_565 as usize]   = YuvToRgb565Upsampler::ffi_upsample_line_pair;
    WebPUpsamplers[MODE_Argb as usize]      = YuvToArgbUpsampler::ffi_upsample_line_pair;
    WebPUpsamplers[MODE_rgbA_4444 as usize] = YuvToRgba4444Upsampler::ffi_upsample_line_pair;
    */
}


#[no_mangle]
pub static WebPUpsamplers: [unsafe extern "C" fn(*const u8, *const u8, *const u8, *const u8, *const u8, *const u8, *mut u8, *mut u8, u32); 13] = [
    UpsampleRgbLinePair_C, //YuvToRgbUpsampler::ffi_upsample_line_pair,  // MODE_RGB
    /*UpsampleRgbaLinePair_C, */ YuvToRgbaUpsampler::ffi_upsample_line_pair, // MODE_RGBA
    UpsampleBgrLinePair_C, //YuvToBgrUpsampler::ffi_upsample_line_pair,  // MODE_BGR
    UpsampleBgraLinePair_C, //YuvToBgraUpsampler::ffi_upsample_line_pair,  // MODE_BGRA
    UpsampleArgbLinePair_C, //YuvToArgbUpsampler::ffi_upsample_line_pair,  // MODE_ARGB
    UpsampleRgba4444LinePair_C, //YuvToRgba4444Upsampler::ffi_upsample_line_pair, // MODE_RGBA_4444
    UpsampleRgb565LinePair_C, // YuvToRgb565Upsampler::ffi_upsample_line_pair,  // MODE_RGB_565
    /*UpsampleRgbaLinePair_C,*/ YuvToRgbaUpsampler::ffi_upsample_line_pair,  // MODE_rgbA
    UpsampleBgraLinePair_C, // YuvToBgraUpsampler::ffi_upsample_line_pair, // MODE_bgrA
    UpsampleArgbLinePair_C, //YuvToArgbUpsampler::ffi_upsample_line_pair,  // MODE_Argb
    UpsampleRgba4444LinePair_C, // YuvToRgba4444Upsampler::ffi_upsample_line_pair, // MODE_rgbA_4444
    NotARealUpsampler::ffi_upsample_line_pair, // MODE_YUV
    NotARealUpsampler::ffi_upsample_line_pair, // MODE_YUVA
];


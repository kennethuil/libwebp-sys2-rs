use core::slice;
use std::{convert::{TryFrom, TryInto}, fmt::Debug, ptr::{slice_from_raw_parts, slice_from_raw_parts_mut}};

use libc::c_uint;

use crate::{MODE_ARGB, MODE_Argb, MODE_BGR, MODE_BGRA, MODE_RGB, MODE_RGBA, MODE_RGBA_4444, MODE_RGB_565, MODE_bgrA, MODE_rgbA, MODE_rgbA_4444, yuv::{vp8_yuv_to_argb, vp8_yuv_to_bgr, vp8_yuv_to_bgra, vp8_yuv_to_rgb, vp8_yuv_to_rgb_565, vp8_yuv_to_rgba, vp8_yuv_to_rgba_4444}};

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

    // We expect y, u, v to be of the same length, and dst to be that length multiplied by XSTEP
    fn convert_yuv_444(y: &[u8], u: &[u8], v: &[u8], dst: &mut [u8]) where for <'a> &'a mut <Self as FancyUpsampler>::UpsampleDest: TryFrom<&'a mut [u8]> {
        let dst_chunks = dst.chunks_exact_mut(Self::XSTEP);
        for ((y, u), (v, dst)) in y.into_iter().zip(u.into_iter()).zip(v.into_iter().zip(dst_chunks.into_iter())) {
            let dst = dst.try_into().unwrap_or_else(|_| panic!("nope"));
            Self::upsample(*y, *u, *v, dst);
        }
    }

    
    unsafe extern "C" fn ffi_convert_yuv_444(y: *const u8, u: *const u8, v: *const u8, dst: *mut u8, len: c_uint) where for <'a> &'a mut <Self as FancyUpsampler>::UpsampleDest: TryFrom<&'a mut [u8]> {
        let len = len as usize;
        let y = slice::from_raw_parts(y, len);
        let u = slice::from_raw_parts(u, len);
        let v = slice::from_raw_parts(v, len);
        let dst = slice::from_raw_parts_mut(dst, len * Self::XSTEP);
        Self::convert_yuv_444(y, u, v, dst);
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

    unsafe extern "C" fn ffi_upsample(y: c_uint, u: c_uint, v: c_uint, out: *mut u8) where for <'a> &'a mut <Self as FancyUpsampler>::UpsampleDest: TryFrom<&'a mut [u8]> {
        let out = slice::from_raw_parts_mut(out, Self::XSTEP);
        let out = out.try_into().unwrap_or_else(|_| panic!("nope"));
        Self::upsample(y as u8, u as u8, v as u8, out);
    }
}

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
        vp8_yuv_to_bgra(y, u, v, out)
    }
}

struct YuvToRgbUpsampler {}
impl FancyUpsampler for YuvToRgbUpsampler {
    type UpsampleDest = [u8; 3];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 3]) {
        vp8_yuv_to_rgb(y, u, v, out)
    }
}

struct YuvToBgrUpsampler {}
impl FancyUpsampler for YuvToBgrUpsampler {
    type UpsampleDest = [u8; 3];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 3]) {
        vp8_yuv_to_bgr(y, u, v, out)
    }
}

struct YuvToArgbUpsampler {}
impl FancyUpsampler for YuvToArgbUpsampler {
    type UpsampleDest = [u8; 4];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 4]) {
        vp8_yuv_to_argb(y, u, v, out)
    }
}

struct YuvToRgba4444Upsampler {}
impl FancyUpsampler for YuvToRgba4444Upsampler {
    type UpsampleDest = [u8; 2];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 2]) {
        vp8_yuv_to_rgba_4444(y, u, v, out);
    }
}

struct YuvToRgb565Upsampler {}
impl FancyUpsampler for YuvToRgb565Upsampler {
    type UpsampleDest = [u8; 2];
    fn upsample(y: u8, u: u8, v: u8, out: &mut [u8; 2]) {
        vp8_yuv_to_rgb_565(y, u, v, out);
    }
}

struct NotARealUpsampler {}
impl FancyUpsampler for NotARealUpsampler {
    type UpsampleDest = [u8; 4];
    fn upsample(_y: u8, _u: u8, _v: u8, _out: &mut [u8; 4]) {
        unimplemented!()
    } 
}

#[no_mangle]
unsafe extern "C" fn WebPInitUpsamplers() {

}

#[no_mangle]
unsafe extern "C" fn WebPInitYUV444Converters() {

}

#[no_mangle]
pub static WebPUpsamplers: [unsafe extern "C" fn(*const u8, *const u8, *const u8, *const u8, *const u8, *const u8, *mut u8, *mut u8, u32); 13] = [
    YuvToRgbUpsampler::ffi_upsample_line_pair,  // MODE_RGB
    YuvToRgbaUpsampler::ffi_upsample_line_pair, // MODE_RGBA
    YuvToBgrUpsampler::ffi_upsample_line_pair,  // MODE_BGR
    YuvToBgraUpsampler::ffi_upsample_line_pair,  // MODE_BGRA
    YuvToArgbUpsampler::ffi_upsample_line_pair,  // MODE_ARGB
    YuvToRgba4444Upsampler::ffi_upsample_line_pair, // MODE_RGBA_4444
    YuvToRgb565Upsampler::ffi_upsample_line_pair,  // MODE_RGB_565
    YuvToRgbaUpsampler::ffi_upsample_line_pair,  // MODE_rgbA
    YuvToBgraUpsampler::ffi_upsample_line_pair, // MODE_bgrA
    YuvToArgbUpsampler::ffi_upsample_line_pair,  // MODE_Argb
    YuvToRgba4444Upsampler::ffi_upsample_line_pair, // MODE_rgbA_4444
    NotARealUpsampler::ffi_upsample_line_pair, // MODE_YUV
    NotARealUpsampler::ffi_upsample_line_pair, // MODE_YUVA
];

#[no_mangle]
pub static WebPYUV444Converters: [unsafe extern "C" fn(y: *const u8, u: *const u8, v: *const u8, dst: *mut u8, len: c_uint); 13] = [
    YuvToRgbUpsampler::ffi_convert_yuv_444,  // MODE_RGB
    YuvToRgbaUpsampler::ffi_convert_yuv_444, // MODE_RGBA
    YuvToBgrUpsampler::ffi_convert_yuv_444,  // MODE_BGR
    YuvToBgraUpsampler::ffi_convert_yuv_444,  // MODE_BGRA
    YuvToArgbUpsampler::ffi_convert_yuv_444,  // MODE_ARGB
    YuvToRgba4444Upsampler::ffi_convert_yuv_444, // MODE_RGBA_4444
    YuvToRgb565Upsampler::ffi_convert_yuv_444,  // MODE_RGB_565
    YuvToRgbaUpsampler::ffi_convert_yuv_444,  // MODE_rgbA
    YuvToBgraUpsampler::ffi_convert_yuv_444, // MODE_bgrA
    YuvToArgbUpsampler::ffi_convert_yuv_444,  // MODE_Argb
    YuvToRgba4444Upsampler::ffi_convert_yuv_444, // MODE_rgbA_4444
    NotARealUpsampler::ffi_convert_yuv_444, // MODE_YUV
    NotARealUpsampler::ffi_convert_yuv_444, // MODE_YUVA
];

#[no_mangle]
unsafe extern "C" fn VP8YuvToRgba(y: c_uint, u: c_uint, v: c_uint, out: *mut u8) {
    YuvToRgbaUpsampler::ffi_upsample(y, u, v, out);
}

#[no_mangle]
unsafe extern "C" fn VP8YuvToBgra(y: c_uint, u: c_uint, v: c_uint, out: *mut u8) {
    YuvToBgraUpsampler::ffi_upsample(y, u, v, out);
}

#[no_mangle]
unsafe extern "C" fn VP8YuvToArgb(y: c_uint, u: c_uint, v: c_uint, out: *mut u8) {
    YuvToArgbUpsampler::ffi_upsample(y, u, v, out);
}

#[no_mangle]
unsafe extern "C" fn VP8YuvToRgba4444(y: c_uint, u: c_uint, v: c_uint, out: *mut u8) {
    YuvToRgba4444Upsampler::ffi_upsample(y, u, v, out);
}

#[no_mangle]
unsafe extern "C" fn VP8YuvToRgb565(y: c_uint, u: c_uint, v: c_uint, out: *mut u8) {
    YuvToRgb565Upsampler::ffi_upsample(y, u, v, out);
}

#[no_mangle]
unsafe extern "C" fn VP8YuvToBgr(y: c_uint, u: c_uint, v: c_uint, out: *mut u8) {
    YuvToBgrUpsampler::ffi_upsample(y, u, v, out);
}

#[no_mangle]
unsafe extern "C" fn VP8YuvToRgb(y: c_uint, u: c_uint, v: c_uint, out: *mut u8) {
    YuvToRgbUpsampler::ffi_upsample(y, u, v, out);
}
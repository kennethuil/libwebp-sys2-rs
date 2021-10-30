use core::slice;
use std::convert::TryInto;
use libc::{c_int, c_void};
use crate::alpha_dec::ALPHDecoder;
use crate::bit_reader_utils::VP8BitReader;
use crate::VP8StatusCode;
use crate::common_dec::{MAX_NUM_PARTITIONS, MB_FEATURE_TREE_PROBS, NUM_BANDS, NUM_CTX, NUM_MB_SEGMENTS, NUM_MODE_LF_DELTAS, NUM_PROBAS, NUM_REF_LF_DELTAS, NUM_TYPES};
use crate::dec::{dc16, dc16_no_left, dc16_no_top, dc16_no_top_left, dc4, dc8_uv, dc8_uv_no_left, dc8_uv_no_top, dc8_uv_no_top_left, h_filter_16, h_filter_16i, h_filter_8, h_filter_8i, hd4, he16, he4, he8_uv, hu4, ld4, rd4, simple_h_filter_16, simple_h_filter_16i, simple_v_filter_16, simple_v_filter_16i, tm16, tm4, tm8uv, v_filter_16, v_filter_16i, v_filter_8, v_filter_8i, ve16, ve4, ve8_uv, vl4, vr4};
use crate::dsp::UBPS;
use crate::frame_dec::{K_SCAN, do_transform, do_uv_transform};
use crate::offsetref::{OffsetArray, OffsetSliceRefMut};
use crate::random_utils::VP8Random;
//use bytemuck::{Pod, Zeroable};

// YUV-cache parameters. Cache is 32-bytes wide (= one cacheline).
// Constraints are: We need to store one 16x16 block of luma samples (y),
// and two 8x8 chroma blocks (u/v). These are better be 16-bytes aligned,
// in order to be SIMD-friendly. We also need to store the top, left and
// top-left samples (from previously decoded blocks), along with four
// extra top-right samples for luma (intra4x4 prediction only).
// One possible layout is, using 32 * (17 + 9) bytes:
//
//   .+------   <- only 1 pixel high
//   .|yyyyt.
//   .|yyyyt.
//   .|yyyyt.
//   .|yyyy..
//   .+--.+--   <- only 1 pixel high
//   .|uu.|vv
//   .|uu.|vv
//
// Every character is a 4x4 block, with legend:
//  '.' = unused
//  'y' = y-samples   'u' = u-samples     'v' = u-samples
//  '|' = left sample,   '-' = top sample,    '+' = top-left sample
//  't' = extra top-right sample for 4x4 modes
//const YUV_SIZE: usize = UBPS * 17 + UBPS * 9;
//const Y_OFF: usize = UBPS * 1 + 8;
//const U_OFF: usize = Y_OFF + UBPS * 16 + UBPS;
//const V_OFF: usize = U_OFF + 16;

#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8Io {
    width: c_int,
    height: c_int,
    mb_y: c_int,
    mb_w: c_int,
    mb_h: c_int,
    y: *mut u8,
    u: *mut u8,
    v: *mut u8,
    y_stride: c_int,
    uv_stride: c_int,
    opaque: *const c_void,
    put: extern "C" fn (*mut VP8Io) -> c_int,
    setup: extern "C" fn (*mut VP8Io) -> c_int,
    teardown: extern "C" fn (*mut VP8Io),
    fancy_upsampling: c_int,
    data_size: usize,
    data: *mut u8,
    bypass_filtering: c_int,
    use_cropping: c_int,
    crop_left: c_int,
    crop_right: c_int,
    crop_top: c_int,
    crop_bottom: c_int,
    use_scaling: c_int,
    scaled_width: c_int,
    scaled_height: c_int,
    a: *mut u8,
}

//------------------------------------------------------------------------------
// Headers
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct VP8FrameHeader {
    key_frame: u8,
    profile: u8,
    show: u8,
    partition_length: u32,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8PictureHeader {
    width: u16,
    height: u16,
    xscale: u8,
    yscale: u8,
    colorspace: u8,   // 0 = YCbCr
    clamp_type: u8,
}

// segment features
#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8SegmentHeader {
    use_segment: c_int,
    update_map: c_int,                      // whether to update the segment map or not
    absolute_delta: c_int,                  // absolute or delta values for quantizer and filter
    quantizer: [i8; NUM_MB_SEGMENTS],       // quantization changes
    filter_strength: [i8; NUM_MB_SEGMENTS], // filter strength for segments
}

// probas associated to one of the contexts
type VP8ProbaArray = [u8; NUM_PROBAS];

#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8BandProbas {   // all the probas associated to one band
    probas: [VP8ProbaArray; NUM_CTX]
}

// Struct collecting all frame-persistent probabilities.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8Proba {
    segments: [u8; MB_FEATURE_TREE_PROBS],
    // Type: 0:Intra16-AC  1:Intra16-DC   2:Chroma   3:Intra4
    bands: [[VP8BandProbas; NUM_BANDS]; NUM_TYPES],
    bands_ptr: [[*mut VP8BandProbas; 16 + 1]; NUM_TYPES],
}

// Filter parameters
#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8FilterHeader {
    simple: c_int,      // 0=complex, 1=simple
    level: c_int,       // [0..63]
    sharpness: c_int,   // [0..7]
    use_lf_delta: c_int,
    ref_lf_delta: [c_int; NUM_REF_LF_DELTAS],
    mode_lf_delta: [c_int; NUM_MODE_LF_DELTAS],
}

//------------------------------------------------------------------------------
// Informations about the macroblocks.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8FInfo {
    f_limit: u8,        // filter limit in [3..189], or 0 if no filtering
    f_ilevel: u8,       // inner limit in [1..63]
    f_inner: u8,        // do inner filtering?
    hev_thresh: u8,     // high edge variance threshold in [0..2]
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8MB {
    nz: u8,     // non-zero AC/DC coeffs (4bit for luma + 4bit for chroma)
    nz_dc: u8,  // non-zero DC coeff (1bit)
}

// Dequantization matrices
type QuantT = [c_int; 2];   // [DC / AC].  Can be 'uint16_t[2]' too (~slower).

#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8QuantMatrix {
    y1_mat: QuantT,
    y2_mat: QuantT,
    uv_mat: QuantT,
    uv_quant: c_int,    // U/V quantizer value
    dither: c_int,      // dithering amplitude (0 = off, max=255)
}

// Data needed to reconstruct a macroblock
#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8MBData {
    coeffs: [i16; 384], // 384 coeffs = (16+4+4) * 4*4
    is_i4x4: u8,        // true if intra4x4
    imodes: [u8; 16],   // one 16x16 mode (#0) or sixteen 4x4 modes
    uvmode: u8,         // chroma prediction mode
    // bit-wise info about the content of each sub-4x4 blocks (in decoding order).
    // Each of the 4x4 blocks for y/u/v is associated with a 2b code according to:
    //   code=0 -> no coefficient
    //   code=1 -> only DC
    //   code=2 -> first three coefficients are non-zero
    //   code=3 -> more than three coefficients are non-zero
    // This allows to call specialized transform functions.
    non_zero_y: u32,
    non_zero_uv: u32,
    dither: u8,     // local dithering strength (deduced from non_zero_*)
    skip: u8,
    segment: u8,
}


// From thread_utils.h
#[allow(dead_code)]
#[repr(C)]
#[derive(Debug)]
enum WebPWorkerStatus {
    
    NotOk = 0,
    Ok,
    Work,
}

#[repr(C)]
#[derive(Debug)]
struct WebPWorker {
    impl_: *const c_void,
    status: WebPWorkerStatus,
    hook: extern "C" fn (*const c_void, *const c_void) -> c_int,
    data1: *const c_void,
    data2: *const c_void,
    had_error: c_int,
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8ThreadContext {
    id: c_int,                  // cache row to process (in [0..2])
    mb_y: c_int,                // macroblock position of the row
    filter_row: c_int,          // true if row-filtering is needed
    f_info: *mut VP8FInfo,      // filter strengths (swapped with dec->f_info_)
    mb_data: *mut VP8MBData,    // reconstruction data (swapped with dec->mb_data_)
    io: VP8Io,                  // copy of the VP8Io to pass to put()
}

// Saved top samples, per macroblock. Fits into a cache-line.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8TopSamples {
    y: [u8; 16],
    u: [u8; 8],
    v: [u8; 8]
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8Decoder_FFI {
    status: VP8StatusCode,
    ready: c_int,  // true if ready to decode a picture with VP8Decode()
    error_msg: *const u8, // set when status_ is not OK.

    // Main data source
    br: VP8BitReader,

    // headers
    frm_hdr: VP8FrameHeader,
    pic_hdr: VP8PictureHeader,
    filter_hdr: VP8FilterHeader,
    segment_hdr: VP8SegmentHeader,

    // Worker
    worker: WebPWorker,
    mt_method: c_int,   // multi-thread method: 0=off, 1=[parse+recon][filter]
                        // 2=[parse][recon+filter]
                        // NOTE: always setting to 0, if needed, multithreading will be added back in the Rust way
    cache_id: c_int,    // current cache row, always 0 in the single-threaded case
    num_caches: c_int,  // number of cached rows of 16 pixels (1, 2 or 3) (always 1 in single-threaded case)
    thread_ctx: VP8ThreadContext,   // Thread context

    // dimension, in macroblock units.
    pub(crate) mb_w: c_int,
    pub(crate) mb_h: c_int,

    // Macroblock to process/filter, depending on cropping and filter_type.
    tl_mb_x: c_int,     // top-left MB that must be in-loop filtered
    tl_mb_y: c_int,
    br_mb_x: c_int,     // last bottom-right MB that must be decoded
    br_mb_y: c_int,

    // number of partitions minus one.
    num_parts_minus_one: u32,
    // per-partition boolean decoders.
    parts: [VP8BitReader; MAX_NUM_PARTITIONS],

    // Dithering strength, deduced from decoding options
    dither: c_int,              // whether to use dithering or not
    dithering_rg: VP8Random,    // random generator for dithering

    // dequantization (one set of DC/AC dequant factor per segment)
    dqm: [VP8QuantMatrix; NUM_MB_SEGMENTS],

    // probabilities
    proba: VP8Proba,
    use_skip_proba: c_int,
    skip_p: u8,

    // Boundary data cache and persistent buffers.
    intra_t: *mut u8,   // top intra modes values: 4 * mb_w_
    intra_l: [u8; 4],   // left intra modes values

    yuv_t: *mut VP8TopSamples,  // top y/u/v samples

    mb_info: *mut VP8MB,        // contextual macroblock info (mb_w_ + 1)
    f_info: *mut VP8FInfo,      // filter strength info
    yuv_b: *mut u8,             // main block for Y/U/V (size = YUV_SIZE)

    cache_y: *mut u8,           // macroblock row for storing unfiltered samples
    cache_u: *mut u8,
    cache_v: *mut u8,
    cache_y_stride: c_int,
    cache_uv_stride: c_int,

    // main memory chunk for the above data. Persistent.
    mem: *mut u8,
    mem_size: usize,

    // Per macroblock non-persistent infos.
    mb_x: c_int,        // current position, in macroblock units
    mb_y: c_int,
    mb_data: *mut VP8MBData, // parsed reconstruction data

    // Filtering side-info
    filter_type: c_int,                             // 0=off, 1=simple, 2=complex
    fstrengths: [[VP8FInfo; 2]; NUM_MB_SEGMENTS],   // precalculated per-segment/type

    // Alpha
    alph_dec: *mut ALPHDecoder, // alpha-plane decoder object
    alpha_data: *mut u8,        // compressed alpha data (if present)
    alpha_data_size: usize,
    is_alpha_decoded: c_int,    // true if alpha_data_ is decoded in alpha_plane_
    alpha_plane_mem: *mut u8,   // memory allocated for alpha_plane_
    alpha_plane: *mut u8,       // output. Persistent, contains the whole data.
    alpha_prev_line: *mut u8,   // last decoded alpha row (or NULL)
    alpha_dithering: c_int,     // derived from decoding options (0=off, 100=full)
}


const YUV_SIZE: usize = UBPS * 17 + UBPS * 9;
const Y_OFF: usize = UBPS * 1 + 8;
const U_OFF: usize = Y_OFF + UBPS * 16 + UBPS;
const V_OFF: usize = U_OFF + 16;

// K_FILTER_EXTRA_ROWS[] = How many extra lines are needed on the MB boundary
// for caching, given a filtering level.
// Simple filter:  up to 2 luma samples are read and 1 is written.
// Complex filter: up to 4 luma samples are read and 3 are written. Same for
//                 U/V, so it's 8 samples total (because of the 2x upsampling).
static K_FILTER_EXTRA_ROWS: [usize; 3] = [0, 2, 8];

//#[derive(Debug)]
pub(crate) struct VP8Decoder<'dec> {
    // dimension, in macroblock units.
    mb_w: usize, // int?
    mb_h: usize, // int?
    yuv_t: &'dec mut [VP8TopSamples],  // top y/u/v samples
    // ...
    f_info: &'dec mut [VP8FInfo],
    yuv_b: &'dec mut [u8; YUV_SIZE], // main block for Y/U/V (size = YUV_SIZE)

    cache_y: OffsetSliceRefMut<'dec, u8>, // macroblock row for storing unfiltered samples
    cache_u: OffsetSliceRefMut<'dec, u8>,
    cache_v: OffsetSliceRefMut<'dec, u8>,
    cache_y_stride: usize, // int?
    cache_uv_stride: usize, // int?

    // Per macroblock non-persistent infos.
    mb_x: usize, // int?
    mb_y: usize, // int?
    mb_data: &'dec mut [VP8MBData], // size: mb_w

    // Filtering side-info
    filter_type: usize,   // 0=off, 1=simple, 2=complex
}

fn copy_32b_left(arr: &mut [u8], dst_idx: usize, src_idx: usize) {
    let (dst, src) = arr.split_at_mut(src_idx);
    dst[dst_idx..dst_idx+4].copy_from_slice(&src[0..4]);
}




impl VP8Decoder<'_> {
    unsafe fn from_ffi(ffi: *mut VP8Decoder_FFI) -> Self {
        let extra_rows = K_FILTER_EXTRA_ROWS[(*ffi).filter_type as usize];
        let extra_y = extra_rows * (*ffi).cache_y_stride as usize;
        let extra_uv = (extra_rows / 2) * (*ffi).cache_uv_stride as usize;
        let f_info_size = if (*ffi).filter_type > 0 {
            (*ffi).mb_w as usize
        } else {
            0
        };

        VP8Decoder { 
            mb_w: (*ffi).mb_w as usize,
            mb_h: (*ffi).mb_h as usize, 
            yuv_t: slice::from_raw_parts_mut((*ffi).yuv_t, (*ffi).mb_w as usize), 
            f_info: slice::from_raw_parts_mut((*ffi).f_info, f_info_size),
            yuv_b: &mut *((*ffi).yuv_b as *mut [u8; YUV_SIZE]),
            cache_y: OffsetSliceRefMut::from_zero_mut_ptr((*ffi).cache_y, -(extra_y as isize),16 * (*ffi).cache_y_stride as isize),
            cache_u: OffsetSliceRefMut::from_zero_mut_ptr((*ffi).cache_u, -(extra_uv as isize), 8 * (*ffi).cache_uv_stride as isize),
            cache_v: OffsetSliceRefMut::from_zero_mut_ptr((*ffi).cache_v, -(extra_uv as isize), 8 * (*ffi).cache_uv_stride as isize),
            cache_y_stride: (*ffi).cache_y_stride as usize, 
            cache_uv_stride: (*ffi).cache_uv_stride as usize, 
            mb_x: (*ffi).mb_x as usize, 
            mb_y: (*ffi).mb_y as usize, 
            mb_data: slice::from_raw_parts_mut((*ffi).mb_data, (*ffi).mb_w as usize),
            filter_type: (*ffi).filter_type as usize,
        }
    }

    pub(crate) fn do_filter(&mut self, mb_x: usize, mb_y: usize) {
        let y_bps = self.cache_y_stride as isize;
        let mut y_dst = self.cache_y.with_offset(mb_x as isize * 16);
        let f_info = &self.f_info[mb_x];
        let ilevel = f_info.f_ilevel;
        let limit = f_info.f_limit as u32;
        if limit == 0 {
            return;
        }
        assert!(limit >= 3);
        if self.filter_type == 1 { // simple
            
            if mb_x > 0 {
                simple_h_filter_16(&mut y_dst, y_bps, limit + 4);
            }
            
            if f_info.f_inner != 0 {
                simple_h_filter_16i(&mut y_dst, y_bps, limit);
            }
            
            
            if mb_y > 0 {
                simple_v_filter_16(&mut y_dst, y_bps, limit + 4);
            }
            
            if f_info.f_inner != 0 {
                simple_v_filter_16i(&mut y_dst, y_bps, limit);
            }
            
        } else {  // complex 
            
            let uv_bps = self.cache_uv_stride as isize;
            let mut u_dst = self.cache_u.with_offset(mb_x as isize * 8);
            let mut v_dst = self.cache_v.with_offset(mb_x as isize * 8);
            let hev_thresh = f_info.hev_thresh;

            if mb_x > 0 {
                h_filter_16(&mut y_dst, y_bps, limit + 4, ilevel, hev_thresh);
                h_filter_8(&mut u_dst, &mut v_dst, uv_bps, limit + 4, ilevel, hev_thresh);
            }
            if f_info.f_inner != 0 {
                h_filter_16i(&mut y_dst, y_bps, limit, ilevel, hev_thresh);
                h_filter_8i(&mut u_dst, &mut v_dst, uv_bps, limit, ilevel, hev_thresh);
            }
            if mb_y > 0 {
                v_filter_16(&mut y_dst, y_bps, limit + 4, ilevel, hev_thresh);
                v_filter_8(&mut u_dst, &mut v_dst, uv_bps, limit + 4, ilevel, hev_thresh);
            }
            if f_info.f_inner != 0 {
                v_filter_16i(&mut y_dst, y_bps, limit, ilevel, hev_thresh);
                v_filter_8i(&mut u_dst, &mut v_dst, uv_bps, limit, ilevel, hev_thresh);
            }
        }
    }

    fn reconstruct_macroblock(&mut self, mb_x: usize, mb_y: usize) {
        let block = &self.mb_data[mb_x];

        // Rotate in the left samples from previously decoded block. We move four
        // pixels at a time for alignment reason, and because of in-loop filter.

        if mb_x > 0 {
            for j in 0..17 {
                copy_32b_left(self.yuv_b, Y_OFF-UBPS + j * UBPS - 4, Y_OFF-UBPS + j * UBPS + 12);
                //self.yuv_b.copy_within(Y_OFF - UBPS + j * UBPS + 12..Y_OFF - UBPS + j * UBPS + 16, 
                //    Y_OFF - UBPS + j * UBPS - 4)
            }
            for j in 0..9 {
                copy_32b_left(self.yuv_b, U_OFF-UBPS + j * UBPS - 4, U_OFF-UBPS + j * UBPS + 4);
                copy_32b_left(self.yuv_b, V_OFF-UBPS + j * UBPS - 4, V_OFF-UBPS + j * UBPS + 4);
            }
        }


        
        // bring top samples into the cache
        let top_yuv = &mut self.yuv_t[mb_x..];
        let coeffs = &block.coeffs;
        let mut bits = block.non_zero_y;

        if mb_y > 0 {
            self.yuv_b[Y_OFF - UBPS..][..16].copy_from_slice(&top_yuv[0].y);
            self.yuv_b[U_OFF - UBPS..][..8].copy_from_slice(&top_yuv[0].u);
            self.yuv_b[V_OFF - UBPS..][..8].copy_from_slice(&top_yuv[0].v);
        }

        // predict and add residuals
        if block.is_i4x4 != 0 {    // 4x4
            // In the C they cast it to an uint32 pointer to let them deal with it 4 bytes at a time
            // we'll skip that and assume the optimizer will essentially do the same thing.
            let top_right = &mut self.yuv_b[Y_OFF - UBPS + 16..];
            if mb_y > 0 {
                if mb_x >= self.mb_w - 1 { // on rightmost border
                    top_right[..4].fill(top_yuv[0].y[15]);
                } else {
                    top_right[..4].copy_from_slice(&top_yuv[1].y[..4]);
                }
            }
            // replicate the top-right pixels below
            let to_replicate: [u8; 4] = top_right[0..4].try_into().unwrap();
            top_right[3*4 * UBPS..][..4].copy_from_slice(&to_replicate);
            top_right[2*4 * UBPS..][..4].copy_from_slice(&to_replicate);
            top_right[1*4 * UBPS..][..4].copy_from_slice(&to_replicate);

            // predict and add residuals for all 4x4 blocks in turn.
            for n in 0..16 {
                let dst_offset = Y_OFF + K_SCAN[n];
                Self::vp8_pred_luma4(self.yuv_b, dst_offset, block.imodes[n]);
                do_transform(bits, &coeffs[n * 16..], &mut self.yuv_b[dst_offset..]);
                bits <<= 2;
            }
        } else {    // 16x16
            let mode = Self::get_mode(mb_x, mb_y, block.imodes[0]);
            Self::vp8_pred_luma16(self.yuv_b, Y_OFF, mode);
            if bits != 0 {
                for n in 0..16 {
                    do_transform(bits, &coeffs[n * 16..], &mut self.yuv_b[Y_OFF + K_SCAN[n]..]);
                    bits <<= 2;
                }
            }
        }


        // Chroma
        let bits_uv = block.non_zero_uv;
        let mode = Self::get_mode(mb_x, mb_y, block.uvmode);
        Self::vp8_pred_chroma8(self.yuv_b, U_OFF, mode);
        Self::vp8_pred_chroma8(self.yuv_b, V_OFF, mode);
        do_uv_transform(bits_uv, &coeffs[16 * 16..], &mut self.yuv_b[U_OFF..]);
        do_uv_transform(bits_uv >> 8,&coeffs[20 * 16..], &mut self.yuv_b[V_OFF..]);

        // stash away top samples for next block
        if mb_y < self.mb_h - 1 {
            top_yuv[0].y.copy_from_slice(&self.yuv_b[Y_OFF + 15 * UBPS..][..16]);
            top_yuv[0].u.copy_from_slice(&self.yuv_b[U_OFF + 7 * UBPS..][..8]);
            top_yuv[0].v.copy_from_slice(&self.yuv_b[V_OFF + 7 * UBPS..][..8]);
        }
        // Transfer reconstructed samples from yuv_b_ cache to final destination.
        let y_out = &mut self.cache_y[mb_x as isize * 16..];
        let u_out = &mut self.cache_u[mb_x as isize * 8..];
        let v_out = &mut self.cache_v[mb_x as isize * 8..];
        for j in 0..16 {
            y_out[j * self.cache_y_stride..][..16].copy_from_slice(&self.yuv_b[Y_OFF + j * UBPS..][..16]);
        }
        for j in 0..8 {
            u_out[j * self.cache_uv_stride..][..8].copy_from_slice(&self.yuv_b[U_OFF + j * UBPS..][..8]);
            v_out[j * self.cache_uv_stride..][..8].copy_from_slice(&self.yuv_b[V_OFF + j * UBPS..][..8]);
        }
    }

    pub(crate) fn reconstruct_row(&mut self, mb_y: usize) {
        // Initialize left-most block.
        for j in 0..16 {
            self.yuv_b[Y_OFF + j*UBPS - 1] = 129;
        }
        for j in 0..8 {
            self.yuv_b[U_OFF + j * UBPS - 1] = 129;
            self.yuv_b[V_OFF + j * UBPS - 1] = 129;
        }

        // Init top-left sample on left column too.
        if mb_y > 0 {
            self.yuv_b[Y_OFF -1 - UBPS] = 129;
            self.yuv_b[U_OFF -1 - UBPS] = 129;
            self.yuv_b[V_OFF -1 - UBPS] = 129;
        } else {
            // we only need to do this init once at block (0,0).
            // Afterward, it remains valid for the whole topmost row.
            self.yuv_b[Y_OFF - UBPS - 1..][..16+4+1].fill(127);
            self.yuv_b[U_OFF - UBPS - 1..][..8+1].fill(127);
            self.yuv_b[V_OFF - UBPS - 1..][..8+1].fill(127);
        }

        // Reconstruct one row.
        for mb_x in 0..self.mb_w {
            self.reconstruct_macroblock(mb_x, mb_y)
        }

    }


    fn vp8_pred_luma4(dst: &mut [u8; YUV_SIZE], dst_offset: usize, imode: u8) {
        match imode {
            0 => dc4(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            1 => tm4(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            2 => ve4(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            3 => he4(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            4 => rd4(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            5 => vr4(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            6 => ld4(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            7 => vl4(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            8 => hd4(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            9 => hu4(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            _ => panic!("Unknown imode {}", imode)
        }
    }

    fn vp8_pred_luma16(dst: &mut [u8; YUV_SIZE], dst_offset: usize, imode: PredMode) {
        match imode {
            PredMode::Dc => dc16(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::Tm => tm16(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::Ve => ve16(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::He => he16(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::DcNoTop => dc16_no_top(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::DcNoLeft => dc16_no_left(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::DcNoTopLeft => dc16_no_top_left((&mut dst[dst_offset..][..UBPS*15+16]).try_into().unwrap()),
        }
    }

    fn vp8_pred_chroma8(dst: &mut [u8; YUV_SIZE], dst_offset: usize, uvmode: PredMode) {
        match uvmode {
            PredMode::Dc => dc8_uv(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::Tm => tm8uv(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::Ve => ve8_uv(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::He => he8_uv(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::DcNoTop => dc8_uv_no_top(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::DcNoLeft => dc8_uv_no_left(OffsetArray::from_zero_offset_slice_mut(dst, dst_offset)),
            PredMode::DcNoTopLeft => dc8_uv_no_top_left((&mut dst[dst_offset..][..UBPS*7+8]).try_into().unwrap()),
        }
    }

    // The C uses the numerical values directly (as index into function pointer array)
    // with some special casing.  We assume that the optimizer will turn this plus the
    // switch on enum above into a switch on numerical values.
    fn get_mode(mb_x: usize, mb_y: usize, mode: u8) -> PredMode {
        match(mode, mb_x, mb_y) {
            (0, 0, 0) => PredMode::DcNoTopLeft,
            (0, 0, _) => PredMode::DcNoLeft,
            (0, _, 0) => PredMode::DcNoTop,
            (0, _, _) => PredMode::Dc,
            (1, _, _) => PredMode::Tm,
            (2, _, _) => PredMode::Ve,
            (3, _, _) => PredMode::He,
            (4, _, _) => PredMode::DcNoTop,
            (5, _, _) => PredMode::DcNoLeft,
            (6, _, _) => PredMode::DcNoTopLeft,
            _ => panic!("Unknown mode {}", mode)
        }
    }
}

#[derive(Copy, Clone)]
enum PredMode {
    Dc,
    Tm,
    Ve,
    He,
    DcNoTop,
    DcNoLeft,
    DcNoTopLeft,    
}

//------------------------------------------------------------------------------------------
// Temporary extern wrappers

#[no_mangle]
unsafe extern "C" fn ReconstructRow(ffi: *mut VP8Decoder_FFI, ctx: *mut VP8ThreadContext) {
    let mut dec = VP8Decoder::from_ffi(ffi);
    dec.reconstruct_row((*ctx).mb_y as usize);
}

#[no_mangle]
unsafe extern "C" fn DoFilter(ffi: *mut VP8Decoder_FFI, mb_x: c_int, mb_y: c_int) {
    let mut dec = VP8Decoder::from_ffi(ffi);
    dec.do_filter(mb_x as usize, mb_y as usize);
}

#[no_mangle]
unsafe extern "C" fn ShowParams(y_dst: *mut u8, stride: isize, limit: u32, ilevel: u32) {
    println!("from C: y_dst = {:?}, stride = {}, limit = {}, ilevel = {}", y_dst, stride, limit, ilevel);
}
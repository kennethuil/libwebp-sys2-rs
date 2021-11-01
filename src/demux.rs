use std::os::raw::*;
use std::ptr;
use std::ptr::null_mut;

#[cfg(feature = "0_5")]
use crate::decode::*;
use crate::mux_types::*;
use std::convert::TryInto;

#[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
pub const WEBP_DEMUX_ABI_VERSION: c_int = WEBP_DEMUX_ABI_VERSION_INTERNAL;

cfg_if! {
    if #[cfg(feature = "0_5")] {
        const WEBP_DEMUX_ABI_VERSION_INTERNAL: c_int = 0x0107;
    } else {
        const WEBP_DEMUX_ABI_VERSION_INTERNAL: c_int = 0x0101;
    }
}

#[cfg(feature = "extern-types")]
extern "C" {
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub type WebPDemuxer;
}

#[cfg(not(feature = "extern-types"))]
#[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
#[repr(C)]
pub struct WebPDemuxer(c_void);

#[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
#[allow(non_camel_case_types)]
pub type WebPDemuxState = i32;

pub const WEBP_DEMUX_PARSE_ERROR: WebPDemuxState = -1;
pub const WEBP_DEMUX_PARSING_HEADER: WebPDemuxState = 0;
pub const WEBP_DEMUX_PARSED_HEADER: WebPDemuxState = 1;
pub const WEBP_DEMUX_DONE: WebPDemuxState = 2;

#[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
#[allow(non_camel_case_types)]
pub type WebPFormatFeature = u32;

pub const WEBP_FF_FORMAT_FLAGS: WebPFormatFeature = 0;
pub const WEBP_FF_CANVAS_WIDTH: WebPFormatFeature = 1;
pub const WEBP_FF_CANVAS_HEIGHT: WebPFormatFeature = 2;
pub const WEBP_FF_LOOP_COUNT: WebPFormatFeature = 3;
pub const WEBP_FF_BACKGROUND_COLOR: WebPFormatFeature = 4;
pub const WEBP_FF_FRAME_COUNT: WebPFormatFeature = 5;

#[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WebPIterator {
    pub frame_num: c_int,
    pub num_frames: c_int,
    #[cfg(not(feature = "0_5"))]
    #[deprecated(note = "Removed as of libwebp 0.5.0")]
    pub fragment_num: c_int,
    #[cfg(not(feature = "0_5"))]
    #[deprecated(note = "Removed as of libwebp 0.5.0")]
    pub num_fragments: c_int,
    pub x_offset: c_int,
    pub y_offset: c_int,
    pub width: c_int,
    pub height: c_int,
    pub duration: c_int,
    pub dispose_method: WebPMuxAnimDispose,
    pub complete: c_int,
    pub fragment: WebPData,
    pub has_alpha: c_int,
    pub blend_method: WebPMuxAnimBlend,
    #[doc(hidden)]
    pub pad: [u32; 2],
    #[doc(hidden)]
    pub private_: *mut c_void,
}

#[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WebPChunkIterator {
    pub chunk_num: c_int,
    pub num_chunks: c_int,
    pub chunk: WebPData,
    #[doc(hidden)]
    pub pad: [u32; 6],
    #[doc(hidden)]
    pub private_: *mut c_void,
}

type BlendRowFunc = unsafe extern "C" fn(*mut u32, *const u32, c_int);

#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[repr(C)]
pub struct WebPAnimDecoder {
    demux_: *mut WebPDemuxer,             // Demuxer created from given WebP bitstream.
    config_: WebPDecoderConfig,       // Decoder config.
    // Note: we use a pointer to a function blending multiple pixels at a time to
    // allow possible inlining of per-pixel blending function.
    blend_func_: BlendRowFunc,        // Pointer to the chose blend row function.
    info_: WebPAnimInfo,              // Global info about the animation.
    curr_frame_: *mut u8,            // Current canvas (not disposed).
    prev_frame_disposed_: *mut u8,   // Previous canvas (properly disposed).
    prev_frame_timestamp_: c_int,       // Previous frame timestamp (milliseconds).
    prev_iter_: WebPIterator,         // Iterator object for previous frame.
    prev_frame_was_keyframe_: c_int,    // True if previous frame was a keyframe.
    next_frame_: c_int,                 // Index of the next frame to be decoded
                                     // (starting from 1).
}

#[cfg(feature = "0_5")]
#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WebPAnimDecoderOptions {
    pub color_mode: WEBP_CSP_MODE,
    pub use_threads: c_int,
    #[doc(hidden)]
    pub padding: [u32; 7],
}

#[cfg(feature = "0_5")]
#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WebPAnimInfo {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub loop_count: u32,
    pub bgcolor: u32,
    pub frame_count: u32,
    #[doc(hidden)]
    pub pad: [u32; 4],
}

extern "C" {
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPGetDemuxVersion() -> c_int;
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    #[doc(hidden)]
    pub fn WebPDemuxInternal(
        _: *const WebPData,
        _: c_int,
        _: *mut WebPDemuxState,
        _: c_int,
    ) -> *mut WebPDemuxer;
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPDemuxDelete(dmux: *mut WebPDemuxer);
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPDemuxGetI(dmux: *const WebPDemuxer, feature: WebPFormatFeature) -> u32;
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPDemuxGetFrame(
        dmux: *const WebPDemuxer,
        frame_number: c_int,
        iter: *mut WebPIterator,
    ) -> c_int;
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPDemuxNextFrame(iter: *mut WebPIterator) -> c_int;
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPDemuxPrevFrame(iter: *mut WebPIterator) -> c_int;
    #[cfg(not(feature = "0_5"))]
    #[cfg_attr(
        feature = "__doc_cfg",
        doc(cfg(all(feature = "demux", feature = "0_5")))
    )]
    #[deprecated(note = "Removed as of libwebp 0.5.0")]
    pub fn WebPDemuxSelectFragment(iter: *mut WebPIterator, fragment_num: c_int) -> c_int;
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPDemuxReleaseIterator(iter: *mut WebPIterator);
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPDemuxGetChunk(
        dmux: *const WebPDemuxer,
        fourcc: *const c_char,
        chunk_number: c_int,
        iter: *mut WebPChunkIterator,
    ) -> c_int;
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPDemuxNextChunk(iter: *mut WebPChunkIterator) -> c_int;
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPDemuxPrevChunk(iter: *mut WebPChunkIterator) -> c_int;
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
    pub fn WebPDemuxReleaseChunkIterator(iter: *mut WebPChunkIterator);
    #[cfg(feature = "0_5")]
    #[cfg_attr(
        feature = "__doc_cfg",
        doc(cfg(all(feature = "demux", feature = "0_5")))
    )]
    #[doc(hidden)]
    pub fn WebPAnimDecoderNewInternal(
        _: *const WebPData,
        _: *const WebPAnimDecoderOptions,
        _: c_int,
    ) -> *mut WebPAnimDecoder;
    #[cfg(feature = "0_5")]
    #[cfg_attr(
        feature = "__doc_cfg",
        doc(cfg(all(feature = "demux", feature = "0_5")))
    )]
    pub fn WebPAnimDecoderGetInfo(dec: *const WebPAnimDecoder, info: *mut WebPAnimInfo) -> c_int;
    #[cfg(feature = "0_5")]
    #[cfg_attr(
        feature = "__doc_cfg",
        doc(cfg(all(feature = "demux", feature = "0_5")))
    )]
    pub fn WebPAnimDecoderGetNext(
        dec: *mut WebPAnimDecoder,
        buf: *mut *mut u8,
        timestamp: *mut c_int,
    ) -> c_int;
    #[cfg(feature = "0_5")]
    #[cfg_attr(
        feature = "__doc_cfg",
        doc(cfg(all(feature = "demux", feature = "0_5")))
    )]
    pub fn WebPAnimDecoderReset(dec: *mut WebPAnimDecoder);
    #[cfg(feature = "0_5")]
    #[cfg_attr(
        feature = "__doc_cfg",
        doc(cfg(all(feature = "demux", feature = "0_5")))
    )]
    pub fn WebPAnimDecoderGetDemuxer(dec: *const WebPAnimDecoder) -> *const WebPDemuxer;
    #[cfg(feature = "0_5")]
    #[cfg_attr(
        feature = "__doc_cfg",
        doc(cfg(all(feature = "demux", feature = "0_5")))
    )]
    pub fn WebPAnimDecoderDelete(dec: *mut WebPAnimDecoder);
}

#[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
#[allow(non_snake_case)]
#[inline]
pub unsafe extern "C" fn WebPDemux(data: *const WebPData) -> *mut WebPDemuxer {
    WebPDemuxInternal(data, 0, ptr::null_mut(), WEBP_DEMUX_ABI_VERSION)
}

#[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "demux")))]
#[allow(non_snake_case)]
#[inline]
pub unsafe extern "C" fn WebPDemuxPartial(
    data: *const WebPData,
    state: *mut WebPDemuxState,
) -> *mut WebPDemuxer {
    WebPDemuxInternal(data, 1, state, WEBP_DEMUX_ABI_VERSION)
}

#[cfg(feature = "0_5")]
#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[allow(non_snake_case)]
#[inline]
pub unsafe extern "C" fn WebPAnimDecoderOptionsInit(
    dec_options: *mut WebPAnimDecoderOptions,
) -> c_int {
    if dec_options == null_mut() {
      return 0;
    }
    DefaultDecoderOptions(dec_options);
    return 1;
}

#[cfg(feature = "0_5")]
#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[allow(non_snake_case)]
#[inline]
pub unsafe extern "C" fn WebPAnimDecoderNew(
    webp_data: *const WebPData,
    dec_options: *const WebPAnimDecoderOptions,
) -> *mut WebPAnimDecoder {
    WebPAnimDecoderNewInternal(webp_data, dec_options, WEBP_DEMUX_ABI_VERSION)
}

#[cfg(feature = "0_5")]
#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn DefaultDecoderOptions(
    dec_options: *mut WebPAnimDecoderOptions
) {
    (*dec_options).color_mode = MODE_RGBA;
    (*dec_options).use_threads = 0;
}

#[cfg(feature = "0_5")]
#[cfg_attr(
    feature = "__doc_cfg",
    doc(cfg(all(feature = "demux", feature = "0_5")))
)]
#[no_mangle]
pub unsafe extern "C" fn WebPAnimDecoderHasMoreFrames(dec: *const WebPAnimDecoder) -> c_int {
    if dec.is_null() {
        0
    } else if (*dec).next_frame_ <= (*dec).info_.frame_count.try_into().expect("comparing i32 to u32 :(") {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEBP_IMAGE: [u8; 94] = [
        0x52, 0x49, 0x46, 0x46, 0x56, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50, 0x38,
        0x20, 0x4A, 0x00, 0x00, 0x00, 0xD0, 0x01, 0x00, 0x9D, 0x01, 0x2A, 0x03, 0x00, 0x02, 0x00,
        0x02, 0x00, 0x34, 0x25, 0xA8, 0x02, 0x74, 0x01, 0x0E, 0xFE, 0x03, 0x8E, 0x00, 0x00, 0xFE,
        0xAD, 0xFF, 0xF1, 0x5C, 0xB4, 0xF8, 0xED, 0xFF, 0xF0, 0xC0, 0xBA, 0xBF, 0x93, 0x05, 0xEA,
        0x0C, 0x9F, 0x93, 0x3F, 0xE8, 0xC0, 0xBF, 0x3F, 0xFF, 0xA9, 0xBF, 0xFF, 0x24, 0x7B, 0xCB,
        0xFF, 0x46, 0x05, 0xF9, 0xFF, 0xFD, 0x4D, 0xFE, 0x30, 0xE5, 0x86, 0xAA, 0x07, 0x31, 0x23,
        0x6F, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn test_new_and_delete() {
        unsafe {
            let data = WebPData {
                bytes: WEBP_IMAGE.as_ptr(),
                size: WEBP_IMAGE.len(),
            };
            let ptr = WebPDemux(&data);
            assert!(!ptr.is_null());
            WebPDemuxDelete(ptr);
        }
    }

    #[cfg(all(feature = "0_5", feature = "demux"))]
    fn hash_animation<F>(buf: &[u8], f: &F) -> u64 where F: Fn(&mut WebPAnimDecoderOptions) {
        use siphasher::sip::SipHasher24;
        use std::hash::{Hasher};
        use std::mem;

        let mut hasher = SipHasher24::new_with_keys(0xca8e6089151e54eb, 0x58dbee492c222104);
        unsafe {
            let mut options = mem::zeroed();
            assert!(WebPAnimDecoderOptionsInit(&mut options) != 0);
            f(&mut options);

            let data = WebPData {
                bytes: buf.as_ptr(),
                size: buf.len(),
            };

            let decoder = WebPAnimDecoderNew(&data, &options);
            assert!(!decoder.is_null());

            let mut info = mem::zeroed();
            assert!(WebPAnimDecoderGetInfo(decoder, &mut info) != 0);

            hasher.write_u32(info.canvas_width);
            hasher.write_u32(info.canvas_height);
            hasher.write_u32(info.loop_count);
            hasher.write_u32(info.bgcolor);
            hasher.write_u32(info.frame_count);

            assert!(WebPAnimDecoderHasMoreFrames(decoder) > 0);

            while WebPAnimDecoderHasMoreFrames(decoder) > 0 {
                let mut buf_ptr = std::ptr::null_mut();
                let mut timestamp: i32 = 42;
                assert!(WebPAnimDecoderGetNext(decoder, &mut buf_ptr, &mut timestamp) > 0);
                let frame_bytes = std::slice::from_raw_parts(buf_ptr, (info.canvas_width * info.canvas_height * 4) as usize);
                hasher.write(frame_bytes);
                hasher.write_i32(timestamp);
            }

            WebPAnimDecoderDelete(decoder);
        }

        hasher.finish()
    }

    fn test_anim_content<F>(filename: &str, expected_hash: u64, f: &F) where F: Fn(&mut WebPAnimDecoderOptions) {
        use std::fs::File;
        use std::io::prelude::*;
        let mut buf = Vec::new();
        let len = File::open(filename)
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        assert!(len > 0);
        let hash = hash_animation(&buf, f);
        assert_eq!(expected_hash, hash, "hash mismatch in {}", filename);        
    }

    #[test]
    fn test_bgrA() {
        let f = |o: &mut WebPAnimDecoderOptions| {o.color_mode = MODE_bgrA;};
        test_anim_content("./tests/chip_lossless.webp", 2075130756176543755, &f);
        test_anim_content("./tests/chip_lossy.webp", 6955902631319670855, &f);
        test_anim_content("./tests/alpha_no_compression.webp", 12061699193968689595, &f);       
        test_anim_content("./tests/lossy_alpha1.webp", 12291741107973064377, &f);
    }
    #[test]
    fn test_rgbA() {
        let f = |o: &mut WebPAnimDecoderOptions| {o.color_mode = MODE_rgbA;};
        test_anim_content("./tests/chip_lossless.webp", 12973136813834081985, &f);
        test_anim_content("./tests/chip_lossy.webp", 12246682744563139019, &f);
        test_anim_content("./tests/alpha_no_compression.webp", 1003779282449725241, &f);
        test_anim_content("./tests/lossy_alpha1.webp", 13574948133336439503, &f);
    }
    #[test]
    fn test_BGRA() {
        let f = |o: &mut WebPAnimDecoderOptions| {o.color_mode = MODE_BGRA;};
        test_anim_content("./tests/chip_lossless.webp", 2075130756176543755, &f);
        test_anim_content("./tests/chip_lossy.webp", 6955902631319670855, &f);
        test_anim_content("./tests/lossy_alpha1.webp", 3005511504114414934, &f);
    }

    #[test]
    #[cfg(all(feature = "0_5", feature = "demux"))]
    fn test_anim_decoder() {
        let f = |_o: &mut WebPAnimDecoderOptions| {};
        // A short clip of my cat Chip, converted with ffmpeg
        test_anim_content("./tests/chip_lossless.webp", 12973136813834081985, &f);
        test_anim_content("./tests/chip_lossy.webp", 12246682744563139019, &f);

        // Taken from original libwebsys2 test image
        test_anim_content("./tests/animated.webp", 11144901834580975337, &f);

        // Taken from https://chromium.googlesource.com/webm/libwebp-test-data/
        test_anim_content("./tests/alpha_color_cache.webp", 7139075528299486749, &f);
        test_anim_content("./tests/alpha_filter_0_method_0.webp", 269959336470181749, &f);
        test_anim_content("./tests/alpha_filter_0_method_1.webp", 269959336470181749, &f);
        test_anim_content("./tests/alpha_filter_1.webp", 11601176151118067127, &f);
        test_anim_content("./tests/alpha_filter_1_method_0.webp", 269959336470181749, &f);
        test_anim_content("./tests/alpha_filter_1_method_1.webp", 269959336470181749, &f);
        test_anim_content("./tests/alpha_filter_2.webp", 11601176151118067127, &f);
        test_anim_content("./tests/alpha_filter_2_method_0.webp", 269959336470181749, &f);
        test_anim_content("./tests/alpha_filter_2_method_1.webp", 269959336470181749, &f);
        test_anim_content("./tests/alpha_filter_3.webp", 11601176151118067127, &f);
        test_anim_content("./tests/alpha_filter_3_method_0.webp", 269959336470181749, &f);
        test_anim_content("./tests/alpha_filter_3_method_1.webp", 269959336470181749, &f);
        test_anim_content("./tests/alpha_no_compression.webp", 11601176151118067127, &f);
        test_anim_content("./tests/bad_palette_index.webp", 3179927305885268547, &f);
        test_anim_content("./tests/big_endian_bug_393.webp", 15821156140267989026, &f);
        test_anim_content("./tests/bryce.webp", 2173216723474731973, &f);
        test_anim_content("./tests/bug3.webp", 16901908562088909750, &f);
        test_anim_content("./tests/color_cache_bits_11.webp", 15805786773633174689, &f);
        test_anim_content("./tests/dual_transform.webp", 18047906347243022880, &f);
        test_anim_content("./tests/lossless1.webp", 17768302409132818389, &f);
        test_anim_content("./tests/lossless2.webp", 17768302409132818389, &f);
        test_anim_content("./tests/lossless3.webp", 17768302409132818389, &f);
        test_anim_content("./tests/lossless4.webp", 16896323752254667796, &f);
        test_anim_content("./tests/lossless_big_random_alpha.webp", 8581210773495775950, &f);
        test_anim_content("./tests/lossless_color_transform.webp", 17741267576952510011, &f);
        test_anim_content("./tests/lossless_vec_1_0.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_1.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_10.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_11.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_12.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_13.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_14.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_15.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_2.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_3.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_4.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_5.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_6.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_7.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_8.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_1_9.webp", 11415221916290673081, &f);
        test_anim_content("./tests/lossless_vec_2_0.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_1.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_10.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_11.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_12.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_13.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_14.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_15.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_2.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_3.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_4.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_5.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_6.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_7.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_8.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossless_vec_2_9.webp", 18285264984328656172, &f);
        test_anim_content("./tests/lossy_alpha1.webp", 8114746806645667275, &f);
        test_anim_content("./tests/lossy_alpha2.webp", 15976630832270258083, &f);
        test_anim_content("./tests/lossy_alpha3.webp", 4395780881492938996, &f);
        test_anim_content("./tests/lossy_alpha4.webp", 1014717458192652218, &f);
        test_anim_content("./tests/lossy_extreme_probabilities.webp", 8920636990862365503, &f);
        test_anim_content("./tests/lossy_q0_f100.webp", 3966577919782747309, &f);
        test_anim_content("./tests/near_lossless_75.webp", 16825044650563630690, &f);
        test_anim_content("./tests/one_color_no_palette.webp", 12998824128782987327, &f);
        test_anim_content("./tests/segment01.webp", 15052450272839161463, &f);
        test_anim_content("./tests/segment02.webp", 14056791416036260084, &f);
        test_anim_content("./tests/segment03.webp", 8270799958870222402, &f);
        test_anim_content("./tests/small_13x1.webp", 1511617637893141007, &f);
        test_anim_content("./tests/small_1x1.webp", 7555012243442796334, &f);
        test_anim_content("./tests/small_1x13.webp", 12944935366533174067, &f);
        test_anim_content("./tests/small_31x13.webp", 10445038715640958683, &f);
        test_anim_content("./tests/test-nostrong.webp", 2541419939380451108, &f);
        test_anim_content("./tests/test.webp", 15805786773633174689, &f);
        test_anim_content("./tests/very_short.webp", 11089220480579370369, &f);
        test_anim_content("./tests/vp80-00-comprehensive-001.webp", 8922955068206135632, &f);
        test_anim_content("./tests/vp80-00-comprehensive-002.webp", 3996769244644888415, &f);
        test_anim_content("./tests/vp80-00-comprehensive-003.webp", 8980587458809154287, &f);
        test_anim_content("./tests/vp80-00-comprehensive-004.webp", 8922955068206135632, &f);
        test_anim_content("./tests/vp80-00-comprehensive-005.webp", 8770929085855477608, &f);
        test_anim_content("./tests/vp80-00-comprehensive-006.webp", 8757280669970610939, &f);
        test_anim_content("./tests/vp80-00-comprehensive-007.webp", 5890312307673978367, &f);
        test_anim_content("./tests/vp80-00-comprehensive-008.webp", 7990464531258155155, &f);
        test_anim_content("./tests/vp80-00-comprehensive-009.webp", 7422481547116844829, &f);
        test_anim_content("./tests/vp80-00-comprehensive-010.webp", 8076175088563676894, &f);
        test_anim_content("./tests/vp80-00-comprehensive-011.webp", 8922955068206135632, &f);
        test_anim_content("./tests/vp80-00-comprehensive-012.webp", 7857227984245784193, &f);
        test_anim_content("./tests/vp80-00-comprehensive-013.webp", 3633754347151800764, &f);
        test_anim_content("./tests/vp80-00-comprehensive-014.webp", 13748566424954352193, &f);
        test_anim_content("./tests/vp80-00-comprehensive-015.webp", 5620003810901009367, &f);
        test_anim_content("./tests/vp80-00-comprehensive-016.webp", 5056416415950061836, &f);
        test_anim_content("./tests/vp80-00-comprehensive-017.webp", 5056416415950061836, &f);
        test_anim_content("./tests/vp80-01-intra-1400.webp", 8224310502037935891, &f);
        test_anim_content("./tests/vp80-01-intra-1411.webp", 17587853229421624357, &f);
        test_anim_content("./tests/vp80-01-intra-1416.webp", 5493569529945269149, &f);
        test_anim_content("./tests/vp80-01-intra-1417.webp", 14121426666263385093, &f);
        test_anim_content("./tests/vp80-02-inter-1402.webp", 8224310502037935891, &f);
        test_anim_content("./tests/vp80-02-inter-1412.webp", 17587853229421624357, &f);
        test_anim_content("./tests/vp80-02-inter-1418.webp", 1804303511074854046, &f);
        test_anim_content("./tests/vp80-02-inter-1424.webp", 106984724960440457, &f);
        test_anim_content("./tests/vp80-03-segmentation-1401.webp", 8224310502037935891, &f);
        test_anim_content("./tests/vp80-03-segmentation-1403.webp", 8224310502037935891, &f);
        test_anim_content("./tests/vp80-03-segmentation-1407.webp", 18431957657024090961, &f);
        test_anim_content("./tests/vp80-03-segmentation-1408.webp", 18431957657024090961, &f);
        test_anim_content("./tests/vp80-03-segmentation-1409.webp", 18431957657024090961, &f);
        test_anim_content("./tests/vp80-03-segmentation-1410.webp", 18431957657024090961, &f);
        test_anim_content("./tests/vp80-03-segmentation-1413.webp", 17587853229421624357, &f);
        test_anim_content("./tests/vp80-03-segmentation-1414.webp", 18058704918143585599, &f);
        test_anim_content("./tests/vp80-03-segmentation-1415.webp", 18058704918143585599, &f);
        test_anim_content("./tests/vp80-03-segmentation-1425.webp", 15541210716571025647, &f);
        test_anim_content("./tests/vp80-03-segmentation-1426.webp", 9445883879984713016, &f);
        test_anim_content("./tests/vp80-03-segmentation-1427.webp", 13406639971166807563, &f);
        test_anim_content("./tests/vp80-03-segmentation-1432.webp", 1634009582774549850, &f);
        test_anim_content("./tests/vp80-03-segmentation-1435.webp", 9794906891697763636, &f);
        test_anim_content("./tests/vp80-03-segmentation-1436.webp", 7637298221140974699, &f);
        test_anim_content("./tests/vp80-03-segmentation-1437.webp", 7005631426988700615, &f);
        test_anim_content("./tests/vp80-03-segmentation-1441.webp", 6301413929326353435, &f);
        test_anim_content("./tests/vp80-03-segmentation-1442.webp", 13563681927532419514, &f);
        test_anim_content("./tests/vp80-04-partitions-1404.webp", 8224310502037935891, &f);
        test_anim_content("./tests/vp80-04-partitions-1405.webp", 8224310502037935891, &f);
        test_anim_content("./tests/vp80-04-partitions-1406.webp", 8224310502037935891, &f);
        test_anim_content("./tests/vp80-05-sharpness-1428.webp", 6369621127819382881, &f);
        test_anim_content("./tests/vp80-05-sharpness-1429.webp", 5736996487575508191, &f);
        test_anim_content("./tests/vp80-05-sharpness-1430.webp", 13927537323346367419, &f);
        test_anim_content("./tests/vp80-05-sharpness-1431.webp", 14288472789759021377, &f);
        test_anim_content("./tests/vp80-05-sharpness-1433.webp", 7637298221140974699, &f);
        test_anim_content("./tests/vp80-05-sharpness-1434.webp", 13643709644326139354, &f);
        test_anim_content("./tests/vp80-05-sharpness-1438.webp", 13962555527159636325, &f);
        test_anim_content("./tests/vp80-05-sharpness-1439.webp", 7684010787539890554, &f);
        test_anim_content("./tests/vp80-05-sharpness-1440.webp", 7637298221140974699, &f);
        test_anim_content("./tests/vp80-05-sharpness-1443.webp", 14626837624944389773, &f);

    }
}

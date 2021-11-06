use std::os::raw::*;
use std::ptr;

// MAJOR(8b) + MINOR(8b)
cfg_if! {
    if #[cfg(feature = "1_1")] {
        pub const WEBP_DECODER_ABI_VERSION: c_int = 0x0209;
    } else if #[cfg(feature = "0_5")] {
        pub const WEBP_DECODER_ABI_VERSION: c_int = 0x0208;
    } else {
        pub const WEBP_DECODER_ABI_VERSION: c_int = 0x0203;
    }
}

#[cfg(feature = "extern-types")]
extern "C" {
    pub type WebPIDecoder;
}

#[cfg(not(feature = "extern-types"))]
#[repr(C)]
pub struct WebPIDecoder(c_void);

// Colorspaces
// Note: the naming describes the byte-ordering of packed samples in memory.
// For instance, MODE_BGRA relates to samples ordered as B,G,R,A,B,G,R,A,...
// Non-capital names (e.g.:MODE_Argb) relates to pre-multiplied RGB channels.
// RGBA-4444 and RGB-565 colorspaces are represented by following byte-order:
// RGBA-4444: [r3 r2 r1 r0 g3 g2 g1 g0], [b3 b2 b1 b0 a3 a2 a1 a0], ...
// RGB-565: [r4 r3 r2 r1 r0 g5 g4 g3], [g2 g1 g0 b4 b3 b2 b1 b0], ...
// In the case WEBP_SWAP_16BITS_CSP is defined, the bytes are swapped for
// these two modes:
// RGBA-4444: [b3 b2 b1 b0 a3 a2 a1 a0], [r3 r2 r1 r0 g3 g2 g1 g0], ...
// RGB-565: [g2 g1 g0 b4 b3 b2 b1 b0], [r4 r3 r2 r1 r0 g5 g4 g3], ...

#[allow(non_camel_case_types)]
pub type WEBP_CSP_MODE = u32;

pub const MODE_RGB: WEBP_CSP_MODE = 0;
pub const MODE_RGBA: WEBP_CSP_MODE = 1;
pub const MODE_BGR: WEBP_CSP_MODE = 2;
pub const MODE_BGRA: WEBP_CSP_MODE = 3;
pub const MODE_ARGB: WEBP_CSP_MODE = 4;
pub const MODE_RGBA_4444: WEBP_CSP_MODE = 5;
pub const MODE_RGB_565: WEBP_CSP_MODE = 6;
// RGB-premultiplied transparent modes (alpha value is preserved)
#[allow(non_upper_case_globals)]
pub const MODE_rgbA: WEBP_CSP_MODE = 7;
#[allow(non_upper_case_globals)]
pub const MODE_bgrA: WEBP_CSP_MODE = 8;
#[allow(non_upper_case_globals)]
pub const MODE_Argb: WEBP_CSP_MODE = 9;
#[allow(non_upper_case_globals)]
pub const MODE_rgbA_4444: WEBP_CSP_MODE = 10;
// YUV modes must come after RGB ones.
pub const MODE_YUV: WEBP_CSP_MODE = 11;
pub const MODE_YUVA: WEBP_CSP_MODE = 12;
pub const MODE_LAST: WEBP_CSP_MODE = 13;

// Some useful macros:

#[allow(non_snake_case)]
#[inline]
pub extern "C" fn WebPIsPremultipliedMode(mode: WEBP_CSP_MODE) -> c_int {
    (mode == MODE_rgbA || mode == MODE_bgrA || mode == MODE_Argb || mode == MODE_rgbA_4444) as c_int
}

#[allow(non_snake_case)]
#[inline]
pub extern "C" fn WebPIsAlphaMode(mode: WEBP_CSP_MODE) -> c_int {
    (mode == MODE_RGBA
        || mode == MODE_BGRA
        || mode == MODE_ARGB
        || mode == MODE_RGBA_4444
        || mode == MODE_YUVA
        || WebPIsPremultipliedMode(mode) != 0) as c_int
}

#[allow(non_snake_case)]
#[inline]
pub extern "C" fn WebPIsRGBMode(mode: WEBP_CSP_MODE) -> c_int {
    (mode < MODE_YUV) as c_int
}

//------------------------------------------------------------------------------
// WebPDecBuffer: Generic structure for describing the output sample buffer.

/// view as RGBA
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WebPRGBABuffer {
    /// pointer to RGBA samples
    pub rgba: *mut u8,
    /// stride in bytes from one scanline to the next.
    pub stride: c_int,
    /// total size of the *rgba buffer.
    pub size: usize,
}

/// view as YUVA
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WebPYUVABuffer {
    /// pointer to luma samples
    pub y: *mut u8,
    /// pointer to chroma U samples
    pub u: *mut u8,
    /// pointer to chroma V samples
    pub v: *mut u8,
    /// pointer to alpha samples
    pub a: *mut u8,
    /// luma stride
    pub y_stride: c_int,
    /// chroma U stride
    pub u_stride: c_int,
    /// chroma V stride
    pub v_stride: c_int,
    /// alpha stride
    pub a_stride: c_int,
    /// luma plane size
    pub y_size: usize,
    /// chroma U plane size
    pub u_size: usize,
    /// chroma V planes size
    pub v_size: usize,
    /// alpha-plane size
    pub a_size: usize,
}

/// Output buffer
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WebPDecBuffer {
    /// Colorspace.
    pub colorspace: WEBP_CSP_MODE,
    /// Dimension (width).
    pub width: c_int,
    /// Dimension (height).
    pub height: c_int,
    /// If non-zero, 'internal_memory' pointer is not
    /// used. If value is '2' or more, the external
    /// memory is considered 'slow' and multiple
    /// read/write will be avoided.
    pub is_external_memory: c_int,
    /// Nameless union of buffer parameters.
    pub u: __WebPDecBufferUnion,
    /// padding for later use
    pub pad: [u32; 4],
    /// Internally allocated memory (only when
    /// is_external_memory is 0). Should not be used
    /// externally, but accessed via the buffer union.
    #[doc(hidden)]
    pub private_memory: *mut u8,
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Clone, Copy)]
pub union __WebPDecBufferUnion {
    pub RGBA: WebPRGBABuffer,
    pub YUVA: WebPYUVABuffer,
}

impl std::fmt::Debug for __WebPDecBufferUnion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("<union>")
    }
}

/// Enumeration of the status codes
#[allow(non_camel_case_types)]
pub type VP8StatusCode = u32;

pub const VP8_STATUS_OK: VP8StatusCode = 0;
pub const VP8_STATUS_OUT_OF_MEMORY: VP8StatusCode = 1;
pub const VP8_STATUS_INVALID_PARAM: VP8StatusCode = 2;
pub const VP8_STATUS_BITSTREAM_ERROR: VP8StatusCode = 3;
pub const VP8_STATUS_UNSUPPORTED_FEATURE: VP8StatusCode = 4;
pub const VP8_STATUS_SUSPENDED: VP8StatusCode = 5;
pub const VP8_STATUS_USER_ABORT: VP8StatusCode = 6;
pub const VP8_STATUS_NOT_ENOUGH_DATA: VP8StatusCode = 7;

/// Deprecated alpha-less version of WebPIDecGetYUVA(): it will ignore the
/// alpha information (if present). Kept for backward compatibility.
#[allow(non_snake_case)]
#[inline]
pub unsafe extern "C" fn WebPIDecGetYUV(
    idec: *const WebPIDecoder,
    last_y: *mut c_int,
    u: *mut *mut u8,
    v: *mut *mut u8,
    width: *mut c_int,
    height: *mut c_int,
    stride: *mut c_int,
    uv_stride: *mut c_int,
) -> *mut u8 {
    WebPIDecGetYUVA(
        idec,
        last_y,
        u,
        v,
        ptr::null_mut(),
        width,
        height,
        stride,
        uv_stride,
        ptr::null_mut(),
    )
}

/// Features gathered from the bitstream
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WebPBitstreamFeatures {
    /// Width in pixels, as read from the bitstream.
    pub width: c_int,
    /// Height in pixels, as read from the bitstream.
    pub height: c_int,
    /// True if the bitstream contains an alpha channel.
    pub has_alpha: c_int,
    /// True if the bitstream is an animation.
    pub has_animation: c_int,
    /// 0 = undefined (/mixed), 1 = lossy, 2 = lossless
    pub format: c_int,
    /// Unused for now. if true, using incremental decoding is not
    /// recommended.
    #[cfg(not(feature = "0_5"))]
    #[deprecated(note = "Removed as of libwebp 0.5.0")]
    pub no_incremental_decoding: c_int,
    /// Unused for now. TODO(later)
    #[cfg(not(feature = "0_5"))]
    #[deprecated(note = "Removed as of libwebp 0.5.0")]
    pub rotate: c_int,
    /// Unused for now. should be 0 for now. TODO(later)
    #[cfg(not(feature = "0_5"))]
    #[deprecated(note = "Removed as of libwebp 0.5.0")]
    pub uv_sampling: c_int,
    /// padding for later use
    #[cfg(not(feature = "0_5"))]
    #[doc(hidden)]
    pub pad: [u32; 2],
    /// padding for later use
    #[cfg(feature = "0_5")]
    #[doc(hidden)]
    pub pad: [u32; 5],
}

/// Decoding options
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WebPDecoderOptions {
    /// if true, skip the in-loop filtering
    pub bypass_filtering: c_int,
    /// if true, use faster pointwise upsampler
    pub no_fancy_upsampling: c_int,
    /// if true, cropping is applied _first_
    pub use_cropping: c_int,
    /// left position for cropping.
    /// Will be snapped to even value.
    pub crop_left: c_int,
    /// top position for cropping.
    /// Will be snapped to even value.
    pub crop_top: c_int,
    /// width of the cropping area
    pub crop_width: c_int,
    /// height of the cropping area
    pub crop_height: c_int,
    /// if true, scaling is applied _afterward_
    pub use_scaling: c_int,
    /// final resolution width
    pub scaled_width: c_int,
    /// final resolution height
    pub scaled_height: c_int,
    /// if true, use multi-threaded decoding
    pub use_threads: c_int,
    /// dithering strength (0=Off, 100=full)
    pub dithering_strength: c_int,
    /// if true, flip output vertically
    #[cfg(feature = "0_5")]
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "0_5")))]
    pub flip: c_int,
    /// alpha dithering strength in [0..100]
    #[cfg(feature = "0_5")]
    #[cfg_attr(feature = "__doc_cfg", doc(cfg(feature = "0_5")))]
    pub alpha_dithering_strength: c_int,
    /// Unused for now. forced rotation (to be applied _last_)
    #[cfg(not(feature = "0_5"))]
    #[deprecated(note = "Removed as of libwebp 0.5.0")]
    pub force_rotation: c_int,
    /// Unused for now. if true, discard enhancement layer
    #[cfg(not(feature = "0_5"))]
    #[deprecated(note = "Removed as of libwebp 0.5.0")]
    pub no_enhancement: c_int,
    /// padding for later use
    #[doc(hidden)]
    pub pad: [u32; 5],
}

/// Main object storing the configuration for advanced decoding.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WebPDecoderConfig {
    /// Immutable bitstream features (optional)
    pub input: WebPBitstreamFeatures,
    /// Output buffer (can point to external mem)
    pub output: WebPDecBuffer,
    /// Decoding options
    pub options: WebPDecoderOptions,
}

extern "C" {
    /// Return the decoder's version number, packed in hexadecimal using 8bits for
    /// each of major/minor/revision. E.g: v2.5.7 is 0x020507.
    pub fn WebPGetDecoderVersion() -> c_int;
    /// Retrieve basic header information: width, height.
    /// This function will also validate the header, returning true on success,
    /// false otherwise. '*width' and '*height' are only valid on successful return.
    /// Pointers 'width' and 'height' can be passed NULL if deemed irrelevant.
    /// Note: The following chunk sequences (before the raw VP8/VP8L data) are
    /// considered valid by this function:
    /// RIFF + VP8(L)
    /// RIFF + VP8X + (optional chunks) + VP8(L)
    /// ALPH + VP8 <-- Not a valid WebP format: only allowed for internal purpose.
    /// VP8(L)     <-- Not a valid WebP format: only allowed for internal purpose.
    pub fn WebPGetInfo(
        data: *const u8,
        data_size: usize,
        width: *mut c_int,
        height: *mut c_int,
    ) -> c_int;
    /// Decodes WebP images pointed to by 'data' and returns RGBA samples, along
    /// with the dimensions in *width and *height. The ordering of samples in
    /// memory is R, G, B, A, R, G, B, A... in scan order (endian-independent).
    /// The returned pointer should be deleted calling WebPFree().
    /// Returns NULL in case of error.
    pub fn WebPDecodeRGBA(
        data: *const u8,
        data_size: usize,
        width: *mut c_int,
        height: *mut c_int,
    ) -> *mut u8;
    /// Same as WebPDecodeRGBA, but returning A, R, G, B, A, R, G, B... ordered data.
    pub fn WebPDecodeARGB(
        data: *const u8,
        data_size: usize,
        width: *mut c_int,
        height: *mut c_int,
    ) -> *mut u8;
    /// Same as WebPDecodeRGBA, but returning B, G, R, A, B, G, R, A... ordered data.
    pub fn WebPDecodeBGRA(
        data: *const u8,
        data_size: usize,
        width: *mut c_int,
        height: *mut c_int,
    ) -> *mut u8;
    /// Same as WebPDecodeRGBA, but returning R, G, B, R, G, B... ordered data.
    /// If the bitstream contains transparency, it is ignored.
    pub fn WebPDecodeRGB(
        data: *const u8,
        data_size: usize,
        width: *mut c_int,
        height: *mut c_int,
    ) -> *mut u8;
    /// Same as WebPDecodeRGB, but returning B, G, R, B, G, R... ordered data.
    pub fn WebPDecodeBGR(
        data: *const u8,
        data_size: usize,
        width: *mut c_int,
        height: *mut c_int,
    ) -> *mut u8;
    /// Decode WebP images pointed to by 'data' to Y'UV format(*). The pointer
    /// returned is the Y samples buffer. Upon return, *u and *v will point to
    /// the U and V chroma data. These U and V buffers need NOT be passed to
    /// WebPFree(), unlike the returned Y luma one. The dimension of the U and V
    /// planes are both (*width + 1) / 2 and (*height + 1)/ 2.
    /// Upon return, the Y buffer has a stride returned as '*stride', while U and V
    /// have a common stride returned as '*uv_stride'.
    /// Return NULL in case of error.
    /// (*) Also named Y'CbCr. See: http://en.wikipedia.org/wiki/YCbCr
    pub fn WebPDecodeYUV(
        data: *const u8,
        data_size: usize,
        width: *mut c_int,
        height: *mut c_int,
        u: *mut *mut u8,
        v: *mut *mut u8,
        stride: *mut c_int,
        uv_stride: *mut c_int,
    ) -> *mut u8;
    // These five functions are variants of the above ones, that decode the image
    // directly into a pre-allocated buffer 'output_buffer'. The maximum storage
    // available in this buffer is indicated by 'output_buffer_size'. If this
    // storage is not sufficient (or an error occurred), NULL is returned.
    // Otherwise, output_buffer is returned, for convenience.
    // The parameter 'output_stride' specifies the distance (in bytes)
    // between scanlines. Hence, output_buffer_size is expected to be at least
    // output_stride x picture-height.
    pub fn WebPDecodeRGBAInto(
        data: *const u8,
        data_size: usize,
        output_buffer: *mut u8,
        output_buffer_size: usize,
        output_stride: c_int,
    ) -> *mut u8;
    pub fn WebPDecodeARGBInto(
        data: *const u8,
        data_size: usize,
        output_buffer: *mut u8,
        output_buffer_size: usize,
        output_stride: c_int,
    ) -> *mut u8;
    pub fn WebPDecodeBGRAInto(
        data: *const u8,
        data_size: usize,
        output_buffer: *mut u8,
        output_buffer_size: usize,
        output_stride: c_int,
    ) -> *mut u8;
    // RGB and BGR variants. Here too the transparency information, if present,
    // will be dropped and ignored.
    pub fn WebPDecodeRGBInto(
        data: *const u8,
        data_size: usize,
        output_buffer: *mut u8,
        output_buffer_size: usize,
        output_stride: c_int,
    ) -> *mut u8;
    pub fn WebPDecodeBGRInto(
        data: *const u8,
        data_size: usize,
        output_buffer: *mut u8,
        output_buffer_size: usize,
        output_stride: c_int,
    ) -> *mut u8;
    /// WebPDecodeYUVInto() is a variant of WebPDecodeYUV() that operates directly
    /// into pre-allocated luma/chroma plane buffers. This function requires the
    /// strides to be passed: one for the luma plane and one for each of the
    /// chroma ones. The size of each plane buffer is passed as 'luma_size',
    /// 'u_size' and 'v_size' respectively.
    /// Pointer to the luma plane ('*luma') is returned or NULL if an error occurred
    /// during decoding (or because some buffers were found to be too small).
    pub fn WebPDecodeYUVInto(
        data: *const u8,
        data_size: usize,
        luma: *mut u8,
        luma_size: usize,
        luma_stride: c_int,
        u: *mut u8,
        u_size: usize,
        u_stride: c_int,
        v: *mut u8,
        v_size: usize,
        v_stride: c_int,
    ) -> *mut u8;
    /// Internal, version-checked, entry point
    #[doc(hidden)]
    pub fn WebPInitDecBufferInternal(_: *mut WebPDecBuffer, _: c_int) -> c_int;
    /// Free any memory associated with the buffer. Must always be called last.
    /// Note: doesn't free the 'buffer' structure itself.
    pub fn WebPFreeDecBuffer(buffer: *mut WebPDecBuffer);
    /// Creates a new incremental decoder with the supplied buffer parameter.
    /// This output_buffer can be passed NULL, in which case a default output buffer
    /// is used (with MODE_RGB). Otherwise, an internal reference to 'output_buffer'
    /// is kept, which means that the lifespan of 'output_buffer' must be larger than
    /// that of the returned WebPIDecoder object.
    /// The supplied 'output_buffer' content MUST NOT be changed between calls to
    /// WebPIAppend() or WebPIUpdate() unless 'output_buffer.is_external_memory' is
    /// not set to 0. In such a case, it is allowed to modify the pointers, size and
    /// stride of output_buffer.u.RGBA or output_buffer.u.YUVA, provided they remain
    /// within valid bounds.
    /// All other fields of WebPDecBuffer MUST remain constant between calls.
    /// Returns NULL if the allocation failed.
    pub fn WebPINewDecoder(output_buffer: *mut WebPDecBuffer) -> *mut WebPIDecoder;
    /// This function allocates and initializes an incremental-decoder object, which
    /// will output the RGB/A samples specified by 'csp' into a preallocated
    /// buffer 'output_buffer'. The size of this buffer is at least
    /// 'output_buffer_size' and the stride (distance in bytes between two scanlines)
    /// is specified by 'output_stride'.
    /// Additionally, output_buffer can be passed NULL in which case the output
    /// buffer will be allocated automatically when the decoding starts. The
    /// colorspace 'csp' is taken into account for allocating this buffer. All other
    /// parameters are ignored.
    /// Returns NULL if the allocation failed, or if some parameters are invalid.
    pub fn WebPINewRGB(
        csp: WEBP_CSP_MODE,
        output_buffer: *mut u8,
        output_buffer_size: usize,
        output_stride: c_int,
    ) -> *mut WebPIDecoder;
    /// This function allocates and initializes an incremental-decoder object, which
    /// will output the raw luma/chroma samples into a preallocated planes if
    /// supplied. The luma plane is specified by its pointer 'luma', its size
    /// 'luma_size' and its stride 'luma_stride'. Similarly, the chroma-u plane
    /// is specified by the 'u', 'u_size' and 'u_stride' parameters, and the chroma-v
    /// plane by 'v' and 'v_size'. And same for the alpha-plane. The 'a' pointer
    /// can be pass NULL in case one is not interested in the transparency plane.
    /// Conversely, 'luma' can be passed NULL if no preallocated planes are supplied.
    /// In this case, the output buffer will be automatically allocated (using
    /// MODE_YUVA) when decoding starts. All parameters are then ignored.
    /// Returns NULL if the allocation failed or if a parameter is invalid.
    pub fn WebPINewYUVA(
        luma: *mut u8,
        luma_size: usize,
        luma_stride: c_int,
        u: *mut u8,
        u_size: usize,
        u_stride: c_int,
        v: *mut u8,
        v_size: usize,
        v_stride: c_int,
        a: *mut u8,
        a_size: usize,
        a_stride: c_int,
    ) -> *mut WebPIDecoder;
    /// Deprecated version of the above, without the alpha plane.
    /// Kept for backward compatibility.
    pub fn WebPINewYUV(
        luma: *mut u8,
        luma_size: usize,
        luma_stride: c_int,
        u: *mut u8,
        u_size: usize,
        u_stride: c_int,
        v: *mut u8,
        v_size: usize,
        v_stride: c_int,
    ) -> *mut WebPIDecoder;
    /// Deletes the WebPIDecoder object and associated memory. Must always be called
    /// if WebPINewDecoder, WebPINewRGB or WebPINewYUV succeeded.
    pub fn WebPIDelete(idec: *mut WebPIDecoder);
    /// Copies and decodes the next available data. Returns VP8_STATUS_OK when
    /// the image is successfully decoded. Returns VP8_STATUS_SUSPENDED when more
    /// data is expected. Returns error in other cases.
    pub fn WebPIAppend(idec: *mut WebPIDecoder, data: *const u8, data_size: usize)
        -> VP8StatusCode;
    /// A variant of the above function to be used when data buffer contains
    /// partial data from the beginning. In this case data buffer is not copied
    /// to the internal memory.
    /// Note that the value of the 'data' pointer can change between calls to
    /// WebPIUpdate, for instance when the data buffer is resized to fit larger data.
    pub fn WebPIUpdate(idec: *mut WebPIDecoder, data: *const u8, data_size: usize)
        -> VP8StatusCode;
    /// Returns the RGB/A image decoded so far. Returns NULL if output params
    /// are not initialized yet. The RGB/A output type corresponds to the colorspace
    /// specified during call to WebPINewDecoder() or WebPINewRGB().
    /// *last_y is the index of last decoded row in raster scan order. Some pointers
    /// (*last_y, *width etc.) can be NULL if corresponding information is not
    /// needed. The values in these pointers are only valid on successful (non-NULL)
    /// return.
    pub fn WebPIDecGetRGB(
        idec: *const WebPIDecoder,
        last_y: *mut c_int,
        width: *mut c_int,
        height: *mut c_int,
        stride: *mut c_int,
    ) -> *mut u8;
    /// Same as above function to get a YUVA image. Returns pointer to the luma
    /// plane or NULL in case of error. If there is no alpha information
    /// the alpha pointer '*a' will be returned NULL.
    pub fn WebPIDecGetYUVA(
        idec: *const WebPIDecoder,
        last_y: *mut c_int,
        u: *mut *mut u8,
        v: *mut *mut u8,
        a: *mut *mut u8,
        width: *mut c_int,
        height: *mut c_int,
        stride: *mut c_int,
        uv_stride: *mut c_int,
        a_stride: *mut c_int,
    ) -> *mut u8;
    /// Generic call to retrieve information about the displayable area.
    /// If non NULL, the left/right/width/height pointers are filled with the visible
    /// rectangular area so far.
    /// Returns NULL in case the incremental decoder object is in an invalid state.
    /// Otherwise returns the pointer to the internal representation. This structure
    /// is read-only, tied to WebPIDecoder's lifespan and should not be modified.
    pub fn WebPIDecodedArea(
        idec: *const WebPIDecoder,
        left: *mut c_int,
        top: *mut c_int,
        width: *mut c_int,
        height: *mut c_int,
    ) -> *const WebPDecBuffer;
    /// Internal, version-checked, entry point
    #[doc(hidden)]
    pub fn WebPGetFeaturesInternal(
        _: *const u8,
        _: usize,
        _: *mut WebPBitstreamFeatures,
        _: c_int,
    ) -> VP8StatusCode;
    /// Internal, version-checked, entry point
    #[doc(hidden)]
    pub fn WebPInitDecoderConfigInternal(_: *mut WebPDecoderConfig, _: c_int) -> c_int;
    /// Instantiate a new incremental decoder object with the requested
    /// configuration. The bitstream can be passed using 'data' and 'data_size'
    /// parameter, in which case the features will be parsed and stored into
    /// config->input. Otherwise, 'data' can be NULL and no parsing will occur.
    /// Note that 'config' can be NULL too, in which case a default configuration
    /// is used. If 'config' is not NULL, it must outlive the WebPIDecoder object
    /// as some references to its fields will be used. No internal copy of 'config'
    /// is made.
    /// The return WebPIDecoder object must always be deleted calling WebPIDelete().
    /// Returns NULL in case of error (and config->status will then reflect
    /// the error condition, if available).
    pub fn WebPIDecode(
        data: *const u8,
        data_size: usize,
        config: *mut WebPDecoderConfig,
    ) -> *mut WebPIDecoder;
    /// Non-incremental version. This version decodes the full data at once, taking
    /// 'config' into account. Returns decoding status (which should be VP8_STATUS_OK
    /// if the decoding was successful). Note that 'config' cannot be NULL.
    pub fn WebPDecode(
        data: *const u8,
        data_size: usize,
        config: *mut WebPDecoderConfig,
    ) -> VP8StatusCode;
}

/// Initialize the structure as empty. Must be called before any other use.
/// Returns false in case of version mismatch
#[allow(non_snake_case)]
#[inline]
pub unsafe extern "C" fn WebPInitDecBuffer(buffer: *mut WebPDecBuffer) -> c_int {
    WebPInitDecBufferInternal(buffer, WEBP_DECODER_ABI_VERSION)
}

/// Retrieve features from the bitstream. The *features structure is filled
/// with information gathered from the bitstream.
/// Returns VP8_STATUS_OK when the features are successfully retrieved. Returns
/// VP8_STATUS_NOT_ENOUGH_DATA when more data is needed to retrieve the
/// features from headers. Returns error in other cases.
/// Note: The following chunk sequences (before the raw VP8/VP8L data) are
/// considered valid by this function:
/// RIFF + VP8(L)
/// RIFF + VP8X + (optional chunks) + VP8(L)
/// ALPH + VP8 <-- Not a valid WebP format: only allowed for internal purpose.
/// VP8(L)     <-- Not a valid WebP format: only allowed for internal purpose.
#[allow(non_snake_case)]
#[inline]
pub unsafe extern "C" fn WebPGetFeatures(
    data: *const u8,
    data_size: usize,
    features: *mut WebPBitstreamFeatures,
) -> VP8StatusCode {
    WebPGetFeaturesInternal(data, data_size, features, WEBP_DECODER_ABI_VERSION)
}

/// Initialize the configuration as empty. This function must always be
/// called first, unless WebPGetFeatures() is to be called.
/// Returns false in case of mismatched version.
#[allow(non_snake_case)]
#[inline]
pub unsafe extern "C" fn WebPInitDecoderConfig(config: *mut WebPDecoderConfig) -> c_int {
    WebPInitDecoderConfigInternal(config, WEBP_DECODER_ABI_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_image_decode<F>(buf: &[u8], f: &F) -> u64
        where F: Fn(&mut WebPDecoderConfig) {
        use siphasher::sip::SipHasher24;
        use std::hash::{Hasher};
        use std::mem;

        let mut hasher = SipHasher24::new_with_keys(0xca8e6089151e54eb, 0x58dbee492c222104);

        unsafe {
            let mut config = mem::zeroed();
            assert!(WebPInitDecoderConfig(&mut config) != 0);

            f(&mut config);
            
            let decode_result = WebPDecode(buf.as_ptr(), buf.len(), &mut config);
            if decode_result == 4 {
                return 4;
            } else {
                assert_eq!(decode_result, 0);
            }
            // config.output.u is either WebPRGBABuffer (for all arrangements of r, g, b, a)
            // or WebPYUVABuffer (for all arrangements of y, u, v)
            match config.output.colorspace {
                MODE_RGB | MODE_RGBA | MODE_BGR | MODE_BGRA | 
                    MODE_ARGB | MODE_RGBA_4444 | MODE_RGB_565 | MODE_rgbA |
                    MODE_bgrA | MODE_Argb | MODE_rgbA_4444 => {
                    let output = std::slice::from_raw_parts(config.output.u.RGBA.rgba,
                        config.output.u.RGBA.size);
                    hasher.write(output);
                    hasher.write_i32(config.output.u.RGBA.stride);
                },
                MODE_YUV | MODE_YUVA => {
                    let yuva = &config.output.u.YUVA;
                    let y = std::slice::from_raw_parts(yuva.y, yuva.y_size);
                    let u = std::slice::from_raw_parts(yuva.u, yuva.u_size);
                    let v = std::slice::from_raw_parts(yuva.v, yuva.v_size);
                    let a = if yuva.a_size > 0 {
                        std::slice::from_raw_parts(yuva.a, yuva.a_size)
                    } else {
                        &[]
                    };
                    hasher.write(y);
                    hasher.write(u);
                    hasher.write(v);
                    hasher.write(a);
                    hasher.write_i32(yuva.y_stride);
                    hasher.write_i32(yuva.u_stride);
                    hasher.write_i32(yuva.v_stride);
                    if yuva.a_size > 0 {
                        hasher.write_i32(yuva.a_stride);
                    }
                }
                _ => unimplemented!()
            }
            // TODO: Free
        }
        hasher.finish()
    }

    fn test_image_content<F>(filename: &str, f: &F,
        expected_hash: u64)
        where F: Fn(&mut WebPDecoderConfig) {
        use std::fs::File;
        use std::io::prelude::*;
        let mut buf = Vec::new();
        let len = File::open(filename)
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        assert!(len > 0);
        let hash = hash_image_decode(&buf, f);
        assert_eq!(expected_hash, hash);          
        //println!(r#"test_image_content("{}", &f, {});"#, filename, hash);
    }

    #[test]
    fn test_scaled_RGB_565() {
        let f = |c: &mut WebPDecoderConfig| {
            c.options.use_scaling = 1;
            c.options.scaled_height = 400;
            c.options.scaled_width = 400;
            c.output.colorspace = MODE_RGB_565;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 3362874590192322579);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 1909703999792213912);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 1909703999792213912);
        test_image_content("./tests/alpha_filter_1.webp", &f, 3064861762824270074);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 1909703999792213912);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 1909703999792213912);
        test_image_content("./tests/alpha_filter_2.webp", &f, 3064861762824270074);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 1909703999792213912);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 1909703999792213912);
        test_image_content("./tests/alpha_filter_3.webp", &f, 3064861762824270074);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 1909703999792213912);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 1909703999792213912);
        test_image_content("./tests/alpha_no_compression.webp", &f, 3064861762824270074);
        test_image_content("./tests/bad_palette_index.webp", &f, 1039796273832814154);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 11514417314695366430);
        test_image_content("./tests/bryce.webp", &f, 9424316601144870331);
        test_image_content("./tests/bug3.webp", &f, 4114272377088361182);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 7662653068266931883);
        test_image_content("./tests/dual_transform.webp", &f, 3362874590192322579);
        test_image_content("./tests/lossless1.webp", &f, 2808818765881763031);
        test_image_content("./tests/lossless2.webp", &f, 2808818765881763031);
        test_image_content("./tests/lossless3.webp", &f, 2808818765881763031);
        test_image_content("./tests/lossless4.webp", &f, 14707525457041792885);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 5901571644370453900);
        test_image_content("./tests/lossless_color_transform.webp", &f, 11204354208857788511);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 14125959071146650681);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 7155605025314564034);
        test_image_content("./tests/lossy_alpha1.webp", &f, 5804511584899324773);
        test_image_content("./tests/lossy_alpha2.webp", &f, 3955792503643562129);
        test_image_content("./tests/lossy_alpha3.webp", &f, 18059161920282673246);
        test_image_content("./tests/lossy_alpha4.webp", &f, 3114225991933340356);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 5196360647712911037);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 1129822577732083145);
        test_image_content("./tests/near_lossless_75.webp", &f, 1557392329299404667);
        test_image_content("./tests/one_color_no_palette.webp", &f, 3362874590192322579);
        test_image_content("./tests/segment01.webp", &f, 8856073611100605858);
        test_image_content("./tests/segment02.webp", &f, 11170285631095198997);
        test_image_content("./tests/segment03.webp", &f, 12373825206718685478);
        test_image_content("./tests/small_13x1.webp", &f, 10658941058017031875);
        test_image_content("./tests/small_1x1.webp", &f, 9174692739531701724);
        test_image_content("./tests/small_1x13.webp", &f, 10658941058017031875);
        test_image_content("./tests/small_31x13.webp", &f, 157628097264599625);
        test_image_content("./tests/test-nostrong.webp", &f, 8738933980477106629);
        test_image_content("./tests/test.webp", &f, 6692350573871530425);
        test_image_content("./tests/very_short.webp", &f, 15540395332939296920);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 1159029284009229414);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 16794890060116716071);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 10071488079934980217);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 1159029284009229414);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 1113206330995191077);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 12255668059724652413);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 12091306914131341137);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 4130984800973612995);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 18177777130783101379);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 11637657417733497847);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 1159029284009229414);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 5207897250425578342);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 18015058328637917267);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 9032399222213241230);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 15014592317003063566);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 17133014618058014704);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 17133014618058014704);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 865247490068632228);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 3771869029110976616);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 4916650495083159007);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 17242635506565859476);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 865247490068632228);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 3771869029110976616);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 10058689508332540031);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 3874169860546469256);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 865247490068632228);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 865247490068632228);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 6001160199978915219);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 6001160199978915219);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 6001160199978915219);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 6001160199978915219);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 3771869029110976616);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 9111970219626862914);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 9111970219626862914);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 6730015891059528896);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 11576546781081034423);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 18217214766006765141);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 8523309808734147490);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 1221532542736219748);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 6476463657933917792);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 14444503487668946696);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 9341406962236957358);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 1406479206743951091);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 865247490068632228);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 865247490068632228);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 865247490068632228);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 14521661689793727555);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 7703500067348531638);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 3362874590192322579);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 7249747568033984155);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 6476463657933917792);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 16805103715734861934);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 17758948272677497423);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 12579129390370139029);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 6476463657933917792);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 3779154933549996974);    
    }

    #[test]
    fn test_scaled_RGBA_4444() {
        let f = |c: &mut WebPDecoderConfig| {
            c.options.use_scaling = 1;
            c.options.scaled_height = 400;
            c.options.scaled_width = 400;
            c.output.colorspace = MODE_RGBA_4444;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 13567106144408869704);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 16758806324644003033);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 16758806324644003033);
        test_image_content("./tests/alpha_filter_1.webp", &f, 12544352429605037279);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 16758806324644003033);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 16758806324644003033);
        test_image_content("./tests/alpha_filter_2.webp", &f, 12544352429605037279);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 16758806324644003033);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 16758806324644003033);
        test_image_content("./tests/alpha_filter_3.webp", &f, 12544352429605037279);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 16758806324644003033);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 16758806324644003033);
        test_image_content("./tests/alpha_no_compression.webp", &f, 12544352429605037279);
        test_image_content("./tests/bad_palette_index.webp", &f, 524350037031221097);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 14967082112246090715);
        test_image_content("./tests/bryce.webp", &f, 17345227571433590916);
        test_image_content("./tests/bug3.webp", &f, 6688128119218277928);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 10809947446105543331);
        test_image_content("./tests/dual_transform.webp", &f, 13352744858750099553);
        test_image_content("./tests/lossless1.webp", &f, 11340890447853671073);
        test_image_content("./tests/lossless2.webp", &f, 11340890447853671073);
        test_image_content("./tests/lossless3.webp", &f, 11340890447853671073);
        test_image_content("./tests/lossless4.webp", &f, 13731107752438708086);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 4798346347565721690);
        test_image_content("./tests/lossless_color_transform.webp", &f, 13193964222345453561);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 1728056406104855592);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 5061431495493730037);
        test_image_content("./tests/lossy_alpha1.webp", &f, 11528095206026251480);
        test_image_content("./tests/lossy_alpha2.webp", &f, 4604213662439131621);
        test_image_content("./tests/lossy_alpha3.webp", &f, 6207722046594935066);
        test_image_content("./tests/lossy_alpha4.webp", &f, 14716828447088814504);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 5663779854222277217);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 3913207914656132569);
        test_image_content("./tests/near_lossless_75.webp", &f, 4436764188976783974);
        test_image_content("./tests/one_color_no_palette.webp", &f, 3362874590192322579);
        test_image_content("./tests/segment01.webp", &f, 14544737780757494553);
        test_image_content("./tests/segment02.webp", &f, 1810518455857096361);
        test_image_content("./tests/segment03.webp", &f, 4991466145001928508);
        test_image_content("./tests/small_13x1.webp", &f, 5096027830163880431);
        test_image_content("./tests/small_1x1.webp", &f, 5096027830163880431);
        test_image_content("./tests/small_1x13.webp", &f, 5096027830163880431);
        test_image_content("./tests/small_31x13.webp", &f, 17723637039203954802);
        test_image_content("./tests/test-nostrong.webp", &f, 2901739820481737608);
        test_image_content("./tests/test.webp", &f, 6808146546860563860);
        test_image_content("./tests/very_short.webp", &f, 16767242939458359196);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 3325290972036796524);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 569380195698483439);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 14202768332370019674);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 3325290972036796524);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 16159375489948592089);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 2232926330782489133);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 5416524407645299922);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 4375407705310274834);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 14790027922304998027);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 4463323738221028428);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 3325290972036796524);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 5397724528811300127);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 10715277369323834968);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 5799427395461270267);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 6742270580363158979);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 9559386344242221029);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 9559386344242221029);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 13834942026576673848);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 12746397669550789262);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 14122287728486618025);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 2295544811636846658);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 13834942026576673848);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 12746397669550789262);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 13799594780358308581);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 13093099992688719953);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 13834942026576673848);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 13834942026576673848);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 16675142258521383898);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 16675142258521383898);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 16675142258521383898);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 16675142258521383898);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 12746397669550789262);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 6624498051341890051);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 6624498051341890051);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 3249680948014072719);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 13672772901327589632);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 13878764703411166659);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 16413224215383700046);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 9224629487325156814);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 6892077550731962792);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 13808231130457193103);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 4113167605220483400);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 13078124313123333525);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 13834942026576673848);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 13834942026576673848);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 13834942026576673848);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 18329006416949995203);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 10703191297755928719);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 9559386344242221029);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 13756483096501054842);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 6892077550731962792);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 12729323610725445993);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 16912447522316937584);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 12564563693432354435);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 6892077550731962792);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 1213056508599575394);
    }

    #[test]
    fn test_scaled_ARGB() {
        let f = |c: &mut WebPDecoderConfig| {
            c.options.use_scaling = 1;
            c.options.scaled_height = 400;
            c.options.scaled_width = 400;
            c.output.colorspace = MODE_ARGB;
        };         
        test_image_content("./tests/alpha_color_cache.webp", &f, 10864773716880263506);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 10937919634339270185);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 10937919634339270185);
        test_image_content("./tests/alpha_filter_1.webp", &f, 15670123612859723159);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 10937919634339270185);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 10937919634339270185);
        test_image_content("./tests/alpha_filter_2.webp", &f, 15670123612859723159);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 10937919634339270185);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 10937919634339270185);
        test_image_content("./tests/alpha_filter_3.webp", &f, 15670123612859723159);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 10937919634339270185);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 10937919634339270185);
        test_image_content("./tests/alpha_no_compression.webp", &f, 15670123612859723159);
        test_image_content("./tests/bad_palette_index.webp", &f, 3688448275686642341);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 1789406460786396674);
        test_image_content("./tests/bryce.webp", &f, 13245427998004925849);
        test_image_content("./tests/bug3.webp", &f, 7714069353107788415);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 11799245618226964137);
        test_image_content("./tests/dual_transform.webp", &f, 1843787497022892575);
        test_image_content("./tests/lossless1.webp", &f, 14822939775737379435);
        test_image_content("./tests/lossless2.webp", &f, 14822939775737379435);
        test_image_content("./tests/lossless3.webp", &f, 14822939775737379435);
        test_image_content("./tests/lossless4.webp", &f, 13061122887584362617);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 1074009677103671311);
        test_image_content("./tests/lossless_color_transform.webp", &f, 8427051048571544261);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 9809177418404841466);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 15823997873103304675);
        test_image_content("./tests/lossy_alpha1.webp", &f, 12610423287382679926);
        test_image_content("./tests/lossy_alpha2.webp", &f, 14712815170450479197);
        test_image_content("./tests/lossy_alpha3.webp", &f, 15428315403267248599);
        test_image_content("./tests/lossy_alpha4.webp", &f, 17851084686624974485);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 12746689036868021740);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 16382532931115689549);
        test_image_content("./tests/near_lossless_75.webp", &f, 4847635644719038708);
        test_image_content("./tests/one_color_no_palette.webp", &f, 8542849973809216197);
        test_image_content("./tests/segment01.webp", &f, 3653597489792206710);
        test_image_content("./tests/segment02.webp", &f, 2083065193208573593);
        test_image_content("./tests/segment03.webp", &f, 16826922750993230627);
        test_image_content("./tests/small_13x1.webp", &f, 17342864825137831248);
        test_image_content("./tests/small_1x1.webp", &f, 1689149048276793198);
        test_image_content("./tests/small_1x13.webp", &f, 1494622374557596785);
        test_image_content("./tests/small_31x13.webp", &f, 4947337295910507481);
        test_image_content("./tests/test-nostrong.webp", &f, 17856658020179350028);
        test_image_content("./tests/test.webp", &f, 13681608589491932);
        test_image_content("./tests/very_short.webp", &f, 12498913353894966402);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 605136080397955805);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 8086713523609779466);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 15442575095037393362);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 605136080397955805);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 777934696084831497);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 16561141470565286304);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 3325055096240454721);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 17965771093393652096);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 1925314651331860851);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 8266621038252666647);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 605136080397955805);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 12714578170140652783);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 13514831820531423977);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 15071444303831862951);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 8142682589057875462);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 6920909130175289902);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 6920909130175289902);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 10157611861532516754);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 2225465858122405799);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 8350192365344265461);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 9627664384013892654);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 10157611861532516754);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 2225465858122405799);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 4377787811593753051);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 2921588215717177483);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 10157611861532516754);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 10157611861532516754);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 11881751492547364385);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 11881751492547364385);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 11881751492547364385);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 11881751492547364385);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 2225465858122405799);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 979977489287057796);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 979977489287057796);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 9372916108361072474);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 18083248741073207699);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 3851873568749393014);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 8362346582730328912);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 10021552496832525651);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 13962492370479756359);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 8925227973449068720);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 16322765795707264783);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 3699166931433851094);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 10157611861532516754);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 10157611861532516754);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 10157611861532516754);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 16645852503299181975);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 5983441330247445427);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 3899788098488913740);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 16551078697746738160);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 13962492370479756359);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 5958241716175188212);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 13778087385470404288);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 1573282796306178902);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 13962492370479756359);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 8712024291074080166);
    }

    #[test]
    fn test_scaled_BGR() {
        let f = |c: &mut WebPDecoderConfig| {
            c.options.use_scaling = 1;
            c.options.scaled_height = 400;
            c.options.scaled_width = 400;
            c.output.colorspace = MODE_BGR;
        };   
        test_image_content("./tests/alpha_color_cache.webp", &f, 8496300406387420057);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 14112819971488801366);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 14112819971488801366);
        test_image_content("./tests/alpha_filter_1.webp", &f, 12559440765889342080);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 14112819971488801366);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 14112819971488801366);
        test_image_content("./tests/alpha_filter_2.webp", &f, 12559440765889342080);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 14112819971488801366);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 14112819971488801366);
        test_image_content("./tests/alpha_filter_3.webp", &f, 12559440765889342080);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 14112819971488801366);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 14112819971488801366);
        test_image_content("./tests/alpha_no_compression.webp", &f, 12559440765889342080);
        test_image_content("./tests/bad_palette_index.webp", &f, 4500192983895370222);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 5887041315799172152);
        test_image_content("./tests/bryce.webp", &f, 13614280541934987348);
        test_image_content("./tests/bug3.webp", &f, 17358144758644298617);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 2078697147175185505);
        test_image_content("./tests/dual_transform.webp", &f, 13514100800583353838);
        test_image_content("./tests/lossless1.webp", &f, 2380492484683551632);
        test_image_content("./tests/lossless2.webp", &f, 2380492484683551632);
        test_image_content("./tests/lossless3.webp", &f, 2380492484683551632);
        test_image_content("./tests/lossless4.webp", &f, 2235961130945775032);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 8821643653615121301);
        test_image_content("./tests/lossless_color_transform.webp", &f, 6062347040954988264);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 11341534252741764983);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 17755755758586072586);
        test_image_content("./tests/lossy_alpha1.webp", &f, 7024562613417019939);
        test_image_content("./tests/lossy_alpha2.webp", &f, 14763281430072891160);
        test_image_content("./tests/lossy_alpha3.webp", &f, 3267992117370934398);
        test_image_content("./tests/lossy_alpha4.webp", &f, 9245141526448032609);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 16751835892494856377);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 12209121583368959331);
        test_image_content("./tests/near_lossless_75.webp", &f, 5838076739906809206);
        test_image_content("./tests/one_color_no_palette.webp", &f, 13514100800583353838);
        test_image_content("./tests/segment01.webp", &f, 1276982448669263569);
        test_image_content("./tests/segment02.webp", &f, 9398874819253875204);
        test_image_content("./tests/segment03.webp", &f, 6975312473160963187);
        test_image_content("./tests/small_13x1.webp", &f, 18165445199902219607);
        test_image_content("./tests/small_1x1.webp", &f, 14245113078886307488);
        test_image_content("./tests/small_1x13.webp", &f, 6591505553100393654);
        test_image_content("./tests/small_31x13.webp", &f, 14087728855182016177);
        test_image_content("./tests/test-nostrong.webp", &f, 14543037108551997329);
        test_image_content("./tests/test.webp", &f, 17100676257361042808);
        test_image_content("./tests/very_short.webp", &f, 11032358957157142164);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 2135787761971947352);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 15419645145220319756);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 5986381883836324005);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 2135787761971947352);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 17363718530339388736);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 7965883562343682236);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 1707522966604974900);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 15168428636549111192);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 10192485192775138979);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 513407781561781302);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 2135787761971947352);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 5685380332875797799);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 11811150368806134196);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 17403308035800063710);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 1618022152290593291);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 13135636112833236445);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 13135636112833236445);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 4330231899916504777);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 12474451997994761192);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 4259918457746897438);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 6082589034653317676);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 4330231899916504777);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 12474451997994761192);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 3888035104480264496);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 8825515464083293311);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 4330231899916504777);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 4330231899916504777);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 1023195768577656520);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 1023195768577656520);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 1023195768577656520);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 1023195768577656520);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 12474451997994761192);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 17898237911885302907);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 17898237911885302907);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 9651432294900039728);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 18111760986758639472);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 8045751724065294576);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 1694032982755851615);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 16587919174011261578);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 11173026541242189230);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 1621759557626913976);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 7358437180787121190);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 12786652015051979133);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 4330231899916504777);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 4330231899916504777);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 4330231899916504777);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 534947326614580776);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 338669392700040160);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 13514100800583353838);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 5031679327480404670);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 11173026541242189230);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 7070367758315800582);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 16389198946346934960);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 15655700824360997088);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 11173026541242189230);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 17604070874595404581);     
    }

    #[test]
    fn test_scaled_RGB() {
        let f = |c: &mut WebPDecoderConfig| {
            c.options.use_scaling = 1;
            c.options.scaled_height = 400;
            c.options.scaled_width = 400;
            c.output.colorspace = MODE_RGB;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 8496300406387420057);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 16991195491338246678);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 16991195491338246678);
        test_image_content("./tests/alpha_filter_1.webp", &f, 7450863392266518560);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 16991195491338246678);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 16991195491338246678);
        test_image_content("./tests/alpha_filter_2.webp", &f, 7450863392266518560);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 16991195491338246678);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 16991195491338246678);
        test_image_content("./tests/alpha_filter_3.webp", &f, 7450863392266518560);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 16991195491338246678);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 16991195491338246678);
        test_image_content("./tests/alpha_no_compression.webp", &f, 7450863392266518560);
        test_image_content("./tests/bad_palette_index.webp", &f, 9041392470906353804);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 15186045675887989614);
        test_image_content("./tests/bryce.webp", &f, 6528301304925262543);
        test_image_content("./tests/bug3.webp", &f, 2151385046846183127);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 9178524647859466749);
        test_image_content("./tests/dual_transform.webp", &f, 13514100800583353838);
        test_image_content("./tests/lossless1.webp", &f, 15641083277579694543);
        test_image_content("./tests/lossless2.webp", &f, 15641083277579694543);
        test_image_content("./tests/lossless3.webp", &f, 15641083277579694543);
        test_image_content("./tests/lossless4.webp", &f, 1885628003695082520);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 18408920678391402826);
        test_image_content("./tests/lossless_color_transform.webp", &f, 11861813437063993512);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 13407654101199383474);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 6689542105647697115);
        test_image_content("./tests/lossy_alpha1.webp", &f, 13441121291645719525);
        test_image_content("./tests/lossy_alpha2.webp", &f, 16051722522726096678);
        test_image_content("./tests/lossy_alpha3.webp", &f, 17515436170795814650);
        test_image_content("./tests/lossy_alpha4.webp", &f, 13060676284275505930);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 2576173423984583205);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 11564139629670853718);
        test_image_content("./tests/near_lossless_75.webp", &f, 11397785317340219358);
        test_image_content("./tests/one_color_no_palette.webp", &f, 13514100800583353838);
        test_image_content("./tests/segment01.webp", &f, 4629497837116994406);
        test_image_content("./tests/segment02.webp", &f, 928343434406594713);
        test_image_content("./tests/segment03.webp", &f, 12348464294891338298);
        test_image_content("./tests/small_13x1.webp", &f, 18165445199902219607);
        test_image_content("./tests/small_1x1.webp", &f, 14245113078886307488);
        test_image_content("./tests/small_1x13.webp", &f, 6591505553100393654);
        test_image_content("./tests/small_31x13.webp", &f, 9570271904886120278);
        test_image_content("./tests/test-nostrong.webp", &f, 12051688518663613288);
        test_image_content("./tests/test.webp", &f, 15575589736808082914);
        test_image_content("./tests/very_short.webp", &f, 50683033560429460);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 17629395591677407945);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 7164630703336303165);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 9322053717678622528);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 17629395591677407945);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 14462235502604654784);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 13617676562813977884);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 2522809024637735789);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 17654490049372849144);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 6003128207391027580);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 9602585010235656042);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 17629395591677407945);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 18419132768935117634);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 13401332930359776485);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 1256061105836515264);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 7575730612500835598);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 13135636112833236445);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 13135636112833236445);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 5396564115783505125);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 12272488456655783548);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 1685521638908905655);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 2459023040063715763);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 5396564115783505125);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 12272488456655783548);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 16089379547133978758);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 11206063671529609606);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 5396564115783505125);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 5396564115783505125);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 15069977853942756286);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 15069977853942756286);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 15069977853942756286);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 15069977853942756286);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 12272488456655783548);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 3286070960789300974);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 3286070960789300974);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 15343121453560826097);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 3626204530911903851);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 9325777924288189392);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 2016818996256342495);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 13287014967162752864);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 3716546023096318344);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 9192912368481220869);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 18049091039776079405);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 15535606991524567813);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 5396564115783505125);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 5396564115783505125);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 5396564115783505125);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 14581424042772773933);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 5855802119874043987);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 13514100800583353838);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 10995224717900898703);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 3716546023096318344);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 3113163136047660473);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 10014473018323729999);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 1140171137329752113);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 3716546023096318344);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 6540809962599009246);    
    }

    #[test]
    fn test_scaled_BGRA() {
        let f = |c: &mut WebPDecoderConfig| {
            c.options.use_scaling = 1;
            c.options.scaled_height = 400;
            c.options.scaled_width = 400;
            c.output.colorspace = MODE_BGRA;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 16207130485323236438);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 2910167333517938926);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 2910167333517938926);
        test_image_content("./tests/alpha_filter_1.webp", &f, 16755115091575944091);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 2910167333517938926);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 2910167333517938926);
        test_image_content("./tests/alpha_filter_2.webp", &f, 16755115091575944091);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 2910167333517938926);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 2910167333517938926);
        test_image_content("./tests/alpha_filter_3.webp", &f, 16755115091575944091);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 2910167333517938926);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 2910167333517938926);
        test_image_content("./tests/alpha_no_compression.webp", &f, 16755115091575944091);
        test_image_content("./tests/bad_palette_index.webp", &f, 3038283499532938534);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 8853510385354192078);
        test_image_content("./tests/bryce.webp", &f, 17482405464736447535);
        test_image_content("./tests/bug3.webp", &f, 15811796986340042553);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 11033778913262293597);
        test_image_content("./tests/dual_transform.webp", &f, 16994494846755445295);
        test_image_content("./tests/lossless1.webp", &f, 4719808626127161447);
        test_image_content("./tests/lossless2.webp", &f, 4719808626127161447);
        test_image_content("./tests/lossless3.webp", &f, 4719808626127161447);
        test_image_content("./tests/lossless4.webp", &f, 6126088187163379886);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 9650814363258855976);
        test_image_content("./tests/lossless_color_transform.webp", &f, 1602912566782731581);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 12700367435566296332);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 3133176412819849169);
        test_image_content("./tests/lossy_alpha1.webp", &f, 14510687716324310830);
        test_image_content("./tests/lossy_alpha2.webp", &f, 5271648806130258927);
        test_image_content("./tests/lossy_alpha3.webp", &f, 17165695619224911699);
        test_image_content("./tests/lossy_alpha4.webp", &f, 2781405648138788747);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 12199264140987133485);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 1025729142117234723);
        test_image_content("./tests/near_lossless_75.webp", &f, 3814575133759313496);
        test_image_content("./tests/one_color_no_palette.webp", &f, 8542849973809216197);
        test_image_content("./tests/segment01.webp", &f, 434350625440940891);
        test_image_content("./tests/segment02.webp", &f, 700370629364882009);
        test_image_content("./tests/segment03.webp", &f, 9383097518827553146);
        test_image_content("./tests/small_13x1.webp", &f, 8608793693822926078);
        test_image_content("./tests/small_1x1.webp", &f, 14704668334517596739);
        test_image_content("./tests/small_1x13.webp", &f, 10724264616134815498);
        test_image_content("./tests/small_31x13.webp", &f, 12337852298058477845);
        test_image_content("./tests/test-nostrong.webp", &f, 224243326540609069);
        test_image_content("./tests/test.webp", &f, 162168234243014032);
        test_image_content("./tests/very_short.webp", &f, 13554398002431798839);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 17317325640451644746);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 14046524992675615051);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 14685951850562873196);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 17317325640451644746);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 8455342342686961444);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 4060645382990650403);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 11948276729814088120);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 15452385177840527740);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 6188695059948823853);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 14953258969560614702);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 17317325640451644746);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 16458992392262152501);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 1650386485136986919);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 1551329692049358307);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 7248572160368391869);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 2584638873926421128);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 2584638873926421128);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 4766004468616640295);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 14959458670714192742);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 4978487114114335340);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 7207942316180976033);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 4766004468616640295);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 14959458670714192742);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 6864570833814718758);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 12432773718554224885);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 4766004468616640295);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 4766004468616640295);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 14295597387364183054);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 14295597387364183054);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 14295597387364183054);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 14295597387364183054);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 14959458670714192742);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 9474329215960220216);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 9474329215960220216);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 14012438578050567869);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 3393626352721490399);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 8446096410644755159);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 1969359003350961740);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 7972352757347372434);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 1231837621976237565);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 2050348561503895811);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 10216885971390547982);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 3332781289449189101);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 4766004468616640295);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 4766004468616640295);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 4766004468616640295);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 2820193371561970919);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 5461010475916074907);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 2981137820298253772);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 15847520528507386951);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 1231837621976237565);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 13315506532589091767);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 16534678259878535676);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 8369885892678207448);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 1231837621976237565);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 13920328878282566572);
    }

    #[test]
    fn test_scaled_RGBA() {
        let f = |c: &mut WebPDecoderConfig| {
            c.options.use_scaling = 1;
            c.options.scaled_height = 400;
            c.options.scaled_width = 400;
            c.output.colorspace = MODE_RGBA;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 16207130485323236438);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 2883133658703380769);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 2883133658703380769);
        test_image_content("./tests/alpha_filter_1.webp", &f, 3598351071849248058);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 2883133658703380769);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 2883133658703380769);
        test_image_content("./tests/alpha_filter_2.webp", &f, 3598351071849248058);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 2883133658703380769);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 2883133658703380769);
        test_image_content("./tests/alpha_filter_3.webp", &f, 3598351071849248058);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 2883133658703380769);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 2883133658703380769);
        test_image_content("./tests/alpha_no_compression.webp", &f, 3598351071849248058);
        test_image_content("./tests/bad_palette_index.webp", &f, 3885651580140281472);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 3260208479358402338);
        test_image_content("./tests/bryce.webp", &f, 13715555985977753286);
        test_image_content("./tests/bug3.webp", &f, 7545500009490461645);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 830309552441583318);
        test_image_content("./tests/dual_transform.webp", &f, 16994494846755445295);
        test_image_content("./tests/lossless1.webp", &f, 15278524241100744341);
        test_image_content("./tests/lossless2.webp", &f, 15278524241100744341);
        test_image_content("./tests/lossless3.webp", &f, 15278524241100744341);
        test_image_content("./tests/lossless4.webp", &f, 3027812491908516883);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 17738327588075475219);
        test_image_content("./tests/lossless_color_transform.webp", &f, 3599428971351345009);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 2197431757746303786);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 56186138694767430);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 56186138694767430);
        test_image_content("./tests/lossy_alpha1.webp", &f, 5045506093995397610);
        test_image_content("./tests/lossy_alpha2.webp", &f, 4713565340324041830);
        test_image_content("./tests/lossy_alpha3.webp", &f, 8635007725704267647);
        test_image_content("./tests/lossy_alpha4.webp", &f, 11150092944956093862);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 11028817587383927682);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 2403919032164702789);
        test_image_content("./tests/near_lossless_75.webp", &f, 12123957375389167107);
        test_image_content("./tests/one_color_no_palette.webp", &f, 8542849973809216197);
        test_image_content("./tests/segment01.webp", &f, 3649938495239351695);
        test_image_content("./tests/segment02.webp", &f, 14228938518414269945);
        test_image_content("./tests/segment03.webp", &f, 8390425387917847226);
        test_image_content("./tests/small_13x1.webp", &f, 8608793693822926078);
        test_image_content("./tests/small_1x1.webp", &f, 14704668334517596739);
        test_image_content("./tests/small_1x13.webp", &f, 10724264616134815498);
        test_image_content("./tests/small_31x13.webp", &f, 9304362180797086987);
        test_image_content("./tests/test-nostrong.webp", &f, 8805649296616090750);
        test_image_content("./tests/test.webp", &f, 16884848731838539906);
        test_image_content("./tests/very_short.webp", &f, 4097184264194442158);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 5874029131745362016);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 17275333781692945710);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 2151687434336739110);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 5874029131745362016);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 2082651432423455742);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 5121135049035337643);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 4440656666799324275);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 12465558169810858116);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 6179985537139204889);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 12255263337081066390);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 5874029131745362016);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 16861932282900967974);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 13205900999582549198);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 8358248226505215350);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 8317784809241444758);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 2584638873926421128);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 2584638873926421128);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 4806402976067391197);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 13107145825566866598);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 15802682522231493465);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 13208778820716076555);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 4806402976067391197);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 13107145825566866598);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 8809808835855360777);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 14716339803048382977);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 4806402976067391197);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 4806402976067391197);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 7472205482659515315);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 7472205482659515315);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 7472205482659515315);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 7472205482659515315);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 13107145825566866598);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 5789850912845350238);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 5789850912845350238);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 9273702148914425119);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 14201441336229406489);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 14358441394111205265);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 10678666295478002966);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 11077314812761271050);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 5297049571631886663);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 3926311551498878336);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 2294442799835320760);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 9852414272858284693);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 4806402976067391197);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 4806402976067391197);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 4806402976067391197);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 1322334222644753240);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 13186799830282504549);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 2981137820298253772);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 13778807318297272842);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 5297049571631886663);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 9399205952362275244);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 15058708297683838409);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 12095718916900125021);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 5297049571631886663);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 12589495910848919478);
    }
    
    #[test]
    fn test_YUVA() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_YUVA;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 9638777573570281386);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 11333332125539935109);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 11333332125539935109);
        test_image_content("./tests/alpha_filter_1.webp", &f, 3174543925403196331);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 11333332125539935109);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 11333332125539935109);
        test_image_content("./tests/alpha_filter_2.webp", &f, 3174543925403196331);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 11333332125539935109);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 11333332125539935109);
        test_image_content("./tests/alpha_filter_3.webp", &f, 3174543925403196331);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 11333332125539935109);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 11333332125539935109);
        test_image_content("./tests/alpha_no_compression.webp", &f, 3174543925403196331);
        test_image_content("./tests/bad_palette_index.webp", &f, 13427082893806224307);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 4309370149400797002);
        test_image_content("./tests/bryce.webp", &f, 9847949348626074141);
        test_image_content("./tests/bug3.webp", &f, 14309417205543555619);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 10052776044956046317);
        test_image_content("./tests/dual_transform.webp", &f, 8485107581655717353);
        test_image_content("./tests/lossless1.webp", &f, 15188151260719389722);
        test_image_content("./tests/lossless2.webp", &f, 15188151260719389722);
        test_image_content("./tests/lossless3.webp", &f, 15188151260719389722);
        test_image_content("./tests/lossless4.webp", &f, 17084265723641649755);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 11576497625723777352);
        test_image_content("./tests/lossless_color_transform.webp", &f, 7175567322191193122);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 6732958304003707902);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 5706696886904449882);
        test_image_content("./tests/lossy_alpha1.webp", &f, 1672043130439064317);
        test_image_content("./tests/lossy_alpha2.webp", &f, 3043159771358723850);
        test_image_content("./tests/lossy_alpha3.webp", &f, 2070126136360731365);
        test_image_content("./tests/lossy_alpha4.webp", &f, 13380485557207142667);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 371088537200822737);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 1465284479499372236);
        test_image_content("./tests/near_lossless_75.webp", &f, 3868647272284159295);
        test_image_content("./tests/one_color_no_palette.webp", &f, 2920545549177329814);
        test_image_content("./tests/segment01.webp", &f, 1326467350647170084);
        test_image_content("./tests/segment02.webp", &f, 5864855897325049479);
        test_image_content("./tests/segment03.webp", &f, 8041140937831072944);
        test_image_content("./tests/small_13x1.webp", &f, 4389229955461352629);
        test_image_content("./tests/small_1x1.webp", &f, 12109158149961729428);
        test_image_content("./tests/small_1x13.webp", &f, 1316115007684749786);
        test_image_content("./tests/small_31x13.webp", &f, 2000031433260122718);
        test_image_content("./tests/test-nostrong.webp", &f, 17935893985264830883);
        test_image_content("./tests/test.webp", &f, 7288851890573521834);
        test_image_content("./tests/very_short.webp", &f, 16795446539083835957);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 934672816217584705);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 7766734285299822590);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 7570861190770505899);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 934672816217584705);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 7927363182838089239);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 9514855390532521769);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 2410458218670928087);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 17696634696710678148);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 13120815856909215176);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 4491552612307828249);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 934672816217584705);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 12472224687190320316);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 3965670920120358444);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 108180327906213424);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 1919329803290165193);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 368308820876499999);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 368308820876499999);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 15791666464963144604);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 6430517178402400543);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 11702315357907761114);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 12501185688308929676);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 15791666464963144604);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 6430517178402400543);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 131709264081748398);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 2390799725621344342);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 15791666464963144604);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 15791666464963144604);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 832177081265852125);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 832177081265852125);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 832177081265852125);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 832177081265852125);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 6430517178402400543);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 18211825258976430090);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 18211825258976430090);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 1499636447826099182);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 15982518551782933399);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 1013449650499919507);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 4583898931638787237);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 15735130951734306068);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 9805954865406842674);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 2780432549759526244);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 12445449707719219582);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 11649790867116495866);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 15791666464963144604);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 15791666464963144604);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 15791666464963144604);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 9975638803265746693);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 3206396608318207916);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 16104459786197880622);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 1664709298765127648);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 9805954865406842674);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 13070801209415181186);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 13178521405283407747);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 16520986417641324853);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 9805954865406842674);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 3658015282472524904);
    }

    #[test]
    fn test_YUV() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_YUV;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 3474992779404248608);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 3004970901018743847);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 3004970901018743847);
        test_image_content("./tests/alpha_filter_1.webp", &f, 11349585745597767857);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 3004970901018743847);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 3004970901018743847);
        test_image_content("./tests/alpha_filter_2.webp", &f, 11349585745597767857);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 3004970901018743847);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 3004970901018743847);
        test_image_content("./tests/alpha_filter_3.webp", &f, 11349585745597767857);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 3004970901018743847);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 3004970901018743847);
        test_image_content("./tests/alpha_no_compression.webp", &f, 11349585745597767857);
        test_image_content("./tests/bad_palette_index.webp", &f, 15230030312675313427);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 10949706236812809113);
        test_image_content("./tests/bryce.webp", &f, 6657667969677286388);
        test_image_content("./tests/bug3.webp", &f, 11275693413315454328);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 650048390178428232);
        test_image_content("./tests/dual_transform.webp", &f, 2485895723696896775);
        test_image_content("./tests/lossless1.webp", &f, 994386865154200822);
        test_image_content("./tests/lossless2.webp", &f, 994386865154200822);
        test_image_content("./tests/lossless3.webp", &f, 994386865154200822);
        test_image_content("./tests/lossless4.webp", &f, 5492913336928769661);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 14048671487548564801);
        test_image_content("./tests/lossless_color_transform.webp", &f, 6778877337624237611);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 2470535486098881377);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 8622848306246478795);
        test_image_content("./tests/lossy_alpha1.webp", &f, 5969530734020840437);
        test_image_content("./tests/lossy_alpha2.webp", &f, 16513800535201139000);
        test_image_content("./tests/lossy_alpha3.webp", &f, 4878680559717078237);
        test_image_content("./tests/lossy_alpha4.webp", &f, 14195593520172435981);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 16756433927207707723);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 13736270089073125532);
        test_image_content("./tests/near_lossless_75.webp", &f, 5708247523083753911);
        test_image_content("./tests/one_color_no_palette.webp", &f, 17198357561468744895);
        test_image_content("./tests/segment01.webp", &f, 10941727075555774318);
        test_image_content("./tests/segment02.webp", &f, 4850056790954214670);
        test_image_content("./tests/segment03.webp", &f, 12551694763951648789);
        test_image_content("./tests/small_13x1.webp", &f, 16372896407227169354);
        test_image_content("./tests/small_1x1.webp", &f, 16491929236272943583);
        test_image_content("./tests/small_1x13.webp", &f, 11685494956061918361);
        test_image_content("./tests/small_31x13.webp", &f, 17013548887507839441);
        test_image_content("./tests/test-nostrong.webp", &f, 14377844994611760394);
        test_image_content("./tests/test.webp", &f, 9627110970154455185);
        test_image_content("./tests/very_short.webp", &f, 7632884772137305221);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 9355848730023103091);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 18028151639488670670);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 7229499870379133576);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 9355848730023103091);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 14079666174632908878);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 8563487092246976822);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 16733147611314642739);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 14989493782480296099);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 8148799910139584875);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 15503015717951169804);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 9355848730023103091);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 7916224860763712627);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 12597879995200144039);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 17247624111630286842);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 18064761448021880166);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 12383978450723721656);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 12383978450723721656);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 6364673629675438833);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 17780236931627830792);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 1926311084405852767);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 4323124624442235217);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 6364673629675438833);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 17780236931627830792);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 15080138185797243634);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 7472832333700018235);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 6364673629675438833);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 6364673629675438833);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 1319654606249522397);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 1319654606249522397);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 1319654606249522397);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 1319654606249522397);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 17780236931627830792);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 6980677691016953888);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 6980677691016953888);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 5973323784049872067);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 13736536249469488120);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 8898392653287470218);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 8331716693260815157);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 9176514705615776748);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 18331513258787515521);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 1697909927119881304);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 14418219510526562350);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 17062929972752140231);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 6364673629675438833);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 6364673629675438833);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 6364673629675438833);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 11630636983220499181);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 5222470421015135298);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 12576087962794241756);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 10336362071919323929);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 18331513258787515521);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 9669468750385223684);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 2387427410075924209);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 11156405828281136236);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 18331513258787515521);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 6634557068448662126);
    }

    #[test]
    fn test_rgbA_4444() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_rgbA_4444;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 5183629220140774223);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 10741743997878403199);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 10741743997878403199);
        test_image_content("./tests/alpha_filter_1.webp", &f, 4614066837185535501);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 10741743997878403199);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 10741743997878403199);
        test_image_content("./tests/alpha_filter_2.webp", &f, 4614066837185535501);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 10741743997878403199);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 10741743997878403199);
        test_image_content("./tests/alpha_filter_3.webp", &f, 4614066837185535501);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 10741743997878403199);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 10741743997878403199);
        test_image_content("./tests/alpha_no_compression.webp", &f, 4614066837185535501);
        test_image_content("./tests/bad_palette_index.webp", &f, 16422146949039710696);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 7756685877010735413);
        test_image_content("./tests/bryce.webp", &f, 941163808647146557);
        test_image_content("./tests/bug3.webp", &f, 15025358370905369722);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 6512479056220625338);
        test_image_content("./tests/dual_transform.webp", &f, 14612174882414264071);
        test_image_content("./tests/lossless1.webp", &f, 9941411965333050850);
        test_image_content("./tests/lossless2.webp", &f, 9941411965333050850);
        test_image_content("./tests/lossless3.webp", &f, 9941411965333050850);
        test_image_content("./tests/lossless4.webp", &f, 14954869146320100727);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 12021971126505636700);
        test_image_content("./tests/lossless_color_transform.webp", &f, 16863369754669540162);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 11069039906785897999);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossy_alpha1.webp", &f, 12727411246866937815);
        test_image_content("./tests/lossy_alpha2.webp", &f, 1918986316844233605);
        test_image_content("./tests/lossy_alpha3.webp", &f, 4445771678048654765);
        test_image_content("./tests/lossy_alpha4.webp", &f, 4712728214993714199);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 3716535579688255619);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 9251583211769441111);
        test_image_content("./tests/near_lossless_75.webp", &f, 4331956676935848714);
        test_image_content("./tests/one_color_no_palette.webp", &f, 17286057445253439696);
        test_image_content("./tests/segment01.webp", &f, 5262223921742871160);
        test_image_content("./tests/segment02.webp", &f, 11460143323253972121);
        test_image_content("./tests/segment03.webp", &f, 17620425050852583186);
        test_image_content("./tests/small_13x1.webp", &f, 4223455076205358891);
        test_image_content("./tests/small_1x1.webp", &f, 10800457634906405495);
        test_image_content("./tests/small_1x13.webp", &f, 10808715285177172796);
        test_image_content("./tests/small_31x13.webp", &f, 7370862965663297888);
        test_image_content("./tests/test-nostrong.webp", &f, 10897815655716335707);
        test_image_content("./tests/test.webp", &f, 6512479056220625338);
        test_image_content("./tests/very_short.webp", &f, 18218761263281754067);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 10661370254249552689);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 17495259820761945245);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 7942602930154249885);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 10661370254249552689);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 2613655418788009688);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 8818734285056629714);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 10020478168318362316);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 840238126922686107);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 14099634718419032495);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 5321014092072803667);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 10661370254249552689);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 1338348102747626514);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 10992988098964162642);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 4655210763121632004);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 6619984434132188606);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 11190655288556094035);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 11190655288556094035);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 1743896026389179106);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 7119236778439611215);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 4555538465363689228);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 1743896026389179106);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 1513051972059982238);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 3116772841842444991);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 3211808057276593186);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 3211808057276593186);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 3211808057276593186);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 3211808057276593186);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 1743896026389179106);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 12294020709199191535);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 12294020709199191535);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 4458984480212397307);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 9778970247381524048);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 17725167822497453783);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 2837535194709607644);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 7950259965822728138);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 5464479712718815327);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 5550918783888670469);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 12517540393848050931);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 8907896123691903823);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 3707991574133676904);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 10602442514559167325);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 14811015334848498454);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 14662583028179007871);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 5464479712718815327);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 16100372722192024654);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 5355400306316767617);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 5927738553698592809);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 5464479712718815327);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 7035062736964009622);
    }

    #[test]
    fn test_Argb() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_Argb;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 15780301003160424808);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 16926493588590932684);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 16926493588590932684);
        test_image_content("./tests/alpha_filter_1.webp", &f, 5177571080631143236);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 16926493588590932684);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 16926493588590932684);
        test_image_content("./tests/alpha_filter_2.webp", &f, 5177571080631143236);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 16926493588590932684);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 16926493588590932684);
        test_image_content("./tests/alpha_filter_3.webp", &f, 5177571080631143236);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 16926493588590932684);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 16926493588590932684);
        test_image_content("./tests/alpha_no_compression.webp", &f, 5177571080631143236);
        test_image_content("./tests/bad_palette_index.webp", &f, 16288752542560152182);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 6041510600072784995);
        test_image_content("./tests/bryce.webp", &f, 12539892442066274394);
        test_image_content("./tests/bug3.webp", &f, 8880813788267211959);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 12012142811921276764);
        test_image_content("./tests/dual_transform.webp", &f, 7612336693437835615);
        test_image_content("./tests/lossless1.webp", &f, 2972150479133917846);
        test_image_content("./tests/lossless2.webp", &f, 2972150479133917846);
        test_image_content("./tests/lossless3.webp", &f, 2972150479133917846);
        test_image_content("./tests/lossless4.webp", &f, 15178743373985596720);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 2432871912815203434);
        test_image_content("./tests/lossless_color_transform.webp", &f, 1384264862805162481);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 11264440178594911241);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossy_alpha1.webp", &f, 206025847743438446);
        test_image_content("./tests/lossy_alpha2.webp", &f, 657964442973896632);
        test_image_content("./tests/lossy_alpha3.webp", &f, 9045279193770940127);
        test_image_content("./tests/lossy_alpha4.webp", &f, 7460380494019500591);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 14681848179105699521);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 10959456711559800550);
        test_image_content("./tests/near_lossless_75.webp", &f, 14714347849546029625);
        test_image_content("./tests/one_color_no_palette.webp", &f, 9808690379913418648);
        test_image_content("./tests/segment01.webp", &f, 6545141103219859828);
        test_image_content("./tests/segment02.webp", &f, 10794331457389943444);
        test_image_content("./tests/segment03.webp", &f, 14285204737964054446);
        test_image_content("./tests/small_13x1.webp", &f, 8244309337705734695);
        test_image_content("./tests/small_1x1.webp", &f, 4706134464809827614);
        test_image_content("./tests/small_1x13.webp", &f, 295290870981034749);
        test_image_content("./tests/small_31x13.webp", &f, 15467145033076586171);
        test_image_content("./tests/test-nostrong.webp", &f, 10345678747868220848);
        test_image_content("./tests/test.webp", &f, 12012142811921276764);
        test_image_content("./tests/very_short.webp", &f, 8497448496564305773);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 7453153193676949298);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 17939983551409002301);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 2190256278640396675);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 7453153193676949298);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 5425251098648699552);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 18373029440679426903);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 6834603127295584174);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 798305262579520679);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 9558553570458306217);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 221566535788154906);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 7453153193676949298);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 4024326113416257796);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 8013138137990559578);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 13524719090177118777);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 16469677954130693492);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 18211383531383787585);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 18211383531383787585);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 2578518875178414922);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 179461387360928300);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 4309158072133128565);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 2578518875178414922);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 294923574533231048);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 6642343728593114357);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 3827750572619311110);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 3827750572619311110);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 3827750572619311110);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 3827750572619311110);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 2578518875178414922);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 13381924260957498147);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 13381924260957498147);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 11022902360953346125);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 15611872889305565365);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 13761928867813915609);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 6712632036351477233);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 8879940848676960649);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 13990993469890565548);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 13615040048890207214);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 10006113905080988855);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 5156089755730069608);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 7250615255519740206);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 18212321833671898876);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 2680980981297113073);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 2432898893684800677);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 13990993469890565548);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 12636119779724504410);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 10132431280037259482);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 4682328995158547136);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 13990993469890565548);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 12458476009264963515);
    }

    #[test]
    fn test_bgrA() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_bgrA;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 13200293064044826714);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 8101000576947234136);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 8101000576947234136);
        test_image_content("./tests/alpha_filter_1.webp", &f, 11068395841074162207);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 8101000576947234136);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 8101000576947234136);
        test_image_content("./tests/alpha_filter_2.webp", &f, 11068395841074162207);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 8101000576947234136);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 8101000576947234136);
        test_image_content("./tests/alpha_filter_3.webp", &f, 11068395841074162207);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 8101000576947234136);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 8101000576947234136);
        test_image_content("./tests/alpha_no_compression.webp", &f, 11068395841074162207);
        test_image_content("./tests/bad_palette_index.webp", &f, 17474090697979398761);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 16891024868352614128);
        test_image_content("./tests/bryce.webp", &f, 12157559163664871321);
        test_image_content("./tests/bug3.webp", &f, 746327605922074558);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 5969897715208742588);
        test_image_content("./tests/dual_transform.webp", &f, 13632087872546005697);
        test_image_content("./tests/lossless1.webp", &f, 9634801422988375213);
        test_image_content("./tests/lossless2.webp", &f, 9634801422988375213);
        test_image_content("./tests/lossless3.webp", &f, 9634801422988375213);
        test_image_content("./tests/lossless4.webp", &f, 3184245717672319610);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 11719710113546070556);
        test_image_content("./tests/lossless_color_transform.webp", &f, 9287545355540144468);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 2695690167902915428);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossy_alpha1.webp", &f, 17996760603775042836);
        test_image_content("./tests/lossy_alpha2.webp", &f, 4218437504228333748);
        test_image_content("./tests/lossy_alpha3.webp", &f, 1196670295886577307);
        test_image_content("./tests/lossy_alpha4.webp", &f, 9480057946066772684);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 11966796738042645461);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 16120337481045668719);
        test_image_content("./tests/near_lossless_75.webp", &f, 17028577828525571183);
        test_image_content("./tests/one_color_no_palette.webp", &f, 9808690379913418648);
        test_image_content("./tests/segment01.webp", &f, 3809206331823092311);
        test_image_content("./tests/segment02.webp", &f, 4229855499043518996);
        test_image_content("./tests/segment03.webp", &f, 14503776990724597833);
        test_image_content("./tests/small_13x1.webp", &f, 17328207324203757418);
        test_image_content("./tests/small_1x1.webp", &f, 8867309404721969852);
        test_image_content("./tests/small_1x13.webp", &f, 17043210923847447474);
        test_image_content("./tests/small_31x13.webp", &f, 171723046979963892);
        test_image_content("./tests/test-nostrong.webp", &f, 7254931212579854822);
        test_image_content("./tests/test.webp", &f, 5969897715208742588);
        test_image_content("./tests/very_short.webp", &f, 3228065784688482537);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 73577089004075016);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 3302807744511586743);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 14196182496434734115);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 73577089004075016);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 3750148275765906197);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 10332832701865592009);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 18349196800419597812);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 2567273672736981381);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 4695235689122006592);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 1836576628614403053);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 73577089004075016);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 1766917071403748682);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 17253053934155809309);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 2541356686606198519);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 5067452791211421523);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 261479151180380287);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 261479151180380287);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 10689352217731477039);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 10679667521117935990);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 13060910251210075990);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 10689352217731477039);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 3434979740678079697);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 17664759833990256283);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 5576282771236288269);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 5576282771236288269);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 5576282771236288269);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 5576282771236288269);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 10689352217731477039);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 9709938067080051701);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 9709938067080051701);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 17561454924418519984);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 1418088999374087334);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 15265144683288670855);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 15629439242737812514);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 18320539227875778655);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 6533417698723098739);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 10925298854696721154);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 2761636557751742598);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 14565682541033616653);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 17407247695358764765);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 4747927697313587835);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 2479647676023108526);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 1005204986322704393);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 6533417698723098739);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 9978110941153372908);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 6521565840935250430);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 3895044348361620165);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 6533417698723098739);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 6335134898569545897);
     }

    #[test]
    fn test_rgbA() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_rgbA;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 13200293064044826714);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 7400433701782259183);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 7400433701782259183);
        test_image_content("./tests/alpha_filter_1.webp", &f, 5272007599078877219);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 7400433701782259183);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 7400433701782259183);
        test_image_content("./tests/alpha_filter_2.webp", &f, 5272007599078877219);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 7400433701782259183);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 7400433701782259183);
        test_image_content("./tests/alpha_filter_3.webp", &f, 5272007599078877219);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 7400433701782259183);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 7400433701782259183);
        test_image_content("./tests/alpha_no_compression.webp", &f, 5272007599078877219);
        test_image_content("./tests/bad_palette_index.webp", &f, 1090910947100558729);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 14578338886089178743);
        test_image_content("./tests/bryce.webp", &f, 14052922019623174749);
        test_image_content("./tests/bug3.webp", &f, 638443471448203063);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 9032696829302294085);
        test_image_content("./tests/dual_transform.webp", &f, 13632087872546005697);
        test_image_content("./tests/lossless1.webp", &f, 1938831185427927290);
        test_image_content("./tests/lossless2.webp", &f, 1938831185427927290);
        test_image_content("./tests/lossless3.webp", &f, 1938831185427927290);
        test_image_content("./tests/lossless4.webp", &f, 15193085000903525588);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 1199510126729056243);
        test_image_content("./tests/lossless_color_transform.webp", &f, 2948087899762506821);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 2123176665777311380);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossy_alpha1.webp", &f, 11968362453639523806);
        test_image_content("./tests/lossy_alpha2.webp", &f, 15710271062743272249);
        test_image_content("./tests/lossy_alpha3.webp", &f, 12131957799916882756);
        test_image_content("./tests/lossy_alpha4.webp", &f, 18391415274081126115);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 525060679554037037);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 10222701466820955306);
        test_image_content("./tests/near_lossless_75.webp", &f, 7530257669359192060);
        test_image_content("./tests/one_color_no_palette.webp", &f, 9808690379913418648);
        test_image_content("./tests/segment01.webp", &f, 6570607044180368746);
        test_image_content("./tests/segment02.webp", &f, 6759032819078165028);
        test_image_content("./tests/segment03.webp", &f, 14463606103756624804);
        test_image_content("./tests/small_13x1.webp", &f, 17328207324203757418);
        test_image_content("./tests/small_1x1.webp", &f, 8867309404721969852);
        test_image_content("./tests/small_1x13.webp", &f, 17043210923847447474);
        test_image_content("./tests/small_31x13.webp", &f, 17507625013798833325);
        test_image_content("./tests/test-nostrong.webp", &f, 6824349801690322642);
        test_image_content("./tests/test.webp", &f, 9032696829302294085);
        test_image_content("./tests/very_short.webp", &f, 17231324244250450511);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 10528993088316021196);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 14184125964389871461);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 9062867463768315371);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 10528993088316021196);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 14458400196758366939);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 13723356361239798526);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 12293182613381521284);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 17682575757107385322);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 14267103081724255437);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 7786489589618002135);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 10528993088316021196);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 13174892953055691469);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 7206270313254120467);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 17282421823304499997);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 8529179724445690373);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 261479151180380287);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 261479151180380287);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 12625967260415013799);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 6239535174712919043);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 2890680062591464316);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 12625967260415013799);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 14251556194460417820);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 4276127980433439213);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 9404820771392955144);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 9404820771392955144);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 9404820771392955144);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 9404820771392955144);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 12625967260415013799);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 7242667258313339677);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 7242667258313339677);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 6017331541839844920);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 9515310519083387731);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 15546706749050635193);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 15795341873803524049);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 1415373690842442863);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 6411878774706275337);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 12248537304651518567);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 14012593689057580956);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 5265431174996979704);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 10441263465703597084);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 15885055801945624629);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 2479647676023108526);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 14847741520843056888);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 6411878774706275337);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 1241115772363534854);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 17236961982664990239);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 18278078283735323454);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 6411878774706275337);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 4462861197001292528);
    }

    #[test]
    fn test_RGB_565() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_RGB_565;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 18174327520223508077);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 2272135329799751637);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 2272135329799751637);
        test_image_content("./tests/alpha_filter_1.webp", &f, 5489029663602100963);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 2272135329799751637);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 2272135329799751637);
        test_image_content("./tests/alpha_filter_2.webp", &f, 5489029663602100963);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 2272135329799751637);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 2272135329799751637);
        test_image_content("./tests/alpha_filter_3.webp", &f, 5489029663602100963);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 2272135329799751637);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 2272135329799751637);
        test_image_content("./tests/alpha_no_compression.webp", &f, 5489029663602100963);
        test_image_content("./tests/bad_palette_index.webp", &f, 1937409351678369743);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 13090659167533812496);
        test_image_content("./tests/bryce.webp", &f, 17753825850377539513);
        test_image_content("./tests/bug3.webp", &f, 13184875099626196462);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 1971102455892693469);
        test_image_content("./tests/dual_transform.webp", &f, 13477057879659351437);
        test_image_content("./tests/lossless1.webp", &f, 11623469009234084140);
        test_image_content("./tests/lossless2.webp", &f, 11623469009234084140);
        test_image_content("./tests/lossless3.webp", &f, 11623469009234084140);
        test_image_content("./tests/lossless4.webp", &f, 14815690862098070298);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 2275929896537787430);
        test_image_content("./tests/lossless_color_transform.webp", &f, 12441266148388039262);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 8143230128769282423);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 9782326405203758645);
        test_image_content("./tests/lossy_alpha1.webp", &f, 10473064640190620214);
        test_image_content("./tests/lossy_alpha2.webp", &f, 17540469889279410548);
        test_image_content("./tests/lossy_alpha3.webp", &f, 6125727981235540870);
        test_image_content("./tests/lossy_alpha4.webp", &f, 16575699563782803801);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 15428154527394951998);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 9870907337335338963);
        test_image_content("./tests/near_lossless_75.webp", &f, 1458217628421688207);
        test_image_content("./tests/one_color_no_palette.webp", &f, 17286057445253439696);
        test_image_content("./tests/segment01.webp", &f, 13342127006770553084);
        test_image_content("./tests/segment02.webp", &f, 923525622410052394);
        test_image_content("./tests/segment03.webp", &f, 6753730905724356426);
        test_image_content("./tests/small_13x1.webp", &f, 9511569406110540886);
        test_image_content("./tests/small_1x1.webp", &f, 6726247825460826148);
        test_image_content("./tests/small_1x13.webp", &f, 17430497218178459879);
        test_image_content("./tests/small_31x13.webp", &f, 15230835394371425544);
        test_image_content("./tests/test-nostrong.webp", &f, 7651583927657705777);
        test_image_content("./tests/test.webp", &f, 1971102455892693469);
        test_image_content("./tests/very_short.webp", &f, 7241069801762899622);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 1048908139123308719);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 9608506504593277801);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 13166687566215232245);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 1048908139123308719);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 3379325138862678325);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 11962227867829876230);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 10640222995831825779);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 1533514155383412488);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 84120690119584624);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 13646413596115684843);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 1048908139123308719);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 6462099236549305640);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 8290370005071393528);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 13773250954332155977);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 13048735802046510000);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 7490484312763933838);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 7490484312763933838);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 16911531864027981388);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 1202188535064312127);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 5683505414247795394);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 16684333824147980379);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 16911531864027981388);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 1202188535064312127);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 1620508023792189620);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 15690188950126067589);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 16911531864027981388);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 16911531864027981388);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 16465817479232299204);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 16465817479232299204);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 16465817479232299204);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 16465817479232299204);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 1202188535064312127);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 7336979285418515523);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 7336979285418515523);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 7202879557583860634);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 16969409947419642542);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 15434687120591339833);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 12685664336043586144);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 4694854424491722273);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 74098784155731295);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 10017650668425856722);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 13504970064460316151);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 16620683450906205514);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 16911531864027981388);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 16911531864027981388);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 16911531864027981388);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 12142571354417974895);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 13709357263130066402);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 6552889966457834835);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 2949443828650716983);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 74098784155731295);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 5615064754871810017);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 16094580173934184436);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 4394478770250916468);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 74098784155731295);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 9698645140289916158);
    }


    #[test]
    fn test_RGBA_4444() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_RGBA_4444;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 5183629220140774223);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 15719085942379298435);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 15719085942379298435);
        test_image_content("./tests/alpha_filter_1.webp", &f, 13071420070293219079);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 15719085942379298435);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 15719085942379298435);
        test_image_content("./tests/alpha_filter_2.webp", &f, 13071420070293219079);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 15719085942379298435);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 15719085942379298435);
        test_image_content("./tests/alpha_filter_3.webp", &f, 13071420070293219079);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 15719085942379298435);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 15719085942379298435);
        test_image_content("./tests/alpha_no_compression.webp", &f, 13071420070293219079);
        test_image_content("./tests/bad_palette_index.webp", &f, 16422146949039710696);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 6015898826911799124);
        test_image_content("./tests/bryce.webp", &f, 941163808647146557);
        test_image_content("./tests/bug3.webp", &f, 15025358370905369722);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 6512479056220625338);
        test_image_content("./tests/dual_transform.webp", &f, 14612174882414264071);
        test_image_content("./tests/lossless1.webp", &f, 5198065899530321176);
        test_image_content("./tests/lossless2.webp", &f, 5198065899530321176);
        test_image_content("./tests/lossless3.webp", &f, 5198065899530321176);
        test_image_content("./tests/lossless4.webp", &f, 14265518728186922735);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 11894556319252374828);
        test_image_content("./tests/lossless_color_transform.webp", &f, 16863369754669540162);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 3449419153404308727);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 9240050527435070912);
        test_image_content("./tests/lossy_alpha1.webp", &f, 16880194605952716712);
        test_image_content("./tests/lossy_alpha2.webp", &f, 17923156432719734103);
        test_image_content("./tests/lossy_alpha3.webp", &f, 16125665289174301679);
        test_image_content("./tests/lossy_alpha4.webp", &f, 543669151136777054);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 3716535579688255619);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 9251583211769441111);
        test_image_content("./tests/near_lossless_75.webp", &f, 4331956676935848714);
        test_image_content("./tests/one_color_no_palette.webp", &f, 17286057445253439696);
        test_image_content("./tests/segment01.webp", &f, 5262223921742871160);
        test_image_content("./tests/segment02.webp", &f, 11460143323253972121);
        test_image_content("./tests/segment03.webp", &f, 17620425050852583186);
        test_image_content("./tests/small_13x1.webp", &f, 4223455076205358891);
        test_image_content("./tests/small_1x1.webp", &f, 10800457634906405495);
        test_image_content("./tests/small_1x13.webp", &f, 10808715285177172796);
        test_image_content("./tests/small_31x13.webp", &f, 7370862965663297888);
        test_image_content("./tests/test-nostrong.webp", &f, 10897815655716335707);
        test_image_content("./tests/test.webp", &f, 6512479056220625338);
        test_image_content("./tests/very_short.webp", &f, 18218761263281754067);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 10661370254249552689);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 17495259820761945245);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 7942602930154249885);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 10661370254249552689);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 2613655418788009688);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 8818734285056629714);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 10020478168318362316);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 840238126922686107);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 14099634718419032495);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 5321014092072803667);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 10661370254249552689);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 1338348102747626514);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 10992988098964162642);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 4655210763121632004);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 6619984434132188606);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 11190655288556094035);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 11190655288556094035);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 1743896026389179106);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 7119236778439611215);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 4555538465363689228);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 1743896026389179106);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 1513051972059982238);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 3116772841842444991);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 3211808057276593186);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 3211808057276593186);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 3211808057276593186);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 3211808057276593186);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 1743896026389179106);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 12294020709199191535);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 12294020709199191535);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 4458984480212397307);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 9778970247381524048);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 17725167822497453783);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 2837535194709607644);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 7950259965822728138);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 5464479712718815327);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 5550918783888670469);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 12517540393848050931);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 8907896123691903823);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 13726836372402021869);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 3707991574133676904);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 10602442514559167325);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 14811015334848498454);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 14662583028179007871);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 5464479712718815327);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 16100372722192024654);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 5355400306316767617);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 5927738553698592809);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 5464479712718815327);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 7035062736964009622); 
    }

    #[test]
    fn test_ARGB() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_ARGB;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 17185441696192082247);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 17858559106544086790);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 17858559106544086790);
        test_image_content("./tests/alpha_filter_1.webp", &f, 3107742467524464490);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 17858559106544086790);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 17858559106544086790);
        test_image_content("./tests/alpha_filter_2.webp", &f, 3107742467524464490);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 17858559106544086790);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 17858559106544086790);
        test_image_content("./tests/alpha_filter_3.webp", &f, 3107742467524464490);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 17858559106544086790);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 17858559106544086790);
        test_image_content("./tests/alpha_no_compression.webp", &f, 3107742467524464490);
        test_image_content("./tests/bad_palette_index.webp", &f, 16288752542560152182);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 13730096087514683319);
        test_image_content("./tests/bryce.webp", &f, 12539892442066274394);
        test_image_content("./tests/bug3.webp", &f, 8880813788267211959);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 12012142811921276764);
        test_image_content("./tests/dual_transform.webp", &f, 7612336693437835615);
        test_image_content("./tests/lossless1.webp", &f, 6039476158613029915);
        test_image_content("./tests/lossless2.webp", &f, 6039476158613029915);
        test_image_content("./tests/lossless3.webp", &f, 6039476158613029915);
        test_image_content("./tests/lossless4.webp", &f, 15197734785450584264);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 4383196573920324133);
        test_image_content("./tests/lossless_color_transform.webp", &f, 1384264862805162481);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 15155389428655740303);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 1596413534574411823);
        test_image_content("./tests/lossy_alpha1.webp", &f, 6113979670510441558);
        test_image_content("./tests/lossy_alpha2.webp", &f, 10239134202029734169);
        test_image_content("./tests/lossy_alpha3.webp", &f, 11261988974136012532);
        test_image_content("./tests/lossy_alpha4.webp", &f, 4956402448369753390);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 14681848179105699521);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 10959456711559800550);
        test_image_content("./tests/near_lossless_75.webp", &f, 14714347849546029625);
        test_image_content("./tests/one_color_no_palette.webp", &f, 9808690379913418648);
        test_image_content("./tests/segment01.webp", &f, 6545141103219859828);
        test_image_content("./tests/segment02.webp", &f, 10794331457389943444);
        test_image_content("./tests/segment03.webp", &f, 14285204737964054446);
        test_image_content("./tests/small_13x1.webp", &f, 8244309337705734695);
        test_image_content("./tests/small_1x1.webp", &f, 4706134464809827614);
        test_image_content("./tests/small_1x13.webp", &f, 295290870981034749);
        test_image_content("./tests/small_31x13.webp", &f, 15467145033076586171);
        test_image_content("./tests/test-nostrong.webp", &f, 10345678747868220848);
        test_image_content("./tests/test.webp", &f, 12012142811921276764);
        test_image_content("./tests/very_short.webp", &f, 8497448496564305773);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 7453153193676949298);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 17939983551409002301);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 2190256278640396675);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 7453153193676949298);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 5425251098648699552);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 18373029440679426903);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 6834603127295584174);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 798305262579520679);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 9558553570458306217);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 221566535788154906);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 7453153193676949298);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 4024326113416257796);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 8013138137990559578);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 13524719090177118777);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 16469677954130693492);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 18211383531383787585);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 18211383531383787585);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 2578518875178414922);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 179461387360928300);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 4309158072133128565);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 2578518875178414922);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 294923574533231048);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 6642343728593114357);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 3827750572619311110);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 3827750572619311110);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 3827750572619311110);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 3827750572619311110);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 2578518875178414922);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 13381924260957498147);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 13381924260957498147);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 11022902360953346125);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 15611872889305565365);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 13761928867813915609);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 6712632036351477233);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 8879940848676960649);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 13990993469890565548);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 13615040048890207214);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 10006113905080988855);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 5156089755730069608);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 15914472308416695211);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 7250615255519740206);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 18212321833671898876);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 2680980981297113073);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 2432898893684800677);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 13990993469890565548);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 12636119779724504410);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 10132431280037259482);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 4682328995158547136);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 13990993469890565548);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 12458476009264963515);
    }
    #[test]
    fn test_BGRA() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_BGRA;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 17451511506666510217);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 9909325855492650068);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 9909325855492650068);
        test_image_content("./tests/alpha_filter_1.webp", &f, 17313289494320060451);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 9909325855492650068);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 9909325855492650068);
        test_image_content("./tests/alpha_filter_2.webp", &f, 17313289494320060451);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 9909325855492650068);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 9909325855492650068);
        test_image_content("./tests/alpha_filter_3.webp", &f, 17313289494320060451);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 9909325855492650068);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 9909325855492650068);
        test_image_content("./tests/alpha_no_compression.webp", &f, 17313289494320060451);
        test_image_content("./tests/bad_palette_index.webp", &f, 17474090697979398761);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 16711464169472171136);
        test_image_content("./tests/bryce.webp", &f, 12157559163664871321);
        test_image_content("./tests/bug3.webp", &f, 746327605922074558);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 5969897715208742588);
        test_image_content("./tests/dual_transform.webp", &f, 13632087872546005697);
        test_image_content("./tests/lossless1.webp", &f, 15186962494088223670);
        test_image_content("./tests/lossless2.webp", &f, 15186962494088223670);
        test_image_content("./tests/lossless3.webp", &f, 15186962494088223670);
        test_image_content("./tests/lossless4.webp", &f, 4611971907617523647);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 11201137920226875540);
        test_image_content("./tests/lossless_color_transform.webp", &f, 9287545355540144468);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 721131849640202151);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 14973893814603935651);
        test_image_content("./tests/lossy_alpha1.webp", &f, 10668156064399310868);
        test_image_content("./tests/lossy_alpha2.webp", &f, 13088920017849019051);
        test_image_content("./tests/lossy_alpha3.webp", &f, 6736945231792946760);
        test_image_content("./tests/lossy_alpha4.webp", &f, 3581949980520453064);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 11966796738042645461);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 16120337481045668719);
        test_image_content("./tests/near_lossless_75.webp", &f, 17028577828525571183);
        test_image_content("./tests/one_color_no_palette.webp", &f, 9808690379913418648);
        test_image_content("./tests/segment01.webp", &f, 3809206331823092311);
        test_image_content("./tests/segment02.webp", &f, 4229855499043518996);
        test_image_content("./tests/segment03.webp", &f, 14503776990724597833);
        test_image_content("./tests/small_13x1.webp", &f, 17328207324203757418);
        test_image_content("./tests/small_1x1.webp", &f, 8867309404721969852);
        test_image_content("./tests/small_1x13.webp", &f, 17043210923847447474);
        test_image_content("./tests/small_31x13.webp", &f, 171723046979963892);
        test_image_content("./tests/test-nostrong.webp", &f, 7254931212579854822);
        test_image_content("./tests/test.webp", &f, 5969897715208742588);
        test_image_content("./tests/very_short.webp", &f, 3228065784688482537);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 73577089004075016);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 3302807744511586743);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 14196182496434734115);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 73577089004075016);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 3750148275765906197);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 10332832701865592009);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 18349196800419597812);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 2567273672736981381);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 4695235689122006592);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 1836576628614403053);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 73577089004075016);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 1766917071403748682);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 17253053934155809309);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 2541356686606198519);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 5067452791211421523);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 261479151180380287);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 261479151180380287);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 10689352217731477039);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 10679667521117935990);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 13060910251210075990);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 10689352217731477039);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 3434979740678079697);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 17664759833990256283);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 5576282771236288269);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 5576282771236288269);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 5576282771236288269);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 5576282771236288269);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 10689352217731477039);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 9709938067080051701);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 9709938067080051701);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 17561454924418519984);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 1418088999374087334);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 15265144683288670855);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 15629439242737812514);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 18320539227875778655);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 6533417698723098739);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 10925298854696721154);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 2761636557751742598);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 14565682541033616653);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 13514471420364027095);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 17407247695358764765);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 4747927697313587835);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 2479647676023108526);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 1005204986322704393);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 6533417698723098739);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 9978110941153372908);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 6521565840935250430);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 3895044348361620165);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 6533417698723098739);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 6335134898569545897);
    }

    #[test]
    fn test_BGR() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_BGR;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 2913454361384720963);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 2556052671562549039);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 2556052671562549039);
        test_image_content("./tests/alpha_filter_1.webp", &f, 6888836383560165000);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 2556052671562549039);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 2556052671562549039);
        test_image_content("./tests/alpha_filter_2.webp", &f, 6888836383560165000);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 2556052671562549039);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 2556052671562549039);
        test_image_content("./tests/alpha_filter_3.webp", &f, 6888836383560165000);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 2556052671562549039);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 2556052671562549039);
        test_image_content("./tests/alpha_no_compression.webp", &f, 6888836383560165000);
        test_image_content("./tests/bad_palette_index.webp", &f, 15624580412524028931);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 8460073930105010369);
        test_image_content("./tests/bryce.webp", &f, 3303855022254911330);
        test_image_content("./tests/bug3.webp", &f, 10380609592399199417);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 11447727374643030316);
        test_image_content("./tests/dual_transform.webp", &f, 8728726499132860700);
        test_image_content("./tests/lossless1.webp", &f, 33568961276049380);
        test_image_content("./tests/lossless2.webp", &f, 33568961276049380);
        test_image_content("./tests/lossless3.webp", &f, 33568961276049380);
        test_image_content("./tests/lossless4.webp", &f, 6083535616309487244);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 9792747207775969028);
        test_image_content("./tests/lossless_color_transform.webp", &f, 9478508619382257237);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 10353594107401603882);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 4402868032947367273);
        test_image_content("./tests/lossy_alpha1.webp", &f, 11843272998389160050);
        test_image_content("./tests/lossy_alpha2.webp", &f, 2766381018261414025);
        test_image_content("./tests/lossy_alpha3.webp", &f, 18057897434994266828);
        test_image_content("./tests/lossy_alpha4.webp", &f, 17028933096393705339);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 7327862714510314964);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 17198200105551251779);
        test_image_content("./tests/near_lossless_75.webp", &f, 8627150847734463307);
        test_image_content("./tests/one_color_no_palette.webp", &f, 4912433058658058779);
        test_image_content("./tests/segment01.webp", &f, 4701576222021800794);
        test_image_content("./tests/segment02.webp", &f, 8175131700568797218);
        test_image_content("./tests/segment03.webp", &f, 9591484417001599507);
        test_image_content("./tests/small_13x1.webp", &f, 1117221405956860059);
        test_image_content("./tests/small_1x1.webp", &f, 6008772656196695338);
        test_image_content("./tests/small_1x13.webp", &f, 7707196893485409434);
        test_image_content("./tests/small_31x13.webp", &f, 14704193394167168675);
        test_image_content("./tests/test-nostrong.webp", &f, 1200969991215966344);
        test_image_content("./tests/test.webp", &f, 11447727374643030316);
        test_image_content("./tests/very_short.webp", &f, 16620800974025584725);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 3423325780544913700);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 8045164918461224317);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 4785440866033393766);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 3423325780544913700);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 9047167223612569390);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 2362240741508225629);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 670750917575893247);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 15598284228754509896);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 3993232526437157034);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 5996442891017305807);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 3423325780544913700);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 8293665714343510074);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 18102292657637009997);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 18306054413145031615);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 18378987130488604950);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 5371427656524237486);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 5371427656524237486);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 14808258233203378825);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 8303753196441249265);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 16453100230678236479);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 1788256484668883394);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 14808258233203378825);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 8303753196441249265);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 6998888872383188402);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 1334193779245335783);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 14808258233203378825);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 14808258233203378825);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 4472580217318414543);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 4472580217318414543);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 4472580217318414543);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 4472580217318414543);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 8303753196441249265);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 15548466716407543572);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 15548466716407543572);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 11977170647020880031);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 8551590680565381662);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 6750916152335219214);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 17429349644569703535);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 11181724788970975428);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 3246610472674455880);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 5015195242684774458);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 7411682114441091188);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 4977765646397091642);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 14808258233203378825);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 14808258233203378825);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 14808258233203378825);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 10510146203022263016);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 3064380500651093601);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 7225410011572615251);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 3481346803660415562);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 3246610472674455880);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 17190357993956502122);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 16484021767215367127);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 13440015616365533627);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 3246610472674455880);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 8281524027016950099);
    }
    #[test]
    fn test_RGB() {
        let f = |c: &mut WebPDecoderConfig| {
            c.output.colorspace = MODE_RGB;
        };
        test_image_content("./tests/alpha_color_cache.webp", &f, 2913454361384720963);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 4454084747568298652);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 4454084747568298652);
        test_image_content("./tests/alpha_filter_1.webp", &f, 6011594410131779124);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 4454084747568298652);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 4454084747568298652);
        test_image_content("./tests/alpha_filter_2.webp", &f, 6011594410131779124);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 4454084747568298652);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 4454084747568298652);
        test_image_content("./tests/alpha_filter_3.webp", &f, 6011594410131779124);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 4454084747568298652);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 4454084747568298652);
        test_image_content("./tests/alpha_no_compression.webp", &f, 6011594410131779124);
        test_image_content("./tests/bad_palette_index.webp", &f, 15128515097722033356);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 6075993726734272980);
        test_image_content("./tests/bryce.webp", &f, 12903181931603038202);
        test_image_content("./tests/bug3.webp", &f, 8259460300808871586);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 5289314567817301600);
        test_image_content("./tests/dual_transform.webp", &f, 8728726499132860700);
        test_image_content("./tests/lossless1.webp", &f, 3075343193298899283);
        test_image_content("./tests/lossless2.webp", &f, 3075343193298899283);
        test_image_content("./tests/lossless3.webp", &f, 3075343193298899283);
        test_image_content("./tests/lossless4.webp", &f, 2769903657267389632);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 11516320113192805733);
        test_image_content("./tests/lossless_color_transform.webp", &f, 3920332086995872300);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 3245683971764466374);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 5066496403881663723);
        test_image_content("./tests/lossy_alpha1.webp", &f, 1270182377547489538);
        test_image_content("./tests/lossy_alpha2.webp", &f, 17759620166629816529);
        test_image_content("./tests/lossy_alpha3.webp", &f, 3225603084777322982);
        test_image_content("./tests/lossy_alpha4.webp", &f, 2438180382572876552);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 697983142080015224);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 15682668151705299868);
        test_image_content("./tests/near_lossless_75.webp", &f, 16075587290272764983);
        test_image_content("./tests/one_color_no_palette.webp", &f, 4912433058658058779);
        test_image_content("./tests/segment01.webp", &f, 12683558896480056539);
        test_image_content("./tests/segment02.webp", &f, 12399818113177726395);
        test_image_content("./tests/segment03.webp", &f, 17760798346118577522);
        test_image_content("./tests/small_13x1.webp", &f, 1117221405956860059);
        test_image_content("./tests/small_1x1.webp", &f, 6008772656196695338);
        test_image_content("./tests/small_1x13.webp", &f, 7707196893485409434);
        test_image_content("./tests/small_31x13.webp", &f, 3879389953699467458);
        test_image_content("./tests/test-nostrong.webp", &f, 3536817610214074710);
        test_image_content("./tests/test.webp", &f, 5289314567817301600);
        test_image_content("./tests/very_short.webp", &f, 11907083675660831987);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 3092389042741364129);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 15603648270904633025);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 7958730883957445053);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 3092389042741364129);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 17894476285664067719);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 3519210625534756174);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 2279209368488547975);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 10224158908559855443);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 6174977411703052838);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 1473455003051326106);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 3092389042741364129);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 18438901565895304713);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 5414064657538452689);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 17242970368144705271);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 10052583312236922216);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 5371427656524237486);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 5371427656524237486);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 4487727252798481727);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 6383539927389607653);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 9237912938755795283);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 12256496291322955625);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 4487727252798481727);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 6383539927389607653);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 5956637924192584512);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 11913436149198565877);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 4487727252798481727);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 4487727252798481727);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 16777804472253501131);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 16777804472253501131);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 16777804472253501131);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 16777804472253501131);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 6383539927389607653);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 6273298305613432435);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 6273298305613432435);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 18292239184435347605);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 6190256874707642733);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 17780479850531050592);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 1037081887476100843);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 17709326174316390388);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 11734058065344326174);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 12041091687345002220);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 6494511559960439435);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 8842533448937048864);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 4487727252798481727);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 4487727252798481727);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 4487727252798481727);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 4668686023924272557);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 9503315707312108438);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 7225410011572615251);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 17330105695096962081);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 11734058065344326174);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 16642773364789350355);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 8762512061584654382);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 17361807787146331473);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 11734058065344326174);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 14946801087224720810); 
    }
    
    
    #[test]
    fn test_dithering() {
        let f = |c: &mut WebPDecoderConfig| {
            c.options.dithering_strength = 50;
            c.options.use_scaling = 0;
            c.options.use_cropping = 0;
            c.output.colorspace = MODE_RGBA;
        };

        test_image_content("./tests/alpha_color_cache.webp", &f, 17451511506666510217);
        test_image_content("./tests/alpha_filter_0_method_0.webp", &f, 13612089935755543267);
        test_image_content("./tests/alpha_filter_0_method_1.webp", &f, 13612089935755543267);
        test_image_content("./tests/alpha_filter_1.webp", &f, 14567914834511743475);
        test_image_content("./tests/alpha_filter_1_method_0.webp", &f, 13612089935755543267);
        test_image_content("./tests/alpha_filter_1_method_1.webp", &f, 13612089935755543267);
        test_image_content("./tests/alpha_filter_2.webp", &f, 14567914834511743475);
        test_image_content("./tests/alpha_filter_2_method_0.webp", &f, 13612089935755543267);
        test_image_content("./tests/alpha_filter_2_method_1.webp", &f, 13612089935755543267);
        test_image_content("./tests/alpha_filter_3.webp", &f, 14567914834511743475);
        test_image_content("./tests/alpha_filter_3_method_0.webp", &f, 13612089935755543267);
        test_image_content("./tests/alpha_filter_3_method_1.webp", &f, 13612089935755543267);
        test_image_content("./tests/alpha_no_compression.webp", &f, 14567914834511743475);
        test_image_content("./tests/bad_palette_index.webp", &f, 1090910947100558729);
        test_image_content("./tests/big_endian_bug_393.webp", &f, 2980967713475538130);
        test_image_content("./tests/bryce.webp", &f, 13020037970601992189);
        test_image_content("./tests/bug3.webp", &f, 638443471448203063);
        test_image_content("./tests/color_cache_bits_11.webp", &f, 9032696829302294085);
        test_image_content("./tests/dual_transform.webp", &f, 13632087872546005697);
        test_image_content("./tests/lossless1.webp", &f, 6665888316076904980);
        test_image_content("./tests/lossless2.webp", &f, 6665888316076904980);
        test_image_content("./tests/lossless3.webp", &f, 6665888316076904980);
        test_image_content("./tests/lossless4.webp", &f, 9888661031943394232);
        test_image_content("./tests/lossless_big_random_alpha.webp", &f, 7594911903621859128);
        test_image_content("./tests/lossless_color_transform.webp", &f, 2948087899762506821);
        test_image_content("./tests/lossless_vec_1_0.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_1.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_10.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_11.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_12.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_13.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_14.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_15.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_2.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_3.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_4.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_5.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_6.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_7.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_8.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_1_9.webp", &f, 17733108235855910246);
        test_image_content("./tests/lossless_vec_2_0.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_1.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_10.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_11.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_12.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_13.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_14.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_15.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_2.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_3.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_4.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_5.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_6.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_7.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_8.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossless_vec_2_9.webp", &f, 1003123684174886669);
        test_image_content("./tests/lossy_alpha1.webp", &f, 4663173370595833811);
        test_image_content("./tests/lossy_alpha2.webp", &f, 13040174325108282556);
        test_image_content("./tests/lossy_alpha3.webp", &f, 8910265089565344758);
        test_image_content("./tests/lossy_alpha4.webp", &f, 6682460183289180607);
        test_image_content("./tests/lossy_extreme_probabilities.webp", &f, 8869165650564247613);
        test_image_content("./tests/lossy_q0_f100.webp", &f, 10222701466820955306);
        test_image_content("./tests/near_lossless_75.webp", &f, 7530257669359192060);
        test_image_content("./tests/one_color_no_palette.webp", &f, 9808690379913418648);
        test_image_content("./tests/segment01.webp", &f, 6570607044180368746);
        test_image_content("./tests/segment02.webp", &f, 6759032819078165028);
        test_image_content("./tests/segment03.webp", &f, 14463606103756624804);
        test_image_content("./tests/small_13x1.webp", &f, 17328207324203757418);
        test_image_content("./tests/small_1x1.webp", &f, 8411948258449527475);
        test_image_content("./tests/small_1x13.webp", &f, 17043210923847447474);
        test_image_content("./tests/small_31x13.webp", &f, 17507625013798833325);
        test_image_content("./tests/test-nostrong.webp", &f, 6824349801690322642);
        test_image_content("./tests/test.webp", &f, 13413763509458305880);
        test_image_content("./tests/very_short.webp", &f, 17231324244250450511);
        test_image_content("./tests/vp80-00-comprehensive-001.webp", &f, 415183656836081989);
        test_image_content("./tests/vp80-00-comprehensive-002.webp", &f, 14184125964389871461);
        test_image_content("./tests/vp80-00-comprehensive-003.webp", &f, 9062867463768315371);
        test_image_content("./tests/vp80-00-comprehensive-004.webp", &f, 415183656836081989);
        test_image_content("./tests/vp80-00-comprehensive-005.webp", &f, 14458400196758366939);
        test_image_content("./tests/vp80-00-comprehensive-006.webp", &f, 13723356361239798526);
        test_image_content("./tests/vp80-00-comprehensive-007.webp", &f, 12293182613381521284);
        test_image_content("./tests/vp80-00-comprehensive-008.webp", &f, 11820616399430363033);
        test_image_content("./tests/vp80-00-comprehensive-009.webp", &f, 14267103081724255437);
        test_image_content("./tests/vp80-00-comprehensive-010.webp", &f, 16919696639985199300);
        test_image_content("./tests/vp80-00-comprehensive-011.webp", &f, 415183656836081989);
        test_image_content("./tests/vp80-00-comprehensive-012.webp", &f, 5116002198453954719);
        test_image_content("./tests/vp80-00-comprehensive-013.webp", &f, 17915879971369554602);
        test_image_content("./tests/vp80-00-comprehensive-014.webp", &f, 17282421823304499997);
        test_image_content("./tests/vp80-00-comprehensive-015.webp", &f, 8529179724445690373);
        test_image_content("./tests/vp80-00-comprehensive-016.webp", &f, 261479151180380287);
        test_image_content("./tests/vp80-00-comprehensive-017.webp", &f, 261479151180380287);
        test_image_content("./tests/vp80-01-intra-1400.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-01-intra-1411.webp", &f, 12625967260415013799);
        test_image_content("./tests/vp80-01-intra-1416.webp", &f, 18432705047646556864);
        test_image_content("./tests/vp80-01-intra-1417.webp", &f, 18135104940564449767);
        test_image_content("./tests/vp80-02-inter-1402.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-02-inter-1412.webp", &f, 12625967260415013799);
        test_image_content("./tests/vp80-02-inter-1418.webp", &f, 14251556194460417820);
        test_image_content("./tests/vp80-02-inter-1424.webp", &f, 4276127980433439213);
        test_image_content("./tests/vp80-03-segmentation-1401.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-03-segmentation-1403.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-03-segmentation-1407.webp", &f, 4233602060963758568);
        test_image_content("./tests/vp80-03-segmentation-1408.webp", &f, 4233602060963758568);
        test_image_content("./tests/vp80-03-segmentation-1409.webp", &f, 4233602060963758568);
        test_image_content("./tests/vp80-03-segmentation-1410.webp", &f, 4233602060963758568);
        test_image_content("./tests/vp80-03-segmentation-1413.webp", &f, 12625967260415013799);
        test_image_content("./tests/vp80-03-segmentation-1414.webp", &f, 10196755882640864001);
        test_image_content("./tests/vp80-03-segmentation-1415.webp", &f, 10196755882640864001);
        test_image_content("./tests/vp80-03-segmentation-1425.webp", &f, 6017331541839844920);
        test_image_content("./tests/vp80-03-segmentation-1426.webp", &f, 9515310519083387731);
        test_image_content("./tests/vp80-03-segmentation-1427.webp", &f, 456413958426692615);
        test_image_content("./tests/vp80-03-segmentation-1432.webp", &f, 15795341873803524049);
        test_image_content("./tests/vp80-03-segmentation-1435.webp", &f, 1415373690842442863);
        test_image_content("./tests/vp80-03-segmentation-1436.webp", &f, 6411878774706275337);
        test_image_content("./tests/vp80-03-segmentation-1437.webp", &f, 12248537304651518567);
        test_image_content("./tests/vp80-03-segmentation-1441.webp", &f, 14012593689057580956);
        test_image_content("./tests/vp80-03-segmentation-1442.webp", &f, 5265431174996979704);
        test_image_content("./tests/vp80-04-partitions-1404.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-04-partitions-1405.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-04-partitions-1406.webp", &f, 3636119088663059664);
        test_image_content("./tests/vp80-05-sharpness-1428.webp", &f, 10441263465703597084);
        test_image_content("./tests/vp80-05-sharpness-1429.webp", &f, 15885055801945624629);
        test_image_content("./tests/vp80-05-sharpness-1430.webp", &f, 2479647676023108526);
        test_image_content("./tests/vp80-05-sharpness-1431.webp", &f, 14847741520843056888);
        test_image_content("./tests/vp80-05-sharpness-1433.webp", &f, 6411878774706275337);
        test_image_content("./tests/vp80-05-sharpness-1434.webp", &f, 1241115772363534854);
        test_image_content("./tests/vp80-05-sharpness-1438.webp", &f, 17236961982664990239);
        test_image_content("./tests/vp80-05-sharpness-1439.webp", &f, 18278078283735323454);
        test_image_content("./tests/vp80-05-sharpness-1440.webp", &f, 6411878774706275337);
        test_image_content("./tests/vp80-05-sharpness-1443.webp", &f, 4462861197001292528);
    }
}

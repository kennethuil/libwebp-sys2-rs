// Copyright 2010 Google Inc. All Rights Reserved.
//
// Use of this source code is governed by a BSD-style license
// that can be found in the COPYING file in the root of the source
// tree. An additional intellectual property rights grant can be found
// in the file PATENTS. All contributing project authors may
// be found in the AUTHORS file in the root of the source tree.
// -----------------------------------------------------------------------------
//

// Author: Skal (pascal.massimino@gmail.com)

#ifndef WEBP_DSP_YUV_H_
#define WEBP_DSP_YUV_H_

#include "src/dsp/dsp.h"
#include "src/dec/vp8_dec.h"

//------------------------------------------------------------------------------
// YUV -> RGB conversion

#ifdef __cplusplus
extern "C" {
#endif

enum {
  YUV_FIX = 16,                    // fixed-point precision for RGB->YUV
  YUV_HALF = 1 << (YUV_FIX - 1),

  YUV_FIX2 = 6,                   // fixed-point precision for YUV->RGB
  YUV_MASK2 = (256 << YUV_FIX2) - 1
};

void VP8YuvToRgb(int y, int u, int v,
                                    uint8_t* const rgb);
void VP8YuvToBgr(int y, int u, int v,
                                    uint8_t* const bgr);

void VP8YuvToRgb565(int y, int u, int v,
                                       uint8_t* const rgb);

void VP8YuvToRgba4444(int y, int u, int v,
                                         uint8_t* const argb);
//-----------------------------------------------------------------------------
// Alpha handling variants

void VP8YuvToArgb(uint8_t y, uint8_t u, uint8_t v,
                                     uint8_t* const argb);

void VP8YuvToBgra(uint8_t y, uint8_t u, uint8_t v,
                                     uint8_t* const bgra);

void VP8YuvToRgba(uint8_t y, uint8_t u, uint8_t v,
                                    uint8_t* const rgba);

//-----------------------------------------------------------------------------
// SSE2 extra functions (mostly for upsampling_sse2.c)

#if defined(WEBP_USE_SSE2)

// Process 32 pixels and store the result (16b, 24b or 32b per pixel) in *dst.
void VP8YuvToRgba32_SSE2(const uint8_t* y, const uint8_t* u, const uint8_t* v,
                         uint8_t* dst);
void VP8YuvToRgb32_SSE2(const uint8_t* y, const uint8_t* u, const uint8_t* v,
                        uint8_t* dst);
void VP8YuvToBgra32_SSE2(const uint8_t* y, const uint8_t* u, const uint8_t* v,
                         uint8_t* dst);
void VP8YuvToBgr32_SSE2(const uint8_t* y, const uint8_t* u, const uint8_t* v,
                        uint8_t* dst);
void VP8YuvToArgb32_SSE2(const uint8_t* y, const uint8_t* u, const uint8_t* v,
                         uint8_t* dst);
void VP8YuvToRgba444432_SSE2(const uint8_t* y, const uint8_t* u,
                             const uint8_t* v, uint8_t* dst);
void VP8YuvToRgb56532_SSE2(const uint8_t* y, const uint8_t* u, const uint8_t* v,
                           uint8_t* dst);

#endif    // WEBP_USE_SSE2

//-----------------------------------------------------------------------------
// SSE41 extra functions (mostly for upsampling_sse41.c)

#if defined(WEBP_USE_SSE41)

// Process 32 pixels and store the result (16b, 24b or 32b per pixel) in *dst.
void VP8YuvToRgb32_SSE41(const uint8_t* y, const uint8_t* u, const uint8_t* v,
                         uint8_t* dst);
void VP8YuvToBgr32_SSE41(const uint8_t* y, const uint8_t* u, const uint8_t* v,
                         uint8_t* dst);

#endif    // WEBP_USE_SSE41

//------------------------------------------------------------------------------
// RGB -> YUV conversion

// Stub functions that can be called with various rounding values:
static WEBP_INLINE int VP8ClipUV(int uv, int rounding) {
  uv = (uv + rounding + (128 << (YUV_FIX + 2))) >> (YUV_FIX + 2);
  return ((uv & ~0xff) == 0) ? uv : (uv < 0) ? 0 : 255;
}

static WEBP_INLINE int VP8RGBToY(int r, int g, int b, int rounding) {
  const int luma = 16839 * r + 33059 * g + 6420 * b;
  return (luma + rounding + (16 << YUV_FIX)) >> YUV_FIX;  // no need to clip
}

static WEBP_INLINE int VP8RGBToU(int r, int g, int b, int rounding) {
  const int u = -9719 * r - 19081 * g + 28800 * b;
  return VP8ClipUV(u, rounding);
}

static WEBP_INLINE int VP8RGBToV(int r, int g, int b, int rounding) {
  const int v = +28800 * r - 24116 * g - 4684 * b;
  return VP8ClipUV(v, rounding);
}

#ifdef __cplusplus
}    // extern "C"
#endif

#endif  // WEBP_DSP_YUV_H_

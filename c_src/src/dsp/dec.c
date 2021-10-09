// Copyright 2010 Google Inc. All Rights Reserved.
//
// Use of this source code is governed by a BSD-style license
// that can be found in the COPYING file in the root of the source
// tree. An additional intellectual property rights grant can be found
// in the file PATENTS. All contributing project authors may
// be found in the AUTHORS file in the root of the source tree.
// -----------------------------------------------------------------------------
//
// Speed-critical decoding functions, default plain-C implementations.
//
// Author: Skal (pascal.massimino@gmail.com)

#include <assert.h>

#include "src/dsp/dsp.h"
#include "src/dec/vp8i_dec.h"
#include "src/utils/utils.h"

//------------------------------------------------------------------------------



//------------------------------------------------------------------------------
// Transforms (Paragraph 14.4)

#if !WEBP_NEON_OMIT_C_CODE
void TransformAC3_C(const int16_t* in, uint8_t* dst);

void TransformTwo_C(const int16_t* in, uint8_t* dst, int do_two);

#endif  // !WEBP_NEON_OMIT_C_CODE

void TransformUV_C(const int16_t* in, uint8_t* dst);

#if !WEBP_NEON_OMIT_C_CODE
void TransformDC_C(const int16_t* in, uint8_t* dst);

#endif  // !WEBP_NEON_OMIT_C_CODE

void TransformDCUV_C(const int16_t* in, uint8_t* dst);

//------------------------------------------------------------------------------
// Paragraph 14.3

#if !WEBP_NEON_OMIT_C_CODE
void TransformWHT_C(const int16_t* in, int16_t* out);

#endif  // !WEBP_NEON_OMIT_C_CODE

void (*VP8TransformWHT)(const int16_t* in, int16_t* out);

//------------------------------------------------------------------------------
// Intra predictions

#if !WEBP_NEON_OMIT_C_CODE

void TM4_C(uint8_t* dst);
void TM8uv_C(uint8_t* dst);
void TM16_C(uint8_t* dst);

//------------------------------------------------------------------------------
// 16x16

void VE16_C(uint8_t* dst);

void HE16_C(uint8_t* dst);

void DC16_C(uint8_t* dst);

void DC16NoTop_C(uint8_t* dst);

void DC16NoLeft_C(uint8_t* dst);

void DC16NoTopLeft_C(uint8_t* dst);

#endif  // !WEBP_NEON_OMIT_C_CODE

VP8PredFunc VP8PredLuma16[NUM_B_DC_MODES];

//------------------------------------------------------------------------------
// 4x4

#if !WEBP_NEON_OMIT_C_CODE
void VE4_C(uint8_t* dst);

#endif  // !WEBP_NEON_OMIT_C_CODE

void HE4_C(uint8_t* dst);

#if !WEBP_NEON_OMIT_C_CODE
void DC4_C(uint8_t* dst);

void RD4_C(uint8_t* dst);

void LD4_C(uint8_t* dst);
#endif  // !WEBP_NEON_OMIT_C_CODE

void VR4_C(uint8_t* dst);

void VL4_C(uint8_t* dst);

void HU4_C(uint8_t* dst);

void HD4_C(uint8_t* dst);

VP8PredFunc VP8PredLuma4[NUM_BMODES];

//------------------------------------------------------------------------------
// Chroma

#if !WEBP_NEON_OMIT_C_CODE
void VE8uv_C(uint8_t* dst);

void HE8uv_C(uint8_t* dst);

void DC8uv_C(uint8_t* dst);

void DC8uvNoLeft_C(uint8_t* dst);

void DC8uvNoTop_C(uint8_t* dst);

void DC8uvNoTopLeft_C(uint8_t* dst);
#endif  // !WEBP_NEON_OMIT_C_CODE

VP8PredFunc VP8PredChroma8[NUM_B_DC_MODES];

//------------------------------------------------------------------------------
// Edge filtering functions

#if !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC


#endif  // !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC

#if !WEBP_NEON_OMIT_C_CODE

#endif  // !WEBP_NEON_OMIT_C_CODE

#if !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC

#endif  // !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC

//------------------------------------------------------------------------------
// Simple In-loop filtering (Paragraph 15.2)

#if !WEBP_NEON_OMIT_C_CODE
void SimpleVFilter16_C(uint8_t* p, int stride, int thresh);

void SimpleHFilter16_C(uint8_t* p, int stride, int thresh);

void SimpleVFilter16i_C(uint8_t* p, int stride, int thresh);

void SimpleHFilter16i_C(uint8_t* p, int stride, int thresh);
#endif  // !WEBP_NEON_OMIT_C_CODE

//------------------------------------------------------------------------------
// Complex In-loop filtering (Paragraph 15.3)

#if !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC

#endif  // !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC

#if !WEBP_NEON_OMIT_C_CODE
// on macroblock edges
void VFilter16_C(uint8_t* p, int stride,
                        int thresh, int ithresh, int hev_thresh);

void HFilter16_C(uint8_t* p, int stride,
                        int thresh, int ithresh, int hev_thresh);

// on three inner edges
void VFilter16i_C(uint8_t* p, int stride,
                         int thresh, int ithresh, int hev_thresh);

#endif  // !WEBP_NEON_OMIT_C_CODE

#if !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC
void HFilter16i_C(uint8_t* p, int stride,
                         int thresh, int ithresh, int hev_thresh);

#endif  // !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC

#if !WEBP_NEON_OMIT_C_CODE
// 8-pixels wide variant, for chroma filtering
void VFilter8_C(uint8_t* u, uint8_t* v, int stride,
                       int thresh, int ithresh, int hev_thresh);
#endif  // !WEBP_NEON_OMIT_C_CODE

#if !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC
void HFilter8_C(uint8_t* u, uint8_t* v, int stride,
                       int thresh, int ithresh, int hev_thresh);

#endif  // !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC

#if !WEBP_NEON_OMIT_C_CODE
void VFilter8i_C(uint8_t* u, uint8_t* v, int stride,
                        int thresh, int ithresh, int hev_thresh);

#endif  // !WEBP_NEON_OMIT_C_CODE

#if !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC
void HFilter8i_C(uint8_t* u, uint8_t* v, int stride,
                        int thresh, int ithresh, int hev_thresh);

#endif  // !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC

//------------------------------------------------------------------------------

void DitherCombine8x8_C(const uint8_t* dither, uint8_t* dst,
                               int dst_stride);

//------------------------------------------------------------------------------

VP8DecIdct2 VP8Transform;
VP8DecIdct VP8TransformAC3;
VP8DecIdct VP8TransformUV;
VP8DecIdct VP8TransformDC;
VP8DecIdct VP8TransformDCUV;

VP8LumaFilterFunc VP8VFilter16;
VP8LumaFilterFunc VP8HFilter16;
VP8ChromaFilterFunc VP8VFilter8;
VP8ChromaFilterFunc VP8HFilter8;
VP8LumaFilterFunc VP8VFilter16i;
VP8LumaFilterFunc VP8HFilter16i;
VP8ChromaFilterFunc VP8VFilter8i;
VP8ChromaFilterFunc VP8HFilter8i;
VP8SimpleFilterFunc VP8SimpleVFilter16;
VP8SimpleFilterFunc VP8SimpleHFilter16;
VP8SimpleFilterFunc VP8SimpleVFilter16i;
VP8SimpleFilterFunc VP8SimpleHFilter16i;

void (*VP8DitherCombine8x8)(const uint8_t* dither, uint8_t* dst,
                            int dst_stride);

WEBP_DSP_INIT_FUNC(VP8DspInit) {
  VP8InitClipTables();

#if !WEBP_NEON_OMIT_C_CODE
  VP8TransformWHT = TransformWHT_C;
  VP8Transform = TransformTwo_C;
  VP8TransformDC = TransformDC_C;
  VP8TransformAC3 = TransformAC3_C;
#endif
  VP8TransformUV = TransformUV_C;
  VP8TransformDCUV = TransformDCUV_C;

#if !WEBP_NEON_OMIT_C_CODE
  VP8VFilter16 = VFilter16_C;
  VP8VFilter16i = VFilter16i_C;
  VP8HFilter16 = HFilter16_C;
  VP8VFilter8 = VFilter8_C;
  VP8VFilter8i = VFilter8i_C;
  VP8SimpleVFilter16 = SimpleVFilter16_C;
  VP8SimpleHFilter16 = SimpleHFilter16_C;
  VP8SimpleVFilter16i = SimpleVFilter16i_C;
  VP8SimpleHFilter16i = SimpleHFilter16i_C;
#endif

#if !WEBP_NEON_OMIT_C_CODE || WEBP_NEON_WORK_AROUND_GCC
  VP8HFilter16i = HFilter16i_C;
  VP8HFilter8 = HFilter8_C;
  VP8HFilter8i = HFilter8i_C;
#endif

#if !WEBP_NEON_OMIT_C_CODE
  VP8PredLuma4[0] = DC4_C;
  VP8PredLuma4[1] = TM4_C;
  VP8PredLuma4[2] = VE4_C;
  VP8PredLuma4[4] = RD4_C;
  VP8PredLuma4[6] = LD4_C;
#endif

  VP8PredLuma4[3] = HE4_C;
  VP8PredLuma4[5] = VR4_C;
  VP8PredLuma4[7] = VL4_C;
  VP8PredLuma4[8] = HD4_C;
  VP8PredLuma4[9] = HU4_C;

#if !WEBP_NEON_OMIT_C_CODE
  VP8PredLuma16[0] = DC16_C;
  VP8PredLuma16[1] = TM16_C;
  VP8PredLuma16[2] = VE16_C;
  VP8PredLuma16[3] = HE16_C;
  VP8PredLuma16[4] = DC16NoTop_C;
  VP8PredLuma16[5] = DC16NoLeft_C;
  VP8PredLuma16[6] = DC16NoTopLeft_C;

  VP8PredChroma8[0] = DC8uv_C;
  VP8PredChroma8[1] = TM8uv_C;
  VP8PredChroma8[2] = VE8uv_C;
  VP8PredChroma8[3] = HE8uv_C;
  VP8PredChroma8[4] = DC8uvNoTop_C;
  VP8PredChroma8[5] = DC8uvNoLeft_C;
  VP8PredChroma8[6] = DC8uvNoTopLeft_C;
#endif

  VP8DitherCombine8x8 = DitherCombine8x8_C;

  assert(VP8TransformWHT != NULL);
  assert(VP8Transform != NULL);
  assert(VP8TransformDC != NULL);
  assert(VP8TransformAC3 != NULL);
  assert(VP8TransformUV != NULL);
  assert(VP8TransformDCUV != NULL);
  assert(VP8VFilter16 != NULL);
  assert(VP8HFilter16 != NULL);
  assert(VP8VFilter8 != NULL);
  assert(VP8HFilter8 != NULL);
  assert(VP8VFilter16i != NULL);
  assert(VP8HFilter16i != NULL);
  assert(VP8VFilter8i != NULL);
  assert(VP8HFilter8i != NULL);
  assert(VP8SimpleVFilter16 != NULL);
  assert(VP8SimpleHFilter16 != NULL);
  assert(VP8SimpleVFilter16i != NULL);
  assert(VP8SimpleHFilter16i != NULL);
  assert(VP8PredLuma4[0] != NULL);
  assert(VP8PredLuma4[1] != NULL);
  assert(VP8PredLuma4[2] != NULL);
  assert(VP8PredLuma4[3] != NULL);
  assert(VP8PredLuma4[4] != NULL);
  assert(VP8PredLuma4[5] != NULL);
  assert(VP8PredLuma4[6] != NULL);
  assert(VP8PredLuma4[7] != NULL);
  assert(VP8PredLuma4[8] != NULL);
  assert(VP8PredLuma4[9] != NULL);
  assert(VP8PredLuma16[0] != NULL);
  assert(VP8PredLuma16[1] != NULL);
  assert(VP8PredLuma16[2] != NULL);
  assert(VP8PredLuma16[3] != NULL);
  assert(VP8PredLuma16[4] != NULL);
  assert(VP8PredLuma16[5] != NULL);
  assert(VP8PredLuma16[6] != NULL);
  assert(VP8PredChroma8[0] != NULL);
  assert(VP8PredChroma8[1] != NULL);
  assert(VP8PredChroma8[2] != NULL);
  assert(VP8PredChroma8[3] != NULL);
  assert(VP8PredChroma8[4] != NULL);
  assert(VP8PredChroma8[5] != NULL);
  assert(VP8PredChroma8[6] != NULL);
  assert(VP8DitherCombine8x8 != NULL);
}

// Copyright 2011 Google Inc. All Rights Reserved.
//
// Use of this source code is governed by a BSD-style license
// that can be found in the COPYING file in the root of the source
// tree. An additional intellectual property rights grant can be found
// in the file PATENTS. All contributing project authors may
// be found in the AUTHORS file in the root of the source tree.
// -----------------------------------------------------------------------------
//
// YUV to RGB upsampling functions.
//
// Author: somnath@google.com (Somnath Banerjee)

#include "src/dsp/dsp.h"
#include "src/dsp/yuv.h"

#include <assert.h>

// For some reason this is used on the encoding path, and I'm not even remotely
// geared up to test that, so I'll leave it here for now.
WebPUpsampleLinePairFunc WebPGetLinePairConverter(int alpha_is_last) {
  WebPInitUpsamplers();
#ifdef FANCY_UPSAMPLING
  return WebPUpsamplers[alpha_is_last ? MODE_BGRA : MODE_ARGB];
#else
  return (alpha_is_last ? DualLineSamplerBGRA : DualLineSamplerARGB);
#endif
}



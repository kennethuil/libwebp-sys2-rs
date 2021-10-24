#![cfg_attr(feature = "__doc_cfg", feature(doc_cfg))]
#![cfg_attr(feature = "extern-types", feature(extern_types))]

#[macro_use]
extern crate cfg_if;

pub use crate::decode::*;
#[cfg(feature = "demux")]
pub use crate::demux::*;
pub use crate::encode::*;
#[cfg(feature = "mux")]
pub use crate::mux::*;
#[cfg(any(feature = "mux", feature = "demux"))]
pub use crate::mux_types::*;
pub use crate::types::*;

mod decode;
#[cfg(feature = "demux")]
mod demux;
mod encode;
#[cfg(feature = "mux")]
mod mux;
#[cfg(any(feature = "mux", feature = "demux"))]
mod mux_types;
mod types;
mod dec;
mod offsetref;
mod dec_clip_tables;
mod frame_dec;
mod array;
mod dsp;
mod vp8_dec;
mod bit_reader_utils;
mod common_dec;
mod random_utils;
mod alpha_dec;
use std::mem::size_of;
use libc::c_int;

// The Boolean decoder needs to maintain infinite precision on the value_ field.
// However, since range_ is only 8bit, we only need an active window of 8 bits
// for value_. Left bits (MSB) gets zeroed and shifted away when value_ falls
// below 128, range_ is updated, and fresh bits read from the bitstream are
// brought in as LSB. To avoid reading the fresh bits one by one (slow), we
// cache BITS of them ahead. The total of (BITS + 8) bits must fit into a
// natural register (with type bit_t). To fetch BITS bits from bitstream we
// use a type lbit_t.
//
// BITS can be any multiple of 8 from 8 to 56 (inclusive).
// Pick values that fit natural register size.
const BITS: usize = size_of::<usize>() - 8;
// C introduces bit_t type which is basically a usize, we're just using usize

#[repr(C)]
#[derive(Debug)]
pub(crate) struct VP8BitReader {
    value: usize,       // current value
    range: u32,         // current range minus 1. In [127, 254] interval.
    bits: c_int,        // number of valid bits left
    // read buffer
    buf: *const u8,     // next byte to be read
    buf_end: *const u8, // end of read buffer
    buf_max: *const u8, // max packed-read position on buffer
    eof: c_int,         // true if input is exhausted
}
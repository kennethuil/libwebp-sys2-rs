use std::{ops::{Index, IndexMut, Range, RangeFrom}, slice::{self, ChunksExactMut, ChunksMut}};

use bytemuck::TransparentWrapper;

pub struct OffsetSliceRef<'a, T> {
    slice: &'a [T],
    zero: isize
}

impl<'a, T> OffsetSliceRef<'a, T> {
    pub const fn new(slice: &'a [T], zero: isize) -> Self {
        OffsetSliceRef {
            slice,
            zero,
        }
    }

    pub const fn with_offset(&self, offset: isize) -> Self {
        OffsetSliceRef { slice: self.slice, zero: self.zero + offset }
    }
}

impl<'a, T> Index<isize> for OffsetSliceRef<'a, T> {
    type Output = T;
    
    fn index(&self, idx: isize) -> &'a Self::Output {
        let inner_idx = idx + self.zero;
        &self.slice[inner_idx as usize]
    }
}

pub struct OffsetSliceRefMut<'a, T> {
    slice: &'a mut [T],
    zero: isize
}

impl<T> OffsetSliceRefMut<'_, T> {
    #[allow(dead_code)]
    pub fn new(slice: &mut [T], zero: isize) -> OffsetSliceRefMut<T> {
        OffsetSliceRefMut {
            slice,
            zero
        }
    }

    #[allow(dead_code)]
    pub fn with_offset<'b, 'c>(&'c mut self, offset: isize) -> OffsetSliceRefMut<'b, T> where 'c: 'b {
        OffsetSliceRefMut{slice: &mut self.slice, zero: self.zero + offset}
    }

    // A regular mutable slice can be resliced in a loop because something something reborrow
    // But that doesn't apply to a struct, even though we're trying to make it act kinda like
    // a mutable reference.
    // So if we try to return an updated struct here, it mutable borrows the original for as long
    // as the return value is live, and we can't do ref = ref.with_offset() in a loop because replacing
    // the original while the replacement is live is forbidden.
    // So to support reslicing in a loop, we must support updating in-place
    #[allow(dead_code)]
    pub fn move_zero(&mut self, offset: isize) {
        self.zero += offset;
    }

    // Safety: max_index must be greater than min_index, ptr must point to the zero index of a 
    // span of memory that is valid & uniquely referenced from min_index to max_index
    pub unsafe fn from_zero_mut_ptr<'a>(ptr: *mut T, min_index: isize, one_plus_max_index: isize)
        -> OffsetSliceRefMut<'a, T> {
        let size = one_plus_max_index - min_index;
        let p_slice = ptr.offset(min_index);
        OffsetSliceRefMut {
            slice: slice::from_raw_parts_mut(p_slice, size as usize),
            zero: -min_index,
        }
    }

}

impl<T> Index<isize> for OffsetSliceRefMut<'_, T> {
    type Output = T;

    fn index(&self, idx: isize) -> &Self::Output {
        let inner_idx = idx + self.zero;
        &self.slice[inner_idx as usize]
    }
}
impl<T> IndexMut<isize> for OffsetSliceRefMut<'_, T> {
    fn index_mut(&mut self, idx: isize) -> &mut Self::Output {
        let inner_idx = idx + self.zero;
        &mut self.slice[inner_idx as usize]       
    }
}

#[derive(TransparentWrapper)]
#[repr(transparent)]
pub struct OffsetArray<T, const SIZE: usize, const ZERO: isize> {
    arr: [T; SIZE]
}

impl<T, const SIZE: usize, const ZERO: isize> OffsetArray<T, SIZE, ZERO> {
    pub fn with_offset<'b, 'c>(&'c mut self, offset: isize) -> OffsetSliceRefMut<'b, T> where 'c: 'b {
        OffsetSliceRefMut{slice: &mut self.arr[..], zero: ZERO + offset}
    }

    // Creates an OffsetArray reference where ZERO represents the location pointed to by ptr
    pub unsafe fn from_zero_mut_ptr<'a>(ptr: *mut T) -> &'a mut OffsetArray<T, SIZE, ZERO> {
        let begin = ptr.offset(-ZERO);
        let arr = &mut *(begin as *mut[T; SIZE]);
        OffsetArray::<T, SIZE, ZERO>::wrap_mut(arr)
    }

    // Passthrough to arr.chunks_exact_mut.  Yields regular slice refs starting at
    // -ZERO (self.arr[0]).
    pub fn chunks_exact_mut(&mut self, chunk_size: usize) -> ChunksExactMut<'_, T> {
        self.arr.chunks_exact_mut(chunk_size)
    }

        // Passthrough to arr.chunks_mut.  Yields regular slice refs starting at
    // -ZERO (self.arr[0]).
    pub fn chunks_mut(&mut self, chunk_size: usize) -> ChunksMut<'_, T> {
        self.arr.chunks_mut(chunk_size)
    }

    // Passthrough to arr.split_at_mut.  Yields regular slice refs.
    pub fn split_at_mut<'b, 'c>(&'c mut self, mid: isize) -> (&'b mut [T], &'b mut [T]) where 'c: 'b {
        let computed_mid = ZERO + mid;
        self.arr.split_at_mut(computed_mid as usize)
    }
}

impl<T, const SIZE: usize, const ZERO: isize> Index<isize> for OffsetArray<T, SIZE, ZERO> {
    type Output = T;

    fn index(&self, index: isize) -> &Self::Output {
        let inner_idx = index + ZERO;
        &self.arr[inner_idx as usize]
    }
}

impl<T, const SIZE: usize, const ZERO: isize> Index<Range<isize>> for OffsetArray<T, SIZE, ZERO> {
    type Output = [T];

    fn index(&self, r: Range<isize>) -> &Self::Output {
        let inner_range = ((r.start + ZERO) as usize)..((r.end + ZERO) as usize);
        &self.arr[inner_range]
    }
}

impl<T, const SIZE: usize, const ZERO: isize> Index<RangeFrom<isize>> for OffsetArray<T, SIZE, ZERO> {
    type Output = [T];

    fn index(&self, r: RangeFrom<isize>) -> &Self::Output {
        let inner_range = ((r.start + ZERO) as usize)..;
        &self.arr[inner_range]
    }
}

impl<T, const SIZE: usize, const ZERO: isize> IndexMut<isize> for OffsetArray<T, SIZE, ZERO> {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let inner_idx = index + ZERO;
        &mut self.arr[inner_idx as usize]
    }
}

impl<T, const SIZE: usize, const ZERO: isize> IndexMut<Range<isize>> for OffsetArray<T, SIZE, ZERO> {
    fn index_mut(&mut self, r: Range<isize>) -> &mut Self::Output {
        let inner_range = ((r.start + ZERO) as usize)..((r.end + ZERO) as usize);
        &mut self.arr[inner_range]
    }
}

impl<T, const SIZE: usize, const ZERO: isize> IndexMut<RangeFrom<isize>> for OffsetArray<T, SIZE, ZERO> {
    fn index_mut(&mut self, r: RangeFrom<isize>) -> &mut Self::Output {
        let inner_range = ((r.start + ZERO) as usize)..;
        &mut self.arr[inner_range]
    }
}

use std::ops::{Index, IndexMut};

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
    pub fn new(slice: &mut [T], zero: isize) -> OffsetSliceRefMut<T> {
        OffsetSliceRefMut {
            slice,
            zero
        }
    }

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
    pub fn move_zero(&mut self, offset: isize) {
        self.zero += offset;
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
}

impl<T, const SIZE: usize, const ZERO: isize> Index<isize> for OffsetArray<T, SIZE, ZERO> {
    type Output = T;

    fn index(&self, index: isize) -> &Self::Output {
        let inner_idx = index + ZERO;
        &self.arr[inner_idx as usize]
    }
}

impl<T, const SIZE: usize, const ZERO: isize> IndexMut<isize> for OffsetArray<T, SIZE, ZERO> {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let inner_idx = index + ZERO;
        &mut self.arr[inner_idx as usize]
    }
}
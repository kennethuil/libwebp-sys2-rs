use std::convert::TryInto;


/// Get a mut reference to the first N items of the slice as an array.
/// Panic if the slice is too small.
pub(crate) fn to_array_ref_mut<T, const N: usize>(s: &mut [T]) -> &mut [T;N] {
    // Parentheses around slice expression needed because otherwise try_into will
    // make a temporary array (not array ref) and then &mut will make a reference to the temporary
    // and put16 will then write to the temporary.
    (&mut s[0..N]).try_into().unwrap()
}

/// Get a reference to the first N items of the slice as an array.
/// Panic if the slice is too small.
pub(crate) fn to_array_ref<T, const N: usize>(s: &[T]) -> &[T;N] {
    (&s[0..N]).try_into().unwrap()
}
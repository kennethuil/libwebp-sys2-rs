pub(crate) const MB_FEATURE_TREE_PROBS:usize = 3;
pub(crate) const NUM_MB_SEGMENTS: usize = 4;
pub(crate) const NUM_REF_LF_DELTAS: usize = 4;
pub(crate) const NUM_MODE_LF_DELTAS: usize = 4;    // I4x4, ZERO, *, SPLIT
pub(crate) const MAX_NUM_PARTITIONS: usize = 8;
// Probabilities
pub(crate) const NUM_TYPES: usize = 4;  // 0: i16-AC,  1: i16-DC,  2:chroma-AC,  3:i4-AC
pub(crate) const NUM_BANDS: usize = 8;
pub(crate) const NUM_CTX: usize = 3;
pub(crate) const NUM_PROBAS: usize = 11;

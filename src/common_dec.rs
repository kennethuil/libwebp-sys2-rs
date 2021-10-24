
// intra prediction modes
pub(crate) const B_DC_PRED: u8 = 0; // 4x4 modes
pub(crate) const B_TM_PRED: u8 = 1;
pub(crate) const B_VE_PRED: u8 = 2;
pub(crate) const B_HE_PRED: u8 = 3;
pub(crate) const B_RD_PRED: u8 = 4;
pub(crate) const B_VR_PRED: u8 = 5;
pub(crate) const B_LD_PRED: u8 = 6;
pub(crate) const B_VL_PRED: u8 = 7;
pub(crate) const B_HD_PRED: u8 = 8;
pub(crate) const B_HU_PRED: u8 = 9;

// Luma16 or UV modes
pub(crate) const DC_PRED: u8 = B_DC_PRED;
pub(crate) const V_PRED: u8 = B_VE_PRED;
pub(crate) const H_PRED: u8 = B_HE_PRED;
pub(crate) const TM_PRED: u8 = B_TM_PRED;
pub(crate) const B_PRED: u8 = 10;   // refined I4x4 mode (seems to be unused in C)

// special modes
pub(crate) const B_DC_PRED_NOTOP:u8 = 4;
pub(crate) const B_DC_PRED_NOLEFT:u8 = 5;
pub(crate) const B_DC_PRED_NOTOPLEFT:u8 = 6;




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

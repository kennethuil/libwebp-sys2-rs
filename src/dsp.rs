pub(crate) const BPS: isize = 32;
pub(crate) const UBPS: usize = BPS as usize;

#[repr(C)]
pub(crate) enum WEBP_FILTER_TYPE {
    #[allow(non_camel_case_types, dead_code)]
    WEBP_FILTER_NONE = 0,
    #[allow(non_camel_case_types, dead_code)]
    WEBP_FILTER_HORIZONTAL,
    #[allow(non_camel_case_types, dead_code)]
    WEBP_FILTER_VERTICAL,
    #[allow(non_camel_case_types, dead_code)]
    WEBP_FILTER_GRADIENT,
    #[allow(non_camel_case_types, dead_code)]
    WEBP_FILTER_LAST,   // end marker
    #[allow(non_camel_case_types, dead_code)]
    WEBP_FILTER_BEST,   // meta-types
    #[allow(non_camel_case_types, dead_code)]
    WEBP_FILTER_FAST,
}
#![allow(dead_code)]

use unicode_normalization::UnicodeNormalization;

use crate::{language_d::TextProcessor, text_processors::BASIC_TEXT_PROCESSOR_OPTIONS}; // Needed for normalize_radicals

/// 1 - minInclusive: number;
/// 2 - maxInclusive: number;
pub type CodepointRange = (u32, u32);

// --- CJK Ideograph Ranges (as before) ---
pub const CJK_UNIFIED_IDEOGRAPHS_RANGE: CodepointRange = (0x4e00, 0x9fff);
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_A_RANGE: CodepointRange = (0x3400, 0x4dbf);
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_B_RANGE: CodepointRange = (0x20000, 0x2a6df);
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_C_RANGE: CodepointRange = (0x2a700, 0x2b73f);
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_D_RANGE: CodepointRange = (0x2b740, 0x2b81f);
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_E_RANGE: CodepointRange = (0x2b820, 0x2ceaf);
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_F_RANGE: CodepointRange = (0x2ceb0, 0x2ebef);
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_G_RANGE: CodepointRange = (0x30000, 0x3134f);
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_H_RANGE: CodepointRange = (0x31350, 0x323af);
pub const CJK_UNIFIED_IDEOGRAPHS_EXTENSION_I_RANGE: CodepointRange = (0x2ebf0, 0x2ee5f);
pub const CJK_COMPATIBILITY_IDEOGRAPHS_RANGE: CodepointRange = (0xf900, 0xfaff);
pub const CJK_COMPATIBILITY_IDEOGRAPHS_SUPPLEMENT_RANGE: CodepointRange = (0x2f800, 0x2fa1f);

pub const CJK_IDEOGRAPH_RANGES: [CodepointRange; 12] = [
    CJK_UNIFIED_IDEOGRAPHS_RANGE,
    CJK_UNIFIED_IDEOGRAPHS_EXTENSION_A_RANGE,
    CJK_UNIFIED_IDEOGRAPHS_EXTENSION_B_RANGE,
    CJK_UNIFIED_IDEOGRAPHS_EXTENSION_C_RANGE,
    CJK_UNIFIED_IDEOGRAPHS_EXTENSION_D_RANGE,
    CJK_UNIFIED_IDEOGRAPHS_EXTENSION_E_RANGE,
    CJK_UNIFIED_IDEOGRAPHS_EXTENSION_F_RANGE,
    CJK_UNIFIED_IDEOGRAPHS_EXTENSION_G_RANGE,
    CJK_UNIFIED_IDEOGRAPHS_EXTENSION_H_RANGE,
    CJK_UNIFIED_IDEOGRAPHS_EXTENSION_I_RANGE,
    CJK_COMPATIBILITY_IDEOGRAPHS_RANGE,
    CJK_COMPATIBILITY_IDEOGRAPHS_SUPPLEMENT_RANGE,
];

// --- Other CJK Ranges (as before) ---
pub const FULLWIDTH_CHARACTER_RANGES: [CodepointRange; 8] = [
    (0xff10, 0xff19), // Fullwidth numbers
    (0xff21, 0xff3a), // Fullwidth upper case Latin letters
    (0xff41, 0xff5a), // Fullwidth lower case Latin letters
    (0xff01, 0xff0f), // Fullwidth punctuation 1
    (0xff1a, 0xff1f), // Fullwidth punctuation 2
    (0xff3b, 0xff3f), // Fullwidth punctuation 3
    (0xff5b, 0xff60), // Fullwidth punctuation 4
    (0xffe0, 0xffee), // Currency markers
];

pub const CJK_PUNCTUATION_RANGE: CodepointRange = (0x3000, 0x303f);

// --- NEW: Added CJK_COMPATIBILITY Range ---
pub const CJK_COMPATIBILITY: CodepointRange = (0x3300, 0x33ff);

// --- NEW: Added CJK Radical Ranges ---
pub const KANGXI_RADICALS_RANGE: CodepointRange = (0x2f00, 0x2fdf);
pub const CJK_RADICALS_SUPPLEMENT_RANGE: CodepointRange = (0x2e80, 0x2eff);
pub const CJK_STROKES_RANGE: CodepointRange = (0x31c0, 0x31ef);

pub const CJK_RADICALS_RANGES: [CodepointRange; 3] = [
    KANGXI_RADICALS_RANGE,
    CJK_RADICALS_SUPPLEMENT_RANGE,
    CJK_STROKES_RANGE,
];

// --- Utility Functions (Fixed and New) ---

pub fn is_code_point_in_range(code_point: u32, range: CodepointRange) -> bool {
    // FIX: Ensured it uses <= max
    code_point >= range.0 && code_point <= range.1
}

pub fn is_code_point_in_ranges(code_point: u32, ranges: &[CodepointRange]) -> bool {
    for &(min, max) in ranges {
        if code_point >= min && code_point <= max {
            return true;
        }
    }
    false
}


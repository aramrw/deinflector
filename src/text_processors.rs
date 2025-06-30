use fancy_regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::{
    cjk_utils::{is_code_point_in_ranges, CJK_RADICALS_RANGES},
    language_d::{TextProcessor, TextProcessorSetting},
};

pub const BASIC_TEXT_PROCESSOR_OPTIONS: &[TextProcessorSetting] = &[
    TextProcessorSetting::Bool(false),
    TextProcessorSetting::Bool(true),
];

fn remove_alphabetic_diacritics(text: &str, setting: TextProcessorSetting) -> String {
    if matches!(setting, TextProcessorSetting::Bool(true)) {
        // Normalize the text to NFD (Normalization Form D) and collect into a String
        let normalized: String = text.nfd().collect();
        // Compile regex for diacritic marks
        let re = Regex::new(r"[\u{0300}-\u{036f}]").unwrap();
        // Remove diacritic marks
        let result = re.replace_all(&normalized, "");
        result.to_string()
    } else {
        text.to_string()
    }
}

pub fn decapitalize_helper(text: &str, setting: TextProcessorSetting) -> String {
    if matches!(setting, TextProcessorSetting::Bool(true)) {
        text.to_lowercase()
    } else {
        text.to_string()
    }
}

pub const DECAPITALIZE: TextProcessor = TextProcessor {
    name: "Decapitalize Text",
    description: "CAPITALIZED TEXT → capitalized text",
    options: BASIC_TEXT_PROCESSOR_OPTIONS,
    process: decapitalize_helper,
};

pub fn capitalize_first_letter_helper(text: &str, setting: TextProcessorSetting) -> String {
    if matches!(setting, TextProcessorSetting::Bool(true)) {
        let mut str = text.to_owned();
        if let Some(r) = str.get_mut(0..1) {
            r.make_ascii_uppercase();
            return str;
        }
    }
    text.to_owned()
}

pub const CAPITALIZE_FIRST_LETTER: TextProcessor = TextProcessor {
    name: "Capitalize First Letter",
    description: "lowercase text → Lowercase text",
    options: BASIC_TEXT_PROCESSOR_OPTIONS,
    process: capitalize_first_letter_helper,
};

pub const REMOVE_ALPHABETIC_DIACRITICS: TextProcessor = TextProcessor {
    name: "Remove Alphabetic Diacritics",
    description: "ἄήé → αηe",
    options: BASIC_TEXT_PROCESSOR_OPTIONS,
    process: remove_alphabetic_diacritics,
};

pub fn normalize_radicals(text: &str) -> String {
    text.chars()
        .map(|c| {
            let code_point = c as u32;
            if is_code_point_in_ranges(code_point, &CJK_RADICALS_RANGES) {
                // Use NFKD normalization, same as JS
                c.nfkd().collect::<String>()
            } else {
                c.to_string()
            }
        })
        .collect()
}

fn normalize_radical_characters_helper(text: &str, setting: TextProcessorSetting) -> String {
    if matches!(setting, TextProcessorSetting::Bool(true)) {
        return normalize_radicals(text);
    }
    text.to_owned()
}

pub const NORMALIZE_RADICAL_CHARACTERS: TextProcessor = TextProcessor {
    name: "Normalize radical characters",
    description: "⼀ → 一 (U+2F00 → U+4E00)",
    options: BASIC_TEXT_PROCESSOR_OPTIONS,
    process: normalize_radical_characters_helper,
};

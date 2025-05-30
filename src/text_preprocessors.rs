use crate::{
    ja::japanese::{
        collapse_emphatic_sequences, convert_alphanumeric_to_fullwidth,
        convert_fullwidth_alphanumeric_to_normal, convert_halfwidth_kana_to_fullwidth,
        convert_hiragana_to_katakana, convert_katakana_to_hiragana,
        normalize_cjk_compatibility_characters, normalize_combining_characters,
    },
    language_d::{
        BidirectionalConversionPreProcessor, BidirectionalPreProcessorOptions, TextProcessor,
        TextProcessorSetting,
    },
    text_processors::BASIC_TEXT_PROCESSOR_OPTIONS,
    wanakana::convert_alphabetic_to_kana,
};

use kanji_processor::convert_variants;

fn convert_half_width_characters_helper(text: &str, setting: TextProcessorSetting) -> String {
    if matches!(setting, TextProcessorSetting::Bool(true)) {
        return convert_halfwidth_kana_to_fullwidth(text);
    }
    text.to_owned()
}

/// <bool, bool>
/// [TextProcessorSetting]
pub const CONVERT_HALF_WIDTH_CHARACTERS: TextProcessor = TextProcessor {
    name: "Convert Half Width Characters to Full Width",
    description: "ﾖﾐﾁｬﾝ → ヨミチャン",
    options: &BASIC_TEXT_PROCESSOR_OPTIONS,
    process: convert_half_width_characters_helper,
};

pub fn alphabetic_to_hiragana_helper(text: &str, setting: TextProcessorSetting) -> String {
    if matches!(setting, TextProcessorSetting::Bool(true)) {
        return convert_alphabetic_to_kana(text);
    }
    text.to_owned()
}

pub const ALPHABETIC_TO_HIRAGANA: TextProcessor = TextProcessor {
    name: "Convert Alphabetic Characters to Hiragana",
    description: "yomichan → よみちゃん",
    options: &BASIC_TEXT_PROCESSOR_OPTIONS,
    process: alphabetic_to_hiragana_helper,
};

fn process_alphanumeric_width_variants(s: &str, setting: TextProcessorSetting) -> String {
    match setting {
        TextProcessorSetting::BiDirectional(opt) => match opt {
            BidirectionalPreProcessorOptions::Off => s.to_string(),
            BidirectionalPreProcessorOptions::Direct => convert_fullwidth_alphanumeric_to_normal(s),
            BidirectionalPreProcessorOptions::Inverse => convert_alphanumeric_to_fullwidth(s),
        },
        _ => s.to_string(),
    }
}

pub const ALPHANUMERIC_WIDTH_VARIANTS: BidirectionalConversionPreProcessor =
    BidirectionalConversionPreProcessor {
        name: "Convert Between Alphabetic Width Variants",
        description: "ｙｏｍｉｔａｎ → yomitan and vice versa",
        options: &[
            TextProcessorSetting::BiDirectional(BidirectionalPreProcessorOptions::Off),
            TextProcessorSetting::BiDirectional(BidirectionalPreProcessorOptions::Direct),
            TextProcessorSetting::BiDirectional(BidirectionalPreProcessorOptions::Inverse),
        ],
        process: process_alphanumeric_width_variants,
    };

fn process_hiragana_to_katakana(s: &str, setting: TextProcessorSetting) -> String {
    match setting {
        TextProcessorSetting::BiDirectional(opt) => match opt {
            BidirectionalPreProcessorOptions::Off => s.to_string(),
            BidirectionalPreProcessorOptions::Direct => convert_hiragana_to_katakana(s),
            BidirectionalPreProcessorOptions::Inverse => convert_katakana_to_hiragana(s, false),
        },
        _ => s.to_string(),
    }
}

pub const CONVERT_HIRAGANA_TO_KATAKANA: BidirectionalConversionPreProcessor =
    BidirectionalConversionPreProcessor {
        name: "Convert Hiragana to Katakana",
        description: "よみちゃん → ヨミチャン and vice versa",
        options: &[
            TextProcessorSetting::BiDirectional(BidirectionalPreProcessorOptions::Off),
            TextProcessorSetting::BiDirectional(BidirectionalPreProcessorOptions::Direct),
            TextProcessorSetting::BiDirectional(BidirectionalPreProcessorOptions::Inverse),
        ],
        process: process_hiragana_to_katakana,
    };

fn collapse_emphatic_sequences_helper(text: &str, setting: TextProcessorSetting) -> String {
    let text = text.to_owned();
    match setting {
        TextProcessorSetting::Emphatic(collapse_emphatic, collapse_emphatic_full) => {
            if collapse_emphatic {
                collapse_emphatic_sequences(&text, collapse_emphatic_full)
            } else {
                text
            }
        }
        _ => unreachable!("you should not pass anything other than `TextProcessorSetting::Emphatic(bool, bool)` to `fn collapse_emphatic_sequences_helper`"),
    }
}

pub const COLLAPSE_EMPHATIC_SEQUENCES: TextProcessor = TextProcessor {
    name: "Collapse Emphatic Character Sequences",
    description: "すっっごーーい → すっごーい / すごい",
    options: &[
        TextProcessorSetting::Emphatic(false, false),
        TextProcessorSetting::Emphatic(true, false),
        TextProcessorSetting::Emphatic(true, true),
    ],
    process: collapse_emphatic_sequences_helper,
};

fn normalize_combining_characters_helper(text: &str, setting: TextProcessorSetting) -> String {
    if matches!(setting, TextProcessorSetting::Bool(true)) {
        return normalize_combining_characters(text);
    }
    text.to_owned()
}

pub const NORMALIZE_COMBINING_CHARACTERS: TextProcessor = TextProcessor {
    name: "Normalize Combining Characters",
    description: "ド → ド (U+30C8 U+3099 → U+30C9)",
    options: &BASIC_TEXT_PROCESSOR_OPTIONS,
    process: normalize_combining_characters_helper,
};

fn normalize_cjk_compatibility_characters_helper(
    text: &str,
    setting: TextProcessorSetting,
) -> String {
    if matches!(setting, TextProcessorSetting::Bool(true)) {
        return normalize_cjk_compatibility_characters(text);
    }
    text.to_owned()
}

pub const NORMALIZE_CJK_COMPATIBILITY_CHARACTERS: TextProcessor = TextProcessor {
    name: "Normalize CJK Compatibility Characters",
    description: "㌀ → アパート",
    options: &BASIC_TEXT_PROCESSOR_OPTIONS,
    process: normalize_cjk_compatibility_characters_helper,
};

fn standardize_kanji_helper(text: &str, setting: TextProcessorSetting) -> String {
    if matches!(setting, TextProcessorSetting::Bool(true)) {
        return convert_variants(text);
    }
    text.to_owned()
}

pub const STANDARDIZE_KANJI: TextProcessor = TextProcessor {
    name: "Convert kanji variants to their modern standard form",
    description: "萬 → 万",
    options: &BASIC_TEXT_PROCESSOR_OPTIONS,
    process: standardize_kanji_helper,
};

// You might also need NORMALIZE_RADICAL_CHARACTERS if you intend to keep it,
// but it's not in the JS provided. If you want strict JS parity, remove it
// from descriptors.rs. If you need it, you'll have to define it here.

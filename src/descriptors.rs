use indexmap::IndexMap;
use std::{collections::HashMap, sync::LazyLock};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    en::en_transforms::ENGLISH_TRANSFORMS_DESCRIPTOR,
    ja::{self, ja_transforms::JAPANESE_TRANSFORMS_DESCRIPTOR},
    japanese::is_string_partially_japanese,
    language_d::{
        AnyTextProcessor, BidirectionalConversionPreProcessor, ReadingNormalizer, TextProcessor,
        TextProcessorWithId,
    },
    text_preprocessors::{
        ALPHABETIC_TO_HIRAGANA, ALPHANUMERIC_WIDTH_VARIANTS, COLLAPSE_EMPHATIC_SEQUENCES,
        CONVERT_HALF_WIDTH_CHARACTERS, CONVERT_HIRAGANA_TO_KATAKANA,
        NORMALIZE_CJK_COMPATIBILITY_CHARACTERS, NORMALIZE_COMBINING_CHARACTERS, STANDARDIZE_KANJI,
    },
    text_processors::{CAPITALIZE_FIRST_LETTER, DECAPITALIZE, NORMALIZE_RADICAL_CHARACTERS},
    transformer::LanguageTransformDescriptor,
};

pub fn collect_graphemes(text: &str) -> Vec<&str> {
    text.graphemes(true).collect::<Vec<&str>>()
}

type IsTextLookupWorthyFP = fn(text: &str) -> bool;

pub struct LanguageDescriptor {
    pub iso: &'static str,
    pub iso639_3: &'static str,
    pub name: &'static str,
    pub example_text: &'static str,
    pub is_text_lookup_worthy: Option<IsTextLookupWorthyFP>,
    pub reading_normalizer: Option<ReadingNormalizer>,
    pub text_processors: PreAndPostProcessors,
    pub language_transforms: Option<&'static LanguageTransformDescriptor>,
}

/// This is a mapping of the iso tag to all of the text processors for that language.
/// Any new language should be added to this struct.
#[derive(Debug, Clone)]
pub struct AllLanguageTextProcessors {
    ja: PreAndPostProcessors,
    en: PreAndPostProcessors,
}

#[derive(Debug, Clone, Default)]
pub struct PreAndPostProcessors {
    pub pre: Vec<TextProcessorWithId>,
    pub post: Vec<TextProcessorWithId>,
}

#[derive(Debug, Clone)]
pub struct PreAndPostProcessorsWithId {
    pub pre: Vec<PreProcessorsWithId>,
    pub post: Vec<PostProcessorsWithId>,
}

#[derive(Debug, Clone)]
pub enum PreProcessors {
    Ja(Box<JapanesePreProcessors>),
    En(CapitalizationPreProcessors),
}
#[derive(Debug, Clone)]
pub enum PostProcessors {
    None,
}
#[derive(Debug, Clone)]
pub enum PreProcessorsWithId {
    None,
}
#[derive(Debug, Clone)]
pub enum PostProcessorsWithId {
    None,
}

type TextProcessorDescriptor<T, F> = HashMap<String, TextProcessor<T, F>>;

#[derive(Debug, Clone)]
pub struct CapitalizationPreProcessors {
    pub decapitalize: TextProcessor<bool, bool>,
    pub capitalize_first_letter: TextProcessor<bool, bool>,
}

#[derive(Debug, Clone)]
struct AlphabeticDiacriticsProcessor<F> {
    pub remove_alphabetic_diacritics: TextProcessor<bool, F>,
}

#[derive(Debug, Clone)]
pub struct JapanesePreProcessors {
    pub convert_half_width_characters: TextProcessor<bool, bool>,
    pub alphabetic_to_hiragana: TextProcessor<bool, bool>,
    pub normalize_combining_characters: TextProcessor<bool, bool>,
    pub alphanumeric_width_variants: BidirectionalConversionPreProcessor,
    pub convert_hiragana_to_katakana: BidirectionalConversionPreProcessor,
    pub collapse_emphatic_sequences: TextProcessor<[bool; 2], [bool; 2]>,
}

// #[derive(Debug, Clone)]
// pub struct CapitalizationPreProcessors {
//     pub decapitalize: TextProcessor<bool, bool>,
// }

pub static LANGUAGE_DESCRIPTOR_MAP: LazyLock<IndexMap<&str, LanguageDescriptor>> =
    LazyLock::new(|| {
        IndexMap::from([
            (
                "ja",
                LanguageDescriptor {
                    iso: "ja",
                    iso639_3: "jpn",
                    name: "Japanese",
                    example_text: "読め",
                    is_text_lookup_worthy: Some(is_string_partially_japanese),
                    reading_normalizer: None,
                    text_processors: PreAndPostProcessors {
                        pre: vec![
                            TextProcessorWithId {
                                id: "convert_half_width_characters",
                                processor: AnyTextProcessor::ConvertHalfWidth(
                                    CONVERT_HALF_WIDTH_CHARACTERS,
                                ),
                            },
                            TextProcessorWithId {
                                id: "alphabetic_to_hiragana",
                                processor: AnyTextProcessor::AlphabeticToHiragana(
                                    ALPHABETIC_TO_HIRAGANA,
                                ),
                            },
                            TextProcessorWithId {
                                id: "normalize_combining_characters",
                                processor: AnyTextProcessor::NormalizeCombiningCharacters(
                                    NORMALIZE_COMBINING_CHARACTERS,
                                ),
                            },
                            TextProcessorWithId {
                                id: "normalize_cjk_compatibility_characters",
                                processor: AnyTextProcessor::NormalizeCjkCompatibilityCharacters(
                                    NORMALIZE_CJK_COMPATIBILITY_CHARACTERS,
                                ),
                            },
                            TextProcessorWithId {
                                id: "normalize_radical_characters",
                                processor: AnyTextProcessor::NormalizeRadicalCharacters(
                                    NORMALIZE_RADICAL_CHARACTERS,
                                ),
                            },
                            TextProcessorWithId {
                                id: "standardize_kanji",
                                processor: AnyTextProcessor::StandardizeKanji(STANDARDIZE_KANJI),
                            },
                            TextProcessorWithId {
                                id: "alphanumeric_width_variants",
                                processor: AnyTextProcessor::AlphanumericWidth(
                                    ALPHANUMERIC_WIDTH_VARIANTS,
                                ),
                            },
                            TextProcessorWithId {
                                id: "convert_hiragana_to_katakana",
                                processor: AnyTextProcessor::HiraganaToKatakana(
                                    CONVERT_HIRAGANA_TO_KATAKANA,
                                ),
                            },
                            TextProcessorWithId {
                                id: "collapse_emphatic_sequences",
                                processor: AnyTextProcessor::CollapseEmphatic(
                                    COLLAPSE_EMPHATIC_SEQUENCES,
                                ),
                            },
                        ],
                        post: vec![],
                    },
                    language_transforms: Some(&*JAPANESE_TRANSFORMS_DESCRIPTOR),
                },
            ),
            (
                "en",
                LanguageDescriptor {
                    iso: "en",
                    iso639_3: "eng",
                    name: "English",
                    example_text: "read",
                    is_text_lookup_worthy: None,
                    reading_normalizer: None,
                    text_processors: PreAndPostProcessors {
                        pre: vec![
                            TextProcessorWithId {
                                id: "decapitalize",
                                processor: AnyTextProcessor::Decapitalize(DECAPITALIZE),
                            },
                            TextProcessorWithId {
                                id: "capitalize_first_letter",
                                processor: AnyTextProcessor::CapitalizeFirst(
                                    CAPITALIZE_FIRST_LETTER,
                                ),
                            },
                        ],
                        post: vec![],
                    },
                    language_transforms: Some(&*ENGLISH_TRANSFORMS_DESCRIPTOR),
                },
            ),
        ])
    });

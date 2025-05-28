use indexmap::IndexMap;
use std::{collections::HashMap, sync::LazyLock};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    en::en_transforms::ENGLISH_TRANSFORMS_DESCRIPTOR,
    ja::ja_transforms::JAPANESE_TRANSFORMS_DESCRIPTOR,
    japanese::is_string_partially_japanese,
    language_d::{BidirectionalConversionPreProcessor, ReadingNormalizer, TextProcessor},
    text_preprocessors::{
        ALPHABETIC_TO_HIRAGANA, ALPHANUMERIC_WIDTH_VARIANTS, COLLAPSE_EMPHATIC_SEQUENCES,
        CONVERT_HALF_WIDTH_CHARACTERS, CONVERT_HIRAGANA_TO_KATAKANA,
        NORMALIZE_COMBINING_CHARACTERS,
    },
    text_processors::{CAPITALIZE_FIRST_LETTER, DECAPITALIZE},
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

type TextProcessorDescriptor<T, F> = HashMap<String, TextProcessor<T, F>>;

#[derive(Debug, Clone)]
struct CapitalizationPreProcessors {
    decapitalize: TextProcessor<bool, bool>,
    capitalize_first_letter: TextProcessor<bool, bool>,
}

#[derive(Debug, Clone)]
struct AlphabeticDiacriticsProcessor<F> {
    remove_alphabetic_diacritics: TextProcessor<bool, F>,
}

/// This is a mapping of the iso tag to all of the text processors for that language.
/// Any new language should be added to this struct.
#[derive(Debug, Clone)]
pub struct AllTextProcessors {
    ja: PreAndPostProcessors,
}

#[derive(Debug, Clone)]
pub enum PreProcessors {
    Ja(JapanesePreProcessors),
    En(CapitalizationPreProcessors),
}
#[derive(Debug, Clone)]
pub enum PostProcessors {
    None,
}

#[derive(Debug, Clone)]
pub struct PreAndPostProcessors {
    pub pre: Option<PreProcessors>,
    pub post: Option<PostProcessors>,
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

pub static LANGUAGE_DESCRIPTORS_MAP: LazyLock<IndexMap<&str, LanguageDescriptor>> =
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
                        pre: Some(PreProcessors::Ja(JapanesePreProcessors {
                            convert_half_width_characters: CONVERT_HALF_WIDTH_CHARACTERS,
                            alphabetic_to_hiragana: ALPHABETIC_TO_HIRAGANA,
                            normalize_combining_characters: NORMALIZE_COMBINING_CHARACTERS,
                            alphanumeric_width_variants: ALPHANUMERIC_WIDTH_VARIANTS,
                            convert_hiragana_to_katakana: CONVERT_HIRAGANA_TO_KATAKANA,
                            collapse_emphatic_sequences: COLLAPSE_EMPHATIC_SEQUENCES,
                        })),
                        post: None,
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
                        pre: Some(PreProcessors::En(CapitalizationPreProcessors {
                            decapitalize: DECAPITALIZE,
                            capitalize_first_letter: CAPITALIZE_FIRST_LETTER,
                        })),
                        post: None,
                    },
                    language_transforms: Some(&*ENGLISH_TRANSFORMS_DESCRIPTOR),
                },
            ),
        ])
    });

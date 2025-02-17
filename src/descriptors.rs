use indexmap::IndexMap;
use std::{collections::HashMap, sync::LazyLock};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    ja::ja_transforms::JAPANESE_TRANSFORMS,
    japanese::is_string_partially_japanese,
    language_d::{BidirectionalConversionPreProcessor, ReadingNormalizer, TextProcessor},
    text_preprocessors::{
        ALPHABETIC_TO_HIRAGANA, ALPHANUMERIC_WIDTH_VARIANTS, COLLAPSE_EMPHATIC_SEQUENCES,
        CONVERT_HALF_WIDTH_CHARACTERS, CONVERT_HIRAGANA_TO_KATAKANA,
        NORMALIZE_COMBINING_CHARACTERS,
    },
    transformer::LanguageTransformDescriptor,
};

pub fn collect_graphemes(text: &str) -> Vec<&str> {
    text.graphemes(true).collect::<Vec<&str>>()
}

type IsTextLookupWorthyFP = fn(text: &str) -> bool;

pub struct LanguageDescriptor<Pre, Post> {
    pub iso: String,
    pub iso639_3: String,
    pub name: String,
    pub example_text: String,
    pub is_text_lookup_worthy: Option<IsTextLookupWorthyFP>,
    pub reading_normalizer: Option<ReadingNormalizer>,
    pub text_processors: PreAndPostProcessors<Pre, Post>,
    pub language_transforms: Option<&'static LanguageTransformDescriptor>,
}

type TextProcessorDescriptor<'a, T, F> = HashMap<String, TextProcessor<'a, T, F>>;

struct CapitalizationPreProcessors<'a, F> {
    capitalize_first_letter: TextProcessor<'a, bool, F>,
    decapitalize: TextProcessor<'a, bool, F>,
}

struct AlphabeticDiacriticsProcessor<'a, F> {
    remove_alphabetic_diacritics: TextProcessor<'a, bool, F>,
}

/// This is a mapping of the iso tag to all of the text processors for that language.
/// Any new language should be added to this struct.
pub struct AllTextProcessors<'a> {
    ja: PreAndPostProcessors<JapanesePreProcessors<'a>, ()>,
}

#[derive(Clone)]
pub struct PreAndPostProcessors<Pre, Post> {
    pub pre: Pre,
    pub post: Option<Post>,
}

// Language Processor structs get created here
#[derive(Clone)]
pub struct JapanesePreProcessors<'a> {
    pub convert_half_width_characters: TextProcessor<'a, bool, bool>,
    pub alphabetic_to_hiragana: TextProcessor<'a, bool, bool>,
    pub normalize_combining_characters: TextProcessor<'a, bool, bool>,
    pub alphanumeric_width_variants: BidirectionalConversionPreProcessor<'a>,
    pub convert_hiragana_to_katakana: BidirectionalConversionPreProcessor<'a>,
    pub collapse_emphatic_sequences: TextProcessor<'a, [bool; 2], &'a [bool; 2]>,
}

pub static LANGUAGE_DESCRIPTORS_MAP: LazyLock<
    IndexMap<&str, LanguageDescriptor<JapanesePreProcessors<'static>, ()>>,
> = LazyLock::new(|| {
    IndexMap::from([(
        "ja",
        LanguageDescriptor {
            iso: "ja".into(),
            iso639_3: "jpn".into(),
            name: "Japanese".into(),
            example_text: "読め".into(),
            is_text_lookup_worthy: Some(is_string_partially_japanese),
            reading_normalizer: None,
            text_processors: PreAndPostProcessors {
                pre: JapanesePreProcessors {
                    convert_half_width_characters: CONVERT_HALF_WIDTH_CHARACTERS,
                    alphabetic_to_hiragana: ALPHABETIC_TO_HIRAGANA,
                    normalize_combining_characters: NORMALIZE_COMBINING_CHARACTERS,
                    alphanumeric_width_variants: ALPHANUMERIC_WIDTH_VARIANTS,
                    convert_hiragana_to_katakana: CONVERT_HIRAGANA_TO_KATAKANA,
                    collapse_emphatic_sequences: COLLAPSE_EMPHATIC_SEQUENCES,
                },
                post: None,
            },
            language_transforms: Some(&*JAPANESE_TRANSFORMS),
        },
    )])
});

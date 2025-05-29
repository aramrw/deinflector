use std::sync::Arc;

use crate::{
    descriptors::{PostProcessors, PreProcessors},
    language_d::{AnyTextProcessor, TextProcessorWithId},
};

use super::{
    descriptors::{self, LANGUAGE_DESCRIPTOR_MAP},
    language_d::{
        LanguageAndProcessors, LanguageAndReadingNormalizer, LanguageAndTransforms, LanguageSummary,
    },
};

pub fn get_language_summaries() -> Vec<LanguageSummary> {
    LANGUAGE_DESCRIPTOR_MAP
        .values()
        .map(|entry| LanguageSummary {
            name: entry.name,
            iso: entry.iso,
            iso639_3: entry.iso639_3,
            example_text: entry.example_text,
        })
        .collect::<Vec<LanguageSummary>>()
}

fn test_get_language_summaries() {
    let s = get_language_summaries();
    dbg!(s);
}

pub fn get_all_language_reading_normalizers() -> Vec<LanguageAndReadingNormalizer> {
    LANGUAGE_DESCRIPTOR_MAP
        .values()
        .filter_map(|entry| {
            if let Some(reading_normalizer) = entry.reading_normalizer {
                return Some(LanguageAndReadingNormalizer {
                    iso: entry.iso,
                    reading_normalizer,
                });
            };
            None
        })
        .collect::<Vec<LanguageAndReadingNormalizer>>()
}

pub fn is_text_lookup_worthy(text: &str, language: &str) -> bool {
    if let Some(descriptor) = LANGUAGE_DESCRIPTOR_MAP.get(language) {
        if let Some(itlw_fn) = descriptor.is_text_lookup_worthy {
            return itlw_fn(text);
        }
    }
    false
}

pub fn get_all_language_transform_descriptors() -> Vec<LanguageAndTransforms> {
    let mut results: Vec<LanguageAndTransforms> = Vec::new();
    for entry in LANGUAGE_DESCRIPTOR_MAP.values() {
        if let Some(language_transforms) = entry.language_transforms {
            let item = LanguageAndTransforms {
                iso: entry.iso,
                language_transforms: language_transforms.clone(),
            };
            results.push(item);
        }
    }
    results
}

// Retrieves a list of language processors for all configured languages.
///
/// This function iterates over the `LANGUAGE_DESCRIPTOR_MAP`, and for each language,
/// it extracts the ISO code, and clones its pre-defined text preprocessors and postprocessors.
/// The structure of `LanguageDescriptor` in Rust already stores processors in a format
/// (`Vec<TextProcessorWithId>`) that matches the desired output, simplifying the translation
/// from the JavaScript equivalent which performs an object-to-array transformation.
///
/// # Returns
///
/// A `Vec<LanguageAndProcessors>` where each element contains the ISO code
/// and the respective lists of preprocessors and postprocessors.
pub fn get_all_language_text_processors() -> Vec<LanguageAndProcessors> {
    let mut processor_results = Vec::with_capacity(LANGUAGE_DESCRIPTOR_MAP.values().len());

    for lang_descriptor in LANGUAGE_DESCRIPTOR_MAP.values() {
        let iso = lang_descriptor.iso;
        let preprocessors: Vec<TextProcessorWithId> = lang_descriptor.text_processors.pre.clone();
        let postprocessors: Vec<TextProcessorWithId> = lang_descriptor.text_processors.post.clone();
        processor_results.push(LanguageAndProcessors {
            iso,
            preprocessors,
            postprocessors,
        });
    }
    processor_results
}

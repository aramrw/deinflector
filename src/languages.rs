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

// pub fn get_all_language_text_processors() -> Vec<LanguageAndProcessors> {
//     let mut results = Vec::new();
//
//     // Iterate through your existing map
//     for (iso, descriptor) in LANGUAGE_DESCRIPTOR_MAP.iter() {
//         let mut pre_vec = Vec::new();
//         // so far for all languages implemented this doesn't get used (yet)
//         let mut post_vec = Vec::new();
//
//         // Handle Pre-processors
//         if let Some(pre_enum) = &descriptor.text_processors.pre {
//             match pre_enum {
//                 PreProcessors::Ja(jp) => {
//                     // Manually map each field to a TextProcessorWithId
//                     pre_vec.push(TextProcessorWithId {
//                         id: "convert_half_width_characters",
//                         processor: AnyTextProcessor::ConvertHalfWidth(
//                             jp.convert_half_width_characters.clone(),
//                         ),
//                     });
//                     pre_vec.push(TextProcessorWithId {
//                         id: "alphabetic_to_hiragana",
//                         processor: AnyTextProcessor::AlphabeticToHiragana(
//                             jp.alphabetic_to_hiragana.clone(),
//                         ),
//                     });
//                     pre_vec.push(TextProcessorWithId {
//                         id: "normalize_combining_characters",
//                         processor: AnyTextProcessor::NormalizeCombining(
//                             jp.normalize_combining_characters.clone(),
//                         ),
//                     });
//                     pre_vec.push(TextProcessorWithId {
//                         id: "alphanumeric_width_variants",
//                         processor: AnyTextProcessor::AlphanumericWidth(
//                             jp.alphanumeric_width_variants.clone(),
//                         ),
//                     });
//                     pre_vec.push(TextProcessorWithId {
//                         id: "convert_hiragana_to_katakana",
//                         processor: AnyTextProcessor::HiraganaToKatakana(
//                             jp.convert_hiragana_to_katakana.clone(),
//                         ),
//                     });
//                     pre_vec.push(TextProcessorWithId {
//                         id: "collapse_emphatic_sequences",
//                         processor: AnyTextProcessor::CollapseEmphatic(
//                             jp.collapse_emphatic_sequences.clone(),
//                         ),
//                     });
//                 }
//                 PreProcessors::En(en) => {
//                     pre_vec.push(TextProcessorWithId {
//                         id: "decapitalize",
//                         processor: AnyTextProcessor::Decapitalize(en.decapitalize.clone()),
//                     });
//                     pre_vec.push(TextProcessorWithId {
//                         id: "capitalize_first_letter",
//                         processor: AnyTextProcessor::CapitalizeFirst(
//                             en.capitalize_first_letter.clone(),
//                         ),
//                     });
//                 }
//             }
//         }
//
//         // Handle Post-processors (Example)
//         if let Some(post_enum) = &descriptor.text_processors.post {
//             match post_enum {
//                 // Add matches if you ever add post-processors
//                 // Example: PostProcessors::SomeLang(sl) => { post_vec.push(...); }
//                 PostProcessors::None => { /* Do nothing */ }
//             }
//         }
//
//         results.push(LanguageAndProcessors {
//             iso: iso.to_string(),
//             preprocessors: pre_vec,
//             postprocessors: post_vec,
//         });
//     }
//
//     results
// }

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

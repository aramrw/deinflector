use std::{
    collections::HashSet,
    sync::{Arc, LazyLock},
};

use fancy_regex::Regex;
use indexmap::IndexMap;

use crate::{
    ja::ja_transforms::{LanguageTransformerTestCase, TransformTest},
    transformer::{
        Condition, ConditionMap, DeinflectFnType, LanguageTransformDescriptor, Rule, RuleType,
        SuffixRule, Transform, TransformMap,
    },
    transforms::{inflection, suffix_inflection},
};

fn doubled_consonant_inflection(
    consonants: &'static str,
    suffix: &'static str,
    conditions_in: &'static [&'static str],
    conditions_out: &'static [&'static str],
) -> Vec<SuffixRule> {
    let fmt = |csn: &char| format!("{csn}{csn}{suffix}");
    let inflections: Vec<SuffixRule> = consonants
        .chars()
        .map(|csn| {
            let cstr = csn.to_string().leak();
            suffix_inflection(&fmt(&csn), cstr, conditions_in, conditions_out)
        })
        .collect();
    inflections
}

pub static PAST_SUFFIX_INFLECTIONS: LazyLock<Vec<SuffixRule>> = LazyLock::new(|| {
    [
        suffix_inflection("ed", "", &["v"], &["v"]),   // "walked"
        suffix_inflection("ed", "e", &["v"], &["v"]),  // "hoped"
        suffix_inflection("ied", "y", &["v"], &["v"]), // "tried"
        suffix_inflection("cked", "c", &["v"], &["v"]), // "frolicked"
        suffix_inflection("laid", "lay", &["v"], &["v"]),
        suffix_inflection("paid", "pay", &["v"], &["v"]),
        suffix_inflection("said", "say", &["v"], &["v"]),
    ]
    .into_iter()
    .chain(doubled_consonant_inflection(
        "bdgklmnprstz",
        "ed",
        &["v"],
        &["v"],
    ))
    .collect()
});

pub static ING_SUFFIX_INFLECTIONS: LazyLock<Vec<SuffixRule>> = LazyLock::new(|| {
    [
        suffix_inflection("ing", "", &["v"], &["v"]), // "walking"
        suffix_inflection("ing", "e", &["v"], &["v"]), // "driving"
        suffix_inflection("ying", "ie", &["v"], &["v"]), // "lying"
        suffix_inflection("cking", "c", &["v"], &["v"]), // "panicking"]
    ]
    .into_iter()
    .chain(doubled_consonant_inflection(
        "bdgklmnprstz",
        "ing",
        &["v"],
        &["v"],
    ))
    .collect()
});

pub static THIRD_PERSON_SG_PRESENT_SUFFIX_INFLECTIONS: LazyLock<[SuffixRule; 3]> =
    LazyLock::new(|| {
        [
            suffix_inflection("s", "", &["v"], &["v"]),    // "walks"
            suffix_inflection("es", "", &["v"], &["v"]),   // "teaches"
            suffix_inflection("ies", "y", &["v"], &["v"]), // "tries"
        ]
    });

#[rustfmt::skip]
const PHRASAL_VERB_PARTICLES: [&str; 57] =
    ["aboard", "about", "above", "across", "ahead", "alongside", "apart", "around", "aside", "astray", "away", "back", "before", "behind", "below", "beneath", "besides", "between", "beyond", "by", "close", "down", "east", "west", "north", "south", "eastward", "westward", "northward", "southward", "forward", "backward", "backwards", "forwards", "home", "in", "inside", "instead", "near", "off", "on", "opposite", "out", "outside", "over", "overhead", "past", "round", "since", "through", "throughout", "together", "under", "underneath", "up", "within", "without"];
#[rustfmt::skip]
pub const PHRASAL_VERB_PREPOSITIONS: [&str; 50] =  ["aback", "about", "above", "across", "after", "against", "ahead", "along", "among", "apart", "around", "as", "aside", "at", "away", "back", "before", "behind", "below", "between", "beyond", "by", "down", "even", "for", "forth", "forward", "from", "in", "into", "of", "off", "on", "onto", "open", "out", "over", "past", "round", "through", "to", "together", "toward", "towards", "under", "up", "upon", "way", "with", "without"];

pub static PARTICLES_DISJUNCTION: LazyLock<String> =
    LazyLock::new(|| PHRASAL_VERB_PARTICLES.join("|"));
pub static PHRASAL_VERB_WORD_SET: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    HashSet::from_iter(
        PHRASAL_VERB_PARTICLES
            .into_iter()
            .chain(PHRASAL_VERB_PREPOSITIONS),
    )
});
pub static PHRASAL_VERB_WORD_DISJUNCTION: LazyLock<String> = LazyLock::new(|| {
    PHRASAL_VERB_WORD_SET
        .iter()
        .copied()
        .collect::<Vec<&str>>()
        .join("|")
});

pub static PHRASAL_VERB_INTERPOSED_OBJECT_RULE: LazyLock<Rule> = LazyLock::new(|| Rule {
    rule_type: RuleType::Other,
    is_inflected: fancy_regex::Regex::new(&format!(
        r"^\w* (?:(?!\b({})\b).)+ (?:{})",
        &*PHRASAL_VERB_WORD_DISJUNCTION, &*PARTICLES_DISJUNCTION
    ))
    .unwrap(),
    // deinflected is not necessary for this fn
    deinflected: None,
    deinflect_fn: DeinflectFnType::EnPhrasalVerbInterposedObjectRule,
    conditions_in: &[],
    conditions_out: &["v_phr"],
});

/// [`DeinflectFnType::EnglishPhrasalVerbInflection`]
fn create_phrasal_verb_inflection(inflected: &'static str, deinflected: &'static str) -> Rule {
    let rgx = format!("^\\w{} (?:${})", inflected, &*PHRASAL_VERB_WORD_DISJUNCTION);
    Rule {
        rule_type: RuleType::Other,
        is_inflected: Regex::new(&rgx).unwrap(),
        deinflected: Some(deinflected),
        deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
        conditions_in: &["v"],
        conditions_out: &["v_phr"],
    }
}

fn create_phrasal_verb_inflections_from_suffix_inflections(
    source_rules: &[SuffixRule],
) -> Vec<Rule> {
    source_rules
        .iter()
        .flat_map(|sr| {
            // in js deinflected is checked with typeof === 'undefined'
            // assume that means an empty string
            if sr.deinflected.is_empty() {
                vec![]
            } else {
                // remove trailing '$' from is_inflected
                let inflected_suffix = sr.is_inflected.as_str().replace('$', "").leak();
                let deinflected_suffix = &sr.deinflected;
                // create verb inflection based on suffixes
                vec![create_phrasal_verb_inflection(
                    inflected_suffix,
                    deinflected_suffix,
                )]
            }
        })
        .collect()
}

static ENGLISH_TRANSFORMS_DESCRIPTOR: LazyLock<LanguageTransformDescriptor> =
    LazyLock::new(|| LanguageTransformDescriptor {
        language: "en",
        conditions: &EN_CONDITIONS_MAP,
        transforms: &EN_TRANSFORMS_MAP,
    });

static EN_CONDITIONS_MAP: LazyLock<ConditionMap> = LazyLock::new(|| {
    ConditionMap(IndexMap::from([
        (
            "v".into(),
            Condition {
                name: "Verb".into(),
                is_dictionary_form: true,
                sub_conditions: Some(&["v_phr"]),
                i18n: None,
            },
        ),
        (
            "v_phr".into(),
            Condition {
                name: "Phrasal verb".into(),
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "n".into(),
            Condition {
                name: "Noun".into(),
                is_dictionary_form: true,
                sub_conditions: Some(&["np", "ns"]),
                i18n: None,
            },
        ),
        (
            "np".into(),
            Condition {
                name: "Noun plural".into(),
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "ns".into(),
            Condition {
                name: "Noun singular".into(),
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "adj".into(),
            Condition {
                name: "Adjective".into(),
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "adv".into(),
            Condition {
                name: "Adverb".into(),
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
    ]))
});

static EN_TRANSFORMS_MAP: LazyLock<TransformMap> = LazyLock::new(|| {
    TransformMap(IndexMap::from([
        (
            "plural",
            Transform {
                name: "plural",
                description: Some("Plural form of a noun"),
                rules: vec![suffix_inflection("s", "", &["np"], &["ns"]).into()],
                i18n: None,
            },
        ),
        (
            "possessive",
            Transform {
                name: "possessive",
                description: Some("Possessive form of a noun"),
                rules: vec![
                    suffix_inflection("'s", "", &["n"], &["n"]).into(),
                    suffix_inflection("s'", "s", &["n"], &["n"]).into(),
                ],
                i18n: None,
            },
        ),
        // Past tense
        (
            "past",
            Transform {
                name: "past",
                description: Some("Simple past tense of a verb"),
                rules: PAST_SUFFIX_INFLECTIONS
                    .clone()
                    .into_iter()
                    .map(|si| si.into())
                    .chain(create_phrasal_verb_inflections_from_suffix_inflections(
                        &PAST_SUFFIX_INFLECTIONS,
                    ))
                    .collect(),

                i18n: None,
            },
        ),
        // Present participle
        (
            "ing",
            Transform {
                name: "ing",
                description: Some("Present participle of a verb"),
                rules: ING_SUFFIX_INFLECTIONS
                    .clone()
                    .into_iter()
                    .map(|v| v.into())
                    .chain(create_phrasal_verb_inflections_from_suffix_inflections(
                        &ING_SUFFIX_INFLECTIONS,
                    ))
                    .collect(),
                i18n: None,
            },
        ),
        // Third person singular present
        (
            "3rd pers. sing. pres",
            Transform {
                name: "3rd pers. sing. pres",
                description: Some("Third person singular present tense of a verb"),
                rules: THIRD_PERSON_SG_PRESENT_SUFFIX_INFLECTIONS
                    .clone()
                    .into_iter()
                    .map(|v| v.into())
                    .chain(create_phrasal_verb_inflections_from_suffix_inflections(
                        &*THIRD_PERSON_SG_PRESENT_SUFFIX_INFLECTIONS,
                    ))
                    .collect(),
                i18n: None,
            },
        ),
        // Interposed object
        (
            "interposed object",
            Transform {
                name: "interposed object",
                description: Some("Phrasal verb with interposed object"),
                rules: vec![PHRASAL_VERB_INTERPOSED_OBJECT_RULE.clone()],
                i18n: None,
            },
        ),
        // Archaic form
        (
            "archaic",
            Transform {
                name: "archaic",
                description: Some("Archaic form of a word"),
                rules: vec![suffix_inflection("'d", "ed", &["v"], &["v"]).into()],
                i18n: None,
            },
        ),
        // Adverb form
        (
            "adverb",
            Transform {
                name: "adverb",
                description: Some("Adverb form of an adjective"),
                rules: vec![
                    suffix_inflection("ly", "", &["adv"], &["adj"]).into(), // 'quickly'
                    suffix_inflection("ily", "y", &["adv"], &["adj"]).into(), // 'happily'
                    suffix_inflection("ly", "le", &["adv"], &["adj"]).into(), // 'humbly'
                ],
                i18n: None,
            },
        ),
        // Comparative form
        (
            "comparative",
            Transform {
                name: "comparative",
                description: Some("Comparative form of an adjective"),
                i18n: None,
                rules: vec![
                    suffix_inflection("er", "", &["adj"], &["adj"]).into(), // 'faster'
                    suffix_inflection("er", "e", &["adj"], &["adj"]).into(), // 'nicer'
                    suffix_inflection("ier", "y", &["adj"], &["adj"]).into(), // 'happier'
                ]
                .into_iter()
                .chain(
                    doubled_consonant_inflection("bdgmnt", "er", &["adj"], &["adj"])
                        .into_iter()
                        .map(|sr| sr.into()),
                )
                .collect(),
            },
        ),
        // Superlative form
        (
            "superlative",
            Transform {
                name: "superlative",
                description: Some("Superlative form of an adjective"),
                rules: vec![
                    suffix_inflection("est", "", &["adj"], &["adj"]).into(), // 'fastest'
                    suffix_inflection("est", "e", &["adj"], &["adj"]).into(), // 'nicest'
                    suffix_inflection("iest", "y", &["adj"], &["adj"]).into(), // 'happiest
                ]
                .into_iter()
                .chain(
                    doubled_consonant_inflection("bdgmnt", "est", &["adj"], &["adj"])
                        .into_iter()
                        .map(|sr| sr.into()),
                )
                .collect(),
                i18n: None,
            },
        ),
        // Dropped g
        (
            "dropped g",
            Transform {
                name: "dropped g",
                description: Some("Dropped g in -ing form of a verb"),
                rules: vec![suffix_inflection("in'", "ing", &["v"], &["v"]).into()],
                i18n: None,
            },
        ),
        // -y form
        (
            "-y",
            Transform {
                name: "-y",
                description: Some("Adjective formed from a verb or noun"),
                rules: vec![
                    suffix_inflection("y", "", &["adj"], &["n", "v"]).into(), // 'dirty', 'pushy'
                    suffix_inflection("y", "e", &["adj"], &["n", "v"]).into(), // 'hazy'
                ]
                .into_iter()
                .chain(
                    doubled_consonant_inflection("glmnprst", "y", &[], &["n", "v"])
                        .into_iter()
                        .map(|sr| sr.into()),
                )
                .collect(),
                i18n: None,
            },
        ),
        // un- prefix
        (
            "un-",
            Transform {
                name: "un-",
                description: Some("Negative form of an adjective, adverb, or verb"),
                rules: vec![inflection(
                    "un",
                    "",
                    &["adj", "adv", "v"],
                    &["adj", "adv", "v"],
                    RuleType::Prefix,
                )],
                i18n: None,
            },
        ),
        // going-to future
        (
            "going-to future",
            Transform {
                name: "going-to future",
                description: Some("Going-to future tense of a verb"),
                rules: vec![inflection(
                    "going to ",
                    "",
                    &["v"],
                    &["v"],
                    RuleType::Prefix,
                )],
                i18n: None,
            },
        ),
        // will future
        (
            "will future",
            Transform {
                name: "will future",
                description: Some("Will-future tense of a verb"),
                rules: vec![inflection("will ", "", &["v"], &["v"], RuleType::Prefix)],
                i18n: None,
            },
        ),
        // imperative negative
        (
            "imperative negative",
            Transform {
                name: "imperative negative",
                description: Some("Negative imperative form of a verb"),
                rules: vec![
                    inflection("don't ", "", &["v"], &["v"], RuleType::Prefix),
                    inflection("do not ", "", &["v"], &["v"], RuleType::Prefix),
                ],
                i18n: None,
            },
        ),
        // -able suffix
        (
            "-able",
            Transform {
                name: "-able",
                description: Some("Adjective formed from a verb"),
                rules: vec![
                    suffix_inflection("able", "", &["v"], &["adj"]).into(),
                    suffix_inflection("able", "e", &["v"], &["adj"]).into(),
                    suffix_inflection("iable", "y", &["v"], &["adj"]).into(),
                ]
                .into_iter()
                .chain(
                    doubled_consonant_inflection("bdgklmnprstz", "able", &["v"], &["adj"])
                        .into_iter()
                        .map(|sr| sr.into()),
                )
                .collect(),
                i18n: None,
            },
        ),
    ]))
});

pub(crate) static EN_TRANSFORM_TESTS: LazyLock<[&TransformTest; 1]> =
    LazyLock::new(|| [&EN_ADJ_TESTS]);

pub(crate) static EN_ADJ_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "funny",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "unfunny",
            rule: "adj",
            reasons: vec!["un-"],
        },
        LanguageTransformerTestCase {
            inner: "funnier",
            rule: "adj",
            reasons: vec!["comparative"],
        },
    ],
});

#[cfg(test)]
pub(crate) mod en_transforms_test {
    use crate::{
        ja::ja_transforms::{has_term_reasons, JP_TRANSFORM_TESTS},
        transformer::LanguageTransformer,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn len() {
        assert_eq!(ENGLISH_TRANSFORMS_DESCRIPTOR.transforms.len(), 17);
        assert_eq!(ENGLISH_TRANSFORMS_DESCRIPTOR.conditions.len(), 7);
    }

    #[test]
    fn transforms() {
        let mut lt = LanguageTransformer::new();
        lt.add_descriptor(&ENGLISH_TRANSFORMS_DESCRIPTOR).unwrap();

        for (i, test) in EN_TRANSFORM_TESTS.iter().enumerate() {
            let term = test.term;
            for case in &test.sources {
                let source = case.inner;
                let rule = case.rule;
                let expected_reasons = &case.reasons;

                let result =
                    has_term_reasons(&lt, source, term, Some(rule), Some(expected_reasons));
                if let Err(e) = result {
                    panic!("Failed: {}", e);
                }
            }
        }
    }
}

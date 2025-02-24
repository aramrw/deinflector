// use std::{collections::HashSet, sync::LazyLock};
//
// use indexmap::IndexMap;
//
// use crate::{
//     transformer::{Condition, ConditionMap, RuleType, SuffixRule, Transform, TransformMap},
//     transforms::{inflection, suffix_inflection},
// };
//
// fn doubled_consonant_inflection(
//     consonants: &'static str,
//     suffix: &'static str,
//     conditions_in: &'static [&'static str],
//     conditions_out: &'static [&'static str],
// ) -> Vec<SuffixRule> {
//     let fmt = |csn: &char| format!("{csn}{csn}{suffix}");
//     let inflections: Vec<SuffixRule> = consonants
//         .chars()
//         .map(|csn| {
//             let cstr = csn.to_string().leak();
//             suffix_inflection(&fmt(&csn), cstr, conditions_in, conditions_out)
//         })
//         .collect();
//     inflections
// }
//
// const PAST_SUFFIX_INFLECTIONS: LazyLock<Vec<SuffixRule>> = LazyLock::new(|| {
//     [
//         suffix_inflection("ed", "", &["v"], &["v"]),   // "walked"
//         suffix_inflection("ed", "e", &["v"], &["v"]),  // "hoped"
//         suffix_inflection("ied", "y", &["v"], &["v"]), // "tried"
//         suffix_inflection("cked", "c", &["v"], &["v"]), // "frolicked"
//         suffix_inflection("laid", "lay", &["v"], &["v"]),
//         suffix_inflection("paid", "pay", &["v"], &["v"]),
//         suffix_inflection("said", "say", &["v"], &["v"]),
//     ]
//     .into_iter()
//     .chain(doubled_consonant_inflection(
//         "bdgklmnprstz",
//         "ed",
//         &["v"],
//         &["v"],
//     ))
//     .collect()
// });
//
// const ING_SUFFIX_INFLECTIONS: LazyLock<Vec<SuffixRule>> = LazyLock::new(|| {
//     [
//         suffix_inflection("ing", "", &["v"], &["v"]), // "walking"
//         suffix_inflection("ing", "e", &["v"], &["v"]), // "driving"
//         suffix_inflection("ying", "ie", &["v"], &["v"]), // "lying"
//         suffix_inflection("cking", "c", &["v"], &["v"]), // "panicking"]
//     ]
//     .into_iter()
//     .chain(doubled_consonant_inflection(
//         "bdgklmnprstz",
//         "ing",
//         &["v"],
//         &["v"],
//     ))
//     .collect()
// });
//
// const THIRD_PERSON_SG_PRESENT_SUFFIX_INFLECTIONS: LazyLock<[SuffixRule; 3]> = LazyLock::new(|| {
//     [
//         suffix_inflection("s", "", &["v"], &["v"]),    // "walks"
//         suffix_inflection("es", "", &["v"], &["v"]),   // "teaches"
//         suffix_inflection("ies", "y", &["v"], &["v"]), // "tries"
//     ]
// });
//
// #[rustfmt::skip]
// const PHRASAL_VERB_PARTICLES: [&str; 57] =
//     ["aboard", "about", "above", "across", "ahead", "alongside", "apart", "around", "aside", "astray", "away", "back", "before", "behind", "below", "beneath", "besides", "between", "beyond", "by", "close", "down", "east", "west", "north", "south", "eastward", "westward", "northward", "southward", "forward", "backward", "backwards", "forwards", "home", "in", "inside", "instead", "near", "off", "on", "opposite", "out", "outside", "over", "overhead", "past", "round", "since", "through", "throughout", "together", "under", "underneath", "up", "within", "without"];
// #[rustfmt::skip]
// const PHRASAL_VERB_PREPOSITIONS: [&str; 50] =  ["aback", "about", "above", "across", "after", "against", "ahead", "along", "among", "apart", "around", "as", "aside", "at", "away", "back", "before", "behind", "below", "between", "beyond", "by", "down", "even", "for", "forth", "forward", "from", "in", "into", "of", "off", "on", "onto", "open", "out", "over", "past", "round", "through", "to", "together", "toward", "towards", "under", "up", "upon", "way", "with", "without"];
//
// const PARTICLES_DISJUNCTION: LazyLock<String> = LazyLock::new(|| PHRASAL_VERB_PARTICLES.join("|"));
// const PHRASAL_VERB_WORD_SET: LazyLock<HashSet<&str>> = LazyLock::new(|| {
//     HashSet::from_iter(
//         PHRASAL_VERB_PARTICLES
//             .into_iter()
//             .chain(PHRASAL_VERB_PREPOSITIONS),
//     )
// });
// const PHRASAL_VERB_WORD_DISJUNCTION: LazyLock<String> = LazyLock::new(|| {
//     PHRASAL_VERB_WORD_SET
//         .iter()
//         .copied()
//         .collect::<Vec<&str>>()
//         .join("|")
// });
//
// static EN_CONDITIONS: LazyLock<ConditionMap> = LazyLock::new(|| {
//     ConditionMap(IndexMap::from([
//         (
//             "v".into(),
//             Condition {
//                 name: "Verb".into(),
//                 is_dictionary_form: true,
//                 sub_conditions: Some(&["v_phr"]),
//                 i18n: None,
//             },
//         ),
//         (
//             "v_phr".into(),
//             Condition {
//                 name: "Phrasal verb".into(),
//                 is_dictionary_form: true,
//                 sub_conditions: None,
//                 i18n: None,
//             },
//         ),
//         (
//             "n".into(),
//             Condition {
//                 name: "Noun".into(),
//                 is_dictionary_form: true,
//                 sub_conditions: Some(&["np", "ns"]),
//                 i18n: None,
//             },
//         ),
//         (
//             "np".into(),
//             Condition {
//                 name: "Noun plural".into(),
//                 is_dictionary_form: true,
//                 sub_conditions: None,
//                 i18n: None,
//             },
//         ),
//         (
//             "ns".into(),
//             Condition {
//                 name: "Noun singular".into(),
//                 is_dictionary_form: true,
//                 sub_conditions: None,
//                 i18n: None,
//             },
//         ),
//         (
//             "adj".into(),
//             Condition {
//                 name: "Adjective".into(),
//                 is_dictionary_form: true,
//                 sub_conditions: None,
//                 i18n: None,
//             },
//         ),
//         (
//             "adv".into(),
//             Condition {
//                 name: "Adverb".into(),
//                 is_dictionary_form: true,
//                 sub_conditions: None,
//                 i18n: None,
//             },
//         ),
//     ]))
// });
//
// static EN_TRANSFORMS: LazyLock<TransformMap> = LazyLock::new(|| {
//     TransformMap(IndexMap::from([
//         (
//             "plural",
//             Transform {
//                 name: "plural",
//                 description: Some("Plural form of a noun"),
//                 rules: vec![suffix_inflection("s", "", &["np"], &["ns"])],
//                 i18n: None,
//             },
//         ),
//         (
//             "possessive",
//             Transform {
//                 name: "possessive",
//                 description: Some("Possessive form of a noun"),
//                 rules: vec![
//                     suffix_inflection("'s", "", &["n"], &["n"]),
//                     suffix_inflection("s'", "s", &["n"], &["n"]),
//                 ],
//                 i18n: None,
//             },
//         ),
//         // Past tense
//         (
//             "past",
//             Transform {
//                 name: "past",
//                 description: Some("Simple past tense of a verb"),
//                 rules: vec![
//                     past_suffix_inflections(),
//                     create_phrasal_verb_inflections_from_suffix_inflections(
//                         past_suffix_inflections(),
//                     ),
//                 ],
//                 i18n: None,
//             },
//         ),
//         // Present participle
//         (
//             "ing",
//             Transform {
//                 name: "ing",
//                 description: Some("Present participle of a verb"),
//                 rules: vec![
//                     ing_suffix_inflections(),
//                     create_phrasal_verb_inflections_from_suffix_inflections(
//                         ing_suffix_inflections(),
//                     ),
//                 ],
//                 i18n: None,
//             },
//         ),
//         // Third person singular present
//         (
//             "3rd pers. sing. pres",
//             Transform {
//                 name: "3rd pers. sing. pres",
//                 description: Some("Third person singular present tense of a verb"),
//                 rules: vec![
//                     third_person_sg_present_suffix_inflections(),
//                     create_phrasal_verb_inflections_from_suffix_inflections(
//                         third_person_sg_present_suffix_inflections(),
//                     ),
//                 ],
//                 i18n: None,
//             },
//         ),
//         // Interposed object
//         (
//             "interposed object",
//             Transform {
//                 name: "interposed object",
//                 description: Some("Phrasal verb with interposed object"),
//                 rules: vec![phrasal_verb_interposed_object_rule()],
//                 i18n: None,
//             },
//         ),
//         // Archaic form
//         (
//             "archaic",
//             Transform {
//                 name: "archaic",
//                 description: Some("Archaic form of a word"),
//                 rules: vec![suffix_inflection("'d", "ed", &["v"], &["v"])],
//                 i18n: None,
//             },
//         ),
//         // Adverb form
//         (
//             "adverb",
//             Transform {
//                 name: "adverb",
//                 description: Some("Adverb form of an adjective"),
//                 rules: vec![
//                     suffix_inflection("ly", "", &["adv"], &["adj"]), // 'quickly'
//                     suffix_inflection("ily", "y", &["adv"], &["adj"]), // 'happily'
//                     suffix_inflection("ly", "le", &["adv"], &["adj"]), // 'humbly'
//                 ],
//                 i18n: None,
//             },
//         ),
//         // Comparative form
//         (
//             "comparative",
//             Transform {
//                 name: "comparative",
//                 description: Some("Comparative form of an adjective"),
//                 i18n: None,
//                 rules: vec![
//                     suffix_inflection("er", "", &["adj"], &["adj"]), // 'faster'
//                     suffix_inflection("er", "e", &["adj"], &["adj"]), // 'nicer'
//                     suffix_inflection("ier", "y", &["adj"], &["adj"]), // 'happier'
//                 ]
//                 .into_iter()
//                 .chain(doubled_consonant_inflection(
//                     "bdgmnt",
//                     "er",
//                     &["adj"],
//                     &["adj"],
//                 ))
//                 .collect(),
//             },
//         ),
//         // Superlative form
//         (
//             "superlative",
//             Transform {
//                 name: "superlative",
//                 description: Some("Superlative form of an adjective"),
//                 rules: vec![
//                     suffix_inflection("est", "", &["adj"], &["adj"]), // 'fastest'
//                     suffix_inflection("est", "e", &["adj"], &["adj"]), // 'nicest'
//                     suffix_inflection("iest", "y", &["adj"], &["adj"]), // 'happiest
//                 ]
//                 .into_iter()
//                 .chain(doubled_consonant_inflection(
//                     "bdgmnt",
//                     "est",
//                     &["adj"],
//                     &["adj"],
//                 ))
//                 .collect(),
//                 i18n: None,
//             },
//         ),
//         // Dropped g
//         (
//             "dropped g",
//             Transform {
//                 name: "dropped g",
//                 description: Some("Dropped g in -ing form of a verb"),
//                 rules: vec![suffix_inflection("in'", "ing", &["v"], &["v"])],
//                 i18n: None,
//             },
//         ),
//         // -y form
//         (
//             "-y",
//             Transform {
//                 name: "-y",
//                 description: Some("Adjective formed from a verb or noun"),
//                 rules: vec![
//                     suffix_inflection("y", "", &["adj"], &["n", "v"]), // 'dirty', 'pushy'
//                     suffix_inflection("y", "e", &["adj"], &["n", "v"]), // 'hazy'
//                 ]
//                 .into_iter()
//                 .chain(doubled_consonant_inflection(
//                     "glmnprst",
//                     "y",
//                     &[],
//                     &["n", "v"],
//                 ))
//                 .collect(),
//                 i18n: None,
//             },
//         ),
//         // un- prefix
//         (
//             "un-",
//             Transform {
//                 name: "un-",
//                 description: Some("Negative form of an adjective, adverb, or verb"),
//                 rules: vec![prefix_inflection(
//                     "un",
//                     "",
//                     &["adj", "adv", "v"],
//                     &["adj", "adv", "v"],
//                 )],
//                 i18n: None,
//             },
//         ),
//         // going-to future
//         (
//             "going-to future",
//             Transform {
//                 name: "going-to future",
//                 description: Some("Going-to future tense of a verb"),
//                 rules: vec![inflection(
//                     "going to ",
//                     "",
//                     &["v"],
//                     &["v"],
//                     RuleType::Prefix,
//                 )],
//                 i18n: None,
//             },
//         ),
//         // will future
//         (
//             "will future",
//             Transform {
//                 name: "will future",
//                 description: Some("Will-future tense of a verb"),
//                 rules: vec![prefix_inflection("will ", "", &["v"], &["v"])],
//                 i18n: None,
//             },
//         ),
//         // imperative negative
//         (
//             "imperative negative",
//             Transform {
//                 name: "imperative negative",
//                 description: Some("Negative imperative form of a verb"),
//                 rules: vec![
//                     prefix_inflection("don't ", "", &["v"], &["v"]),
//                     prefix_inflection("do not ", "", &["v"], &["v"]),
//                 ],
//                 i18n: None,
//             },
//         ),
//         // -able suffix
//         (
//             "-able",
//             Transform {
//                 name: "-able",
//                 description: Some("Adjective formed from a verb"),
//                 rules: vec![
//                     suffix_inflection("able", "", &["v"], &["adj"]),
//                     suffix_inflection("able", "e", &["v"], &["adj"]),
//                     suffix_inflection("iable", "y", &["v"], &["adj"]),
//                 ]
//                 .into_iter()
//                 .chain(doubled_consonant_inflection(
//                     "bdgklmnprstz",
//                     "able",
//                     &["v"],
//                     &["adj"],
//                 ))
//                 .collect(),
//                 i18n: None,
//             },
//         ),
//     ]))
// });

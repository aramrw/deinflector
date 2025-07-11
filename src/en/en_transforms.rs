use std::{
    collections::HashSet,
    ops::Deref,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use fancy_regex::Regex;
use indexmap::{IndexMap, IndexSet};

use crate::{
    ja::ja_transforms::{LanguageTransformerTestCase, TransformTest},
    transformer::{
        Condition, ConditionMap, DeinflectFnType, LanguageTransformDescriptor, Rule,
        RuleDeinflectFnTrait, RuleType, SuffixRule, Transform, TransformMap,
    },
    transforms::inflection,
};

fn doubled_consonant_inflection<'a: 'static>(
    consonants: &'a str,
    suffix: &'a str,
    conditions_in: &'a [&'a str],
    conditions_out: &'a [&'a str],
) -> Vec<SuffixRule> {
    let fmt = |csn: &char| format!("{csn}{csn}{suffix}");
    let inflections: Vec<SuffixRule> = consonants
        .chars()
        .map(|csn| {
            let cstr = csn.to_string().leak();
            inflection(
                &fmt(&csn),
                cstr,
                conditions_in,
                conditions_out,
                RuleType::Suffix,
            )
            .into()
        })
        .collect();
    inflections
}

#[test]
fn double_consonant_inflection() {
    use pretty_assertions::assert_eq as passert_eq;
    let expected: &[Rule] = &[
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("bbing$").unwrap(),
            inflected_str: Some("bbing".to_string()),
            deinflected: Some("b"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("dding$").unwrap(),
            inflected_str: Some("dding".to_string()),
            deinflected: Some("d"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("gging$").unwrap(),
            inflected_str: Some("gging".to_string()),
            deinflected: Some("g"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("kking$").unwrap(),
            inflected_str: Some("kking".to_string()),
            deinflected: Some("k"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("lling$").unwrap(),
            inflected_str: Some("lling".to_string()),
            deinflected: Some("l"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("mming$").unwrap(),
            inflected_str: Some("mming".to_string()),
            deinflected: Some("m"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("nning$").unwrap(),
            inflected_str: Some("nning".to_string()),
            deinflected: Some("n"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("pping$").unwrap(),
            inflected_str: Some("pping".to_string()),
            deinflected: Some("p"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("rring$").unwrap(),
            inflected_str: Some("rring".to_string()),
            deinflected: Some("r"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("ssing$").unwrap(),
            inflected_str: Some("ssing".to_string()),
            deinflected: Some("s"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("tting$").unwrap(),
            inflected_str: Some("tting".to_string()),
            deinflected: Some("t"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        Rule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("zzing$").unwrap(),
            inflected_str: Some("zzing".to_string()),
            deinflected: Some("z"),
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
    ];
    let result: Vec<Rule> = doubled_consonant_inflection("bdgklmnprstz", "ing", &["v"], &["v"])
        .into_iter()
        .map(|sr| sr.into())
        .collect();
    passert_eq!(result, expected);
}

pub static PAST_SUFFIX_INFLECTIONS: LazyLock<Vec<SuffixRule>> = LazyLock::new(|| {
    [
        inflection("ed", "", &["v"], &["v"], RuleType::Suffix).into(), // "walked"
        inflection("ed", "e", &["v"], &["v"], RuleType::Suffix).into(), // "hoped"
        inflection("ied", "y", &["v"], &["v"], RuleType::Suffix).into(), // "tried"
        inflection("cked", "c", &["v"], &["v"], RuleType::Suffix).into(), // "frolicked"
    ]
    .into_iter()
    .chain(doubled_consonant_inflection(
        "bdgklmnprstz",
        "ed",
        &["v"],
        &["v"],
    ))
    .chain([
        inflection("laid", "lay", &["v"], &["v"], RuleType::Suffix).into(),
        inflection("paid", "pay", &["v"], &["v"], RuleType::Suffix).into(),
        inflection("said", "say", &["v"], &["v"], RuleType::Suffix).into(),
    ])
    .collect()
});

/// ["walking", "driving", "lying", "panicking"]
pub static ING_SUFFIX_INFLECTIONS: LazyLock<Vec<SuffixRule>> = LazyLock::new(|| {
    [
        inflection("ing", "", &["v"], &["v"], RuleType::Suffix).into(),
        inflection("ing", "e", &["v"], &["v"], RuleType::Suffix).into(),
        inflection("ying", "ie", &["v"], &["v"], RuleType::Suffix).into(),
        inflection("cking", "c", &["v"], &["v"], RuleType::Suffix).into(),
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

#[test]
fn ing_suffix_inflections() {
    use pretty_assertions::assert_eq as passert_eq;
    let expected: &[SuffixRule] = &[
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("ing$").unwrap(),
            inflected_str: Some("ing".to_string()), // Added
            deinflected: "",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("ing$").unwrap(),
            inflected_str: Some("ing".to_string()), // Added
            deinflected: "e",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("ying$").unwrap(),
            inflected_str: Some("ying".to_string()), // Added
            deinflected: "ie",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("cking$").unwrap(),
            inflected_str: Some("cking".to_string()), // Added
            deinflected: "c",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("bbing$").unwrap(),
            inflected_str: Some("bbing".to_string()), // Added
            deinflected: "b",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("dding$").unwrap(),
            inflected_str: Some("dding".to_string()), // Added
            deinflected: "d",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("gging$").unwrap(),
            inflected_str: Some("gging".to_string()), // Added
            deinflected: "g",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("kking$").unwrap(),
            inflected_str: Some("kking".to_string()), // Added
            deinflected: "k",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("lling$").unwrap(),
            inflected_str: Some("lling".to_string()), // Added
            deinflected: "l",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("mming$").unwrap(),
            inflected_str: Some("mming".to_string()), // Added
            deinflected: "m",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("nning$").unwrap(),
            inflected_str: Some("nning".to_string()), // Added
            deinflected: "n",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("pping$").unwrap(),
            inflected_str: Some("pping".to_string()), // Added
            deinflected: "p",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("rring$").unwrap(),
            inflected_str: Some("rring".to_string()), // Added
            deinflected: "r",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("ssing$").unwrap(),
            inflected_str: Some("ssing".to_string()), // Added
            deinflected: "s",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("tting$").unwrap(),
            inflected_str: Some("tting".to_string()), // Added
            deinflected: "t",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
        SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("zzing$").unwrap(),
            inflected_str: Some("zzing".to_string()), // Added
            deinflected: "z",
            deinflect_fn: DeinflectFnType::GenericSuffix,
            conditions_in: &["v"],
            conditions_out: &["v"],
        },
    ];

    passert_eq!(&*ING_SUFFIX_INFLECTIONS, expected);

    let deinflect_txt_expected = [
        "going to w",
        "going to we",
        "going to ie",
        "going toc",
        "going tob",
        "going tod",
        "going tog",
        "going tok",
        "going tol",
        "going tom",
        "going ton",
        "going top",
        "going tor",
        "going tos",
        "going tot",
        "going toz",
    ]
    .to_vec();
    let results = ING_SUFFIX_INFLECTIONS
        .iter()
        .map(|rule| rule.deinflect("going to walk"))
        .collect::<Vec<String>>();
    passert_eq!(deinflect_txt_expected, results);
}

/// ["walks", "teaches", "tries"]
pub static THIRD_PERSON_SG_PRESENT_SUFFIX_INFLECTIONS: LazyLock<[SuffixRule; 3]> =
    LazyLock::new(|| {
        [
            inflection("s", "", &["v"], &["v"], RuleType::Suffix).into(),
            inflection("es", "", &["v"], &["v"], RuleType::Suffix).into(),
            inflection("ies", "y", &["v"], &["v"], RuleType::Suffix).into(),
        ]
    });

#[rustfmt::skip]
const PHRASAL_VERB_PARTICLES: [&str; 57] =
    ["aboard", "about", "above", "across", "ahead", "alongside", "apart", "around", "aside", "astray", "away", "back", "before", "behind", "below", "beneath", "besides", "between", "beyond", "by", "close", "down", "east", "west", "north", "south", "eastward", "westward", "northward", "southward", "forward", "backward", "backwards", "forwards", "home", "in", "inside", "instead", "near", "off", "on", "opposite", "out", "outside", "over", "overhead", "past", "round", "since", "through", "throughout", "together", "under", "underneath", "up", "within", "without"];
#[rustfmt::skip]
pub const PHRASAL_VERB_PREPOSITIONS: [&str; 50] =  ["aback", "about", "above", "across", "after", "against", "ahead", "along", "among", "apart", "around", "as", "aside", "at", "away", "back", "before", "behind", "below", "between", "beyond", "by", "down", "even", "for", "forth", "forward", "from", "in", "into", "of", "off", "on", "onto", "open", "out", "over", "past", "round", "through", "to", "together", "toward", "towards", "under", "up", "upon", "way", "with", "without"];

// the ordering of words in your disjunction isn’t guaranteed
pub static PARTICLES_DISJUNCTION: LazyLock<String> =
    LazyLock::new(|| PHRASAL_VERB_PARTICLES.join("|"));
pub static PHRASAL_VERB_WORD_SET: LazyLock<IndexSet<&str>> = LazyLock::new(|| {
    IndexSet::from_iter(
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

/// https://github.com/yomidevs/yomitan/blob/4427bcf3c2de3d294b9a82aac2f97d6e72c2706c/ext/js/language/en/english-transforms.js#L72
pub static PHRASAL_VERB_INTERPOSED_OBJECT_RULE: LazyLock<Rule> = LazyLock::new(|| Rule {
    rule_type: RuleType::Other,
    is_inflected: fancy_regex::Regex::new(&format!(
        r"^\w* (?:(?!\b({})\b).)+ (?:{})",
        &*PHRASAL_VERB_WORD_DISJUNCTION, &*PARTICLES_DISJUNCTION
    ))
    .unwrap(),
    inflected_str: None,
    // deinflected is not necessary for this fn
    deinflected: None,
    deinflect_fn: DeinflectFnType::EnPhrasalVerbInterposedObjectRule,
    conditions_in: &[],
    conditions_out: &["v_phr"],
});

#[test]
fn test_phrasal_verb_interposed_object_rule() {
    use pretty_assertions::assert_eq as passert_eq;
    let expected = Rule {
        rule_type: RuleType::Other,
        is_inflected: Regex::from_str(
            r"^\w* (?:(?!\b(aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)\b).)+ (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without)"
        ).unwrap(),
        inflected_str: None,
        deinflected: None,
        deinflect_fn: DeinflectFnType::EnPhrasalVerbInterposedObjectRule,
        conditions_in: &[],
        conditions_out: &["v_phr"],
    };
    let result = PHRASAL_VERB_INTERPOSED_OBJECT_RULE.deref();
    passert_eq!(*result, expected);
    // no change happens in javascript as well
    let result_txt = expected.deinflect("going to walk");
    let expected_txt = "going to walk";
    passert_eq!(result_txt, expected_txt);
}

/// has deinflect_fn type of: [`DeinflectFnType::EnCreatePhrasalVerbInflection`]
/// only used in english
fn create_phrasal_verb_inflection(inflected: String, deinflected: &'static str) -> Rule {
    let is_inflected = Regex::new(&format!(
        r"^\w*{} (?:{})",
        inflected, &*PHRASAL_VERB_WORD_DISJUNCTION
    ))
    .unwrap();
    Rule {
        rule_type: RuleType::Other,
        is_inflected,
        inflected_str: Some(inflected),
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
            // remove trailing '$' from is_inflected
            let inflected_suffix = sr.is_inflected.as_str().replace('$', "");
            let deinflected_suffix = sr.deinflected;
            // create verb inflection based on suffixes
            vec![create_phrasal_verb_inflection(
                inflected_suffix,
                deinflected_suffix,
            )]
        })
        .collect()
}

#[test]
fn test_create_phrasal_verb_inflections_from_suffix_inflections() {
    let tests = vec![
        Rule {
            rule_type: RuleType::Other,
            is_inflected: Regex::new(r"^\w*ed (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("ed".to_string()),
            deinflected: Some(""),
            deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
            conditions_in: &["v"],
            conditions_out: &["v_phr"],
        },
        Rule {
            rule_type: RuleType::Other,
            is_inflected: Regex::new(r"^\w*ed (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("ed".to_string()),
            deinflected: Some("e"),
            deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
            conditions_in: &["v"],
            conditions_out: &["v_phr"],
        },
        Rule {
            rule_type: RuleType::Other,
            is_inflected: Regex::new(r"^\w*ied (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("ied".to_string()),
            deinflected: Some("y"),
            deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
            conditions_in: &["v"],
            conditions_out: &["v_phr"],
        },
        Rule {
            rule_type: RuleType::Other,
            is_inflected: Regex::new(r"^\w*cked (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("cked".to_string()),
            deinflected: Some("c"),
            deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
            conditions_in: &["v"],
            conditions_out: &["v_phr"],
        },
        Rule {
            rule_type: RuleType::Other,
            is_inflected: Regex::new(r"^\w*bbed (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("bbed".to_string()),
            deinflected: Some("b"),
            deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
            conditions_in: &["v"],
            conditions_out: &["v_phr"],
        },
        Rule {
            rule_type: RuleType::Other,
            is_inflected: Regex::new(r"^\w*dded (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("dded".to_string()),
            deinflected: Some("d"),
            deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
            conditions_in: &["v"],
            conditions_out: &["v_phr"],
        },
        Rule {
            rule_type: RuleType::Other,
            is_inflected: Regex::new(r"^\w*gged (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("gged".to_string()),
            deinflected: Some("g"),
            deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
            conditions_in: &["v"],
            conditions_out: &["v_phr"],
},
        Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*kked (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("kked".to_string()),
    deinflected: Some("k"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*lled (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("lled".to_string()),
    deinflected: Some("l"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*mmed (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("mmed".to_string()),
    deinflected: Some("m"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*nned (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("nned".to_string()),
    deinflected: Some("n"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*pped (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("pped".to_string()),
    deinflected: Some("p"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*rred (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("rred".to_string()),
    deinflected: Some("r"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*ssed (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("ssed".to_string()),
    deinflected: Some("s"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*tted (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("tted".to_string()),
    deinflected: Some("t"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*zzed (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("zzed".to_string()),
    deinflected: Some("z"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*laid (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("laid".to_string()),
    deinflected: Some("lay"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*paid (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("paid".to_string()),
    deinflected: Some("pay"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
Rule {
    rule_type: RuleType::Other,
    is_inflected: Regex::new(r"^\w*said (?:aboard|about|above|across|ahead|alongside|apart|around|aside|astray|away|back|before|behind|below|beneath|besides|between|beyond|by|close|down|east|west|north|south|eastward|westward|northward|southward|forward|backward|backwards|forwards|home|in|inside|instead|near|off|on|opposite|out|outside|over|overhead|past|round|since|through|throughout|together|under|underneath|up|within|without|aback|after|against|along|among|as|at|even|for|forth|from|into|of|onto|open|to|toward|towards|upon|way|with)").unwrap(),
            inflected_str: Some("said".to_string()),
    deinflected: Some("say"),
    deinflect_fn: DeinflectFnType::EnCreatePhrasalVerbInflection,
    conditions_in: &["v"],
    conditions_out: &["v_phr"],
},
    ];
    let res = create_phrasal_verb_inflections_from_suffix_inflections(&PAST_SUFFIX_INFLECTIONS);
    assert_eq!(res.len(), tests.len(), "create_phrasal_verb_inflections_from_suffix_inflections resulting Vec<Rule> length didn't match the tests.");
    for (i, test) in tests.iter().enumerate() {
        assert_eq!(res.get(i).unwrap(), test, "failed on rule[{i}]");
    }
}

pub static ENGLISH_TRANSFORMS_DESCRIPTOR: LazyLock<LanguageTransformDescriptor> =
    LazyLock::new(|| LanguageTransformDescriptor {
        language: "en",
        conditions: &EN_CONDITIONS_MAP,
        transforms: &EN_TRANSFORMS_MAP,
    });

pub static EN_CONDITIONS_MAP: LazyLock<ConditionMap> = LazyLock::new(|| {
    ConditionMap(IndexMap::from([
        (
            "v",
            Condition {
                name: "Verb",
                is_dictionary_form: true,
                sub_conditions: Some(&["v_phr"]),
                i18n: None,
            },
        ),
        (
            "v_phr",
            Condition {
                name: "Phrasal verb",
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "n",
            Condition {
                name: "Noun",
                is_dictionary_form: true,
                sub_conditions: Some(&["np", "ns"]),
                i18n: None,
            },
        ),
        (
            "np",
            Condition {
                name: "Noun plural",
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "ns",
            Condition {
                name: "Noun singular",
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "adj",
            Condition {
                name: "Adjective",
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "adv",
            Condition {
                name: "Adverb",
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
                rules: vec![
                    inflection("s", "", &["np"], &["ns"], RuleType::Suffix),
                    inflection("es", "", &["np"], &["ns"], RuleType::Suffix),
                    inflection("ies", "y", &["np"], &["ns"], RuleType::Suffix),
                    inflection("ves", "fe", &["np"], &["ns"], RuleType::Suffix),
                    inflection("ves", "f", &["np"], &["ns"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
        (
            "possessive",
            Transform {
                name: "possessive",
                description: Some("Possessive form of a noun"),
                rules: vec![
                    inflection("'s", "", &["n"], &["n"], RuleType::Suffix),
                    inflection("s'", "s", &["n"], &["n"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
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
        (
            "interposed object",
            Transform {
                name: "interposed object",
                description: Some("Phrasal verb with interposed object"),
                rules: vec![PHRASAL_VERB_INTERPOSED_OBJECT_RULE.clone()],
                i18n: None,
            },
        ),
        (
            "archaic",
            Transform {
                name: "archaic",
                description: Some("Archaic form of a word"),
                rules: vec![inflection("'d", "ed", &["v"], &["v"], RuleType::Suffix)],
                i18n: None,
            },
        ),
        (
            "adverb",
            Transform {
                name: "adverb",
                description: Some("Adverb form of an adjective"),
                rules: vec![
                    inflection("ly", "", &["adv"], &["adj"], RuleType::Suffix),
                    inflection("ily", "y", &["adv"], &["adj"], RuleType::Suffix),
                    inflection("ly", "le", &["adv"], &["adj"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
        (
            "comparative",
            Transform {
                name: "comparative",
                description: Some("Comparative form of an adjective"),
                i18n: None,
                rules: vec![
                    inflection("er", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("er", "e", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("ier", "y", &["adj"], &["adj"], RuleType::Suffix),
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
        (
            "superlative",
            Transform {
                name: "superlative",
                description: Some("Superlative form of an adjective"),
                rules: vec![
                    inflection("est", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("est", "e", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("iest", "y", &["adj"], &["adj"], RuleType::Suffix),
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
        (
            "dropped g",
            Transform {
                name: "dropped g",
                description: Some("Dropped g in -ing form of a verb"),
                rules: vec![inflection("in'", "ing", &["v"], &["v"], RuleType::Suffix)],
                i18n: None,
            },
        ),
        (
            "-y",
            Transform {
                name: "-y",
                description: Some("Adjective formed from a verb or noun"),
                rules: vec![
                    inflection("y", "", &["adj"], &["n", "v"], RuleType::Suffix),
                    inflection("y", "e", &["adj"], &["n", "v"], RuleType::Suffix),
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
        (
            "will future",
            Transform {
                name: "will future",
                description: Some("Will-future tense of a verb"),
                rules: vec![inflection("will ", "", &["v"], &["v"], RuleType::Prefix)],
                i18n: None,
            },
        ),
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
        (
            "-able",
            Transform {
                name: "-able",
                description: Some("Adjective formed from a verb"),
                rules: vec![
                    inflection("able", "", &["v"], &["adj"], RuleType::Suffix),
                    inflection("able", "e", &["v"], &["adj"], RuleType::Suffix),
                    inflection("iable", "y", &["v"], &["adj"], RuleType::Suffix),
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
    LazyLock::new(|| [&EN_VERB_TESTS]);

pub(crate) static EN_VERB_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "walk",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "walked",
            rule: "v",
            reasons: vec!["past"],
        },
        LanguageTransformerTestCase {
            inner: "going to walk",
            rule: "v",
            reasons: vec!["going-to future"],
        },
        LanguageTransformerTestCase {
            inner: "will walk",
            rule: "v",
            reasons: vec!["will future"],
        },
        LanguageTransformerTestCase {
            inner: "don't walk",
            rule: "v",
            reasons: vec!["imperative negative"],
        },
        LanguageTransformerTestCase {
            inner: "do not walk",
            rule: "v",
            reasons: vec!["imperative negative"],
        },
    ],
});

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

/// cargo test en_transforms_test
#[cfg(test)]
pub(crate) mod entransforms {
    use crate::{
        ja::ja_transforms::{has_term_reasons, JP_TRANSFORM_TESTS},
        transformer::{LanguageTransformer, TraceFrame, TransformedText},
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn len() {
        assert_eq!(ENGLISH_TRANSFORMS_DESCRIPTOR.transforms.len(), 17);
        assert_eq!(ENGLISH_TRANSFORMS_DESCRIPTOR.conditions.len(), 7);
        //dbg!(ENGLISH_TRANSFORMS_DESCRIPTOR.transforms);
    }

    #[test]
    fn transform() {
        let mut lt = LanguageTransformer::new();
        lt.add_descriptor(&ENGLISH_TRANSFORMS_DESCRIPTOR).unwrap();

        let expected = vec![
            TransformedText {
                text: "going to walk".into(),
                conditions: 0,
                trace: vec![],
            },
            TransformedText {
                text: "go to walk".into(),
                conditions: 1,
                trace: vec![TraceFrame {
                    transform: "ing".into(),
                    rule_index: 16,
                    text: "going to walk".into(),
                }],
            },
            TransformedText {
                text: "goe to walk".into(),
                conditions: 1,
                trace: vec![TraceFrame {
                    transform: "ing".into(),
                    rule_index: 17,
                    text: "going to walk".into(),
                }],
            },
            TransformedText {
                text: "walk".into(),
                conditions: 1,
                trace: vec![TraceFrame {
                    transform: "going-to future".into(),
                    rule_index: 0,
                    text: "going to walk".into(),
                }],
            },
        ];
        let res = lt.transform("going to walk");
        assert_eq!(res, expected)
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
                    panic!("Failed: {e}");
                }
            }
        }
    }
}

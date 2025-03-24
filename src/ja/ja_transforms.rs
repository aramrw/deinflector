use std::sync::LazyLock;

use regex::Regex;

use crate::{
    transformer::{
        Condition, ConditionMap, LanguageTransformer, Rule, RuleI18n, RuleType, SuffixRule,
    },
    transforms::suffix_inflection,
};

// impl From<Vec<SuffixRule>> for Vec<Rule> {
//     fn from(value: Vec<SuffixRule>) -> Self {
//         value.into_iter().map(|v| v.into()).collect::<Vec<Rule>>()
//     }
// }

#[derive(Debug, thiserror::Error)]
enum TransformTestError<'a> {
    #[error("{term} should have term candidate {src} with rule {rule} with reasons {reasons:?}")]
    MissingTransformation {
        src: &'static str,
        term: &'static str,
        rule: &'static str,
        reasons: &'a [&'static str],
    },
}

pub(crate) struct TransformTest {
    pub(crate) term: &'static str,
    pub(crate) sources: Vec<LanguageTransformerTestCase>,
}

pub(crate) struct LanguageTransformerTestCase {
    inner: &'static str,
    rule: &'static str,
    reasons: Vec<&'static str>,
}

#[derive(Debug)]
pub(crate) struct HasTermReasons {
    pub(crate) reasons: Vec<String>,
    pub(crate) rules: usize,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum HasTermReasonsError {
    #[error("No transformation from '{src}' to '{term}' with rule '{rule}'.\nRejected candidates:\n{}", .rejected.join("\n"))]
    NoMatch {
        src: String,
        term: String,
        rule: String,
        rejected: Vec<String>,
    },
    #[error("Trace length mismatch: expected {expected}, found {found}")]
    TraceLengthMismatch { expected: usize, found: usize },
    #[error("Reason {index}: expected '{expected}', found '{found}'")]
    ReasonMismatch {
        index: usize,
        expected: String,
        found: String,
    },
}

#[derive(Debug)]
pub(crate) enum IrregularVerbSuffix {
    て,
    た,
    たら,
    たり,
}

impl std::fmt::Display for IrregularVerbSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub fn irregular_verb_suffix_inflections(
    suffix: IrregularVerbSuffix,
    conditions_in: &'static [&'static str],
    conditions_out: &'static [&'static str],
) -> Vec<SuffixRule> {
    let suffix_str = suffix.to_string();

    let iku_inflections = IKU_VERBS.iter().map(|verb| {
        let first_char = verb.chars().next().unwrap();
        let transformed: &'static str = format!("{}っ{}", first_char, suffix_str).leak();
        suffix_inflection(transformed, verb, conditions_in, conditions_out)
    });

    let godan_inflections = GODAN_U_SPECIAL_VERBS.iter().map(|verb| {
        let transformed: &'static str = format!("{}{}", verb, suffix_str).leak();
        suffix_inflection(transformed, verb, conditions_in, conditions_out)
    });

    let fu_inflections = FU_VERB_TE_CONJUGATIONS.iter().map(|[verb, te_root]| {
        let transformed: &'static str = format!("{}{}", te_root, suffix_str).leak();
        suffix_inflection(transformed, verb, conditions_in, conditions_out)
    });

    iku_inflections
        .chain(godan_inflections)
        .chain(fu_inflections)
        .collect()
}

#[cfg(test)]
mod inflection_tests {

    use pretty_assertions::assert_eq;
    use regex::Regex;

    use crate::{
        ja::ja_transforms::{irregular_verb_suffix_inflections, suffix_inflection},
        transformer::{RuleType, SuffixRule, SuffixRuleDeinflectFnTrait},
    };

    //#[test]
    // pub fn irregular_verb_suffix() {
    //     #[rustfmt::skip]
    //     let te_test = [SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("いって$").unwrap(), deinflected: "いく", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("行って$").unwrap(), deinflected: "行く", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("逝って$").unwrap(), deinflected: "逝く", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("往って$").unwrap(), deinflected: "往く", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("こうて$").unwrap(), deinflected: "こう", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("とうて$").unwrap(), deinflected: "とう", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("請うて$").unwrap(), deinflected: "請う", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("乞うて$").unwrap(), deinflected: "乞う", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("恋うて$").unwrap(), deinflected: "恋う", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("問うて$").unwrap(), deinflected: "問う", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("訪うて$").unwrap(), deinflected: "訪う", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("宣うて$").unwrap(), deinflected: "宣う", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("曰うて$").unwrap(), deinflected: "曰う", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("給うて$").unwrap(), deinflected: "給う", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("賜うて$").unwrap(), deinflected: "賜う", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("揺蕩うて$").unwrap(), deinflected: "揺蕩う", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("のたもうて$").unwrap(), deinflected: "のたまう", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("たもうて$").unwrap(), deinflected: "たまう", conditions_in: &["-て"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("たゆとうて$").unwrap(), deinflected: "たゆたう", conditions_in: &["-て"], conditions_out: &["v5"] }];
    //     #[rustfmt::skip]
    //     let ta_test = [SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("いった$").unwrap(), deinflected: "いく", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("行った$").unwrap(), deinflected: "行く", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("逝った$").unwrap(), deinflected: "逝く", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("往った$").unwrap(), deinflected: "往く", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("こうた$").unwrap(), deinflected: "こう", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("とうた$").unwrap(), deinflected: "とう", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("請うた$").unwrap(), deinflected: "請う", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("乞うた$").unwrap(), deinflected: "乞う", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("恋うた$").unwrap(), deinflected: "恋う", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("問うた$").unwrap(), deinflected: "問う", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("訪うた$").unwrap(), deinflected: "訪う", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("宣うた$").unwrap(), deinflected: "宣う", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("曰うた$").unwrap(), deinflected: "曰う", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("給うた$").unwrap(), deinflected: "給う", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("賜うた$").unwrap(), deinflected: "賜う", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("揺蕩うた$").unwrap(), deinflected: "揺蕩う", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("のたもうた$").unwrap(), deinflected: "のたまう", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("たもうた$").unwrap(), deinflected: "たまう", conditions_in: &["-た"], conditions_out: &["v5"] }, SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("たゆとうた$").unwrap(), deinflected: "たゆたう", conditions_in: &["-た"], conditions_out: &["v5"] }];
    //     let て =
    //         irregular_verb_suffix_inflections(super::IrregularVerbSuffix::て, &["-て"], &["v5"]);
    //     assert_eq!(て, te_test);
    //     let た =
    //         irregular_verb_suffix_inflections(super::IrregularVerbSuffix::た, &["-た"], &["v5"]);
    //     assert_eq!(た, ta_test);
    // }

    #[test]
    pub fn suffix() {
        let test = SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("ければ$").unwrap(),
            deinflected: "い",
            deinflect_fn: crate::transformer::DeinflectFnType::GenericSuffix,
            conditions_in: &["-ば"],
            conditions_out: &["adj-i"],
        };
        let sr = suffix_inflection("ければ", "い", &["-ば"], &["adj-i"]);
        assert_eq!(sr, test);
        assert_eq!(sr.deinflect("食べれば"), test.deinflect("食べれば"));
    }
}

pub(crate) static TRANSFORM_TESTS: LazyLock<[&TransformTest; 14]> = LazyLock::new(|| {
    [
        &*JP_ADJ_TESTS,
        &*JP_ICHIDAN_VERB_TESTS,
        &*JP_VERB_U_TESTS,
        &*JP_VERB_KU_TESTS,
        &*JP_VERB_GU_TESTS,
        &*JP_VERB_SU_TESTS,
        &*JP_VERB_TSU_TESTS,
        &*JP_VERB_NU_TESTS,
        &*JP_VERB_BU_TESTS,
        &*JP_VERB_MU_TESTS,
        &*JP_IRREGULAR_VERB_SURU_TESTS,
        &*JP_IRREGULAR_VERB_KURU_TESTS,
        &*JP_ZURU_VERB_TESTS,
        &*JP_EE_ENDING_TESTS,
    ]
});
pub(crate) static JP_ADJ_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "愛しい",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "愛しそう",
            rule: "adj-i",
            reasons: vec!["-そう"],
        },
        LanguageTransformerTestCase {
            inner: "愛しすぎる",
            rule: "adj-i",
            reasons: vec!["-すぎる"],
        },
        LanguageTransformerTestCase {
            inner: "愛し過ぎる",
            rule: "adj-i",
            reasons: vec!["-過ぎる"],
        },
        LanguageTransformerTestCase {
            inner: "愛しかったら",
            rule: "adj-i",
            reasons: vec!["-たら"],
        },
        LanguageTransformerTestCase {
            inner: "愛しかったり",
            rule: "adj-i",
            reasons: vec!["-たり"],
        },
        LanguageTransformerTestCase {
            inner: "愛しくて",
            rule: "adj-i",
            reasons: vec!["-て"],
        },
        LanguageTransformerTestCase {
            inner: "愛しく",
            rule: "adj-i",
            reasons: vec!["-く"],
        },
        LanguageTransformerTestCase {
            inner: "愛しくない",
            rule: "adj-i",
            reasons: vec!["negative"],
        },
        LanguageTransformerTestCase {
            inner: "愛しさ",
            rule: "adj-i",
            reasons: vec!["-さ"],
        },
        LanguageTransformerTestCase {
            inner: "愛しかった",
            rule: "adj-i",
            reasons: vec!["-た"],
        },
        LanguageTransformerTestCase {
            inner: "愛しくありません",
            rule: "adj-i",
            reasons: vec!["-ます", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "愛しくありませんでした",
            rule: "adj-i",
            reasons: vec!["-ます", "negative", "-た"],
        },
        LanguageTransformerTestCase {
            inner: "愛しき",
            rule: "adj-i",
            reasons: vec!["-き"],
        },
        LanguageTransformerTestCase {
            inner: "愛しげ",
            rule: "adj-i",
            reasons: vec!["-げ"],
        },
        LanguageTransformerTestCase {
            inner: "愛し気",
            rule: "adj-i",
            reasons: vec!["-げ"],
        },
        LanguageTransformerTestCase {
            inner: "愛しがる",
            rule: "adj-i",
            reasons: vec!["-がる"],
        },
    ],
});
pub(crate) static JP_VERB_U_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "買う",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "買う",
            rule: "v5",
            reasons: vec![],
        },
        LanguageTransformerTestCase {
            inner: "買います",
            rule: "v5",
            reasons: vec!["-ます"],
        },
        LanguageTransformerTestCase {
            inner: "買った",
            rule: "v5",
            reasons: vec!["-た"],
        },
        LanguageTransformerTestCase {
            inner: "買いました",
            rule: "v5",
            reasons: vec!["-ます", "-た"],
        },
        LanguageTransformerTestCase {
            inner: "買って",
            rule: "v5",
            reasons: vec!["-て"],
        },
        LanguageTransformerTestCase {
            inner: "買える",
            rule: "v5",
            reasons: vec!["potential"],
        },
        LanguageTransformerTestCase {
            inner: "買われる",
            rule: "v5",
            reasons: vec!["passive"],
        },
        LanguageTransformerTestCase {
            inner: "買わせる",
            rule: "v5",
            reasons: vec!["causative"],
        },
        LanguageTransformerTestCase {
            inner: "買わす",
            rule: "v5",
            reasons: vec!["short causative"],
        },
        LanguageTransformerTestCase {
            inner: "買わします",
            rule: "v5",
            reasons: vec!["short causative", "-ます"],
        },
        LanguageTransformerTestCase {
            inner: "買わせられる",
            rule: "v5",
            reasons: vec!["causative", "potential or passive"],
        },
        LanguageTransformerTestCase {
            inner: "買え",
            rule: "v5",
            reasons: vec!["imperative"],
        },
        LanguageTransformerTestCase {
            inner: "買わない",
            rule: "v5",
            reasons: vec!["negative"],
        },
        LanguageTransformerTestCase {
            inner: "買いません",
            rule: "v5",
            reasons: vec!["-ます", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "買わなかった",
            rule: "v5",
            reasons: vec!["negative", "-た"],
        },
        LanguageTransformerTestCase {
            inner: "買いませんでした",
            rule: "v5",
            reasons: vec!["-ます", "negative", "-た"],
        },
        LanguageTransformerTestCase {
            inner: "買わなくて",
            rule: "v5",
            reasons: vec!["negative", "-て"],
        },
        LanguageTransformerTestCase {
            inner: "買えない",
            rule: "v5",
            reasons: vec!["potential", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "買われない",
            rule: "v5",
            reasons: vec!["passive", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "買わせない",
            rule: "v5",
            reasons: vec!["causative", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "買わさない",
            rule: "v5",
            reasons: vec!["short causative", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "買わせられない",
            rule: "v5",
            reasons: vec!["causative", "potential or passive", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "買いまして",
            rule: "v5",
            reasons: vec!["-ます", "-て"],
        },
        LanguageTransformerTestCase {
            inner: "買えば",
            rule: "v5",
            reasons: vec!["-ば"],
        },
        LanguageTransformerTestCase {
            inner: "買や",
            rule: "v5",
            reasons: vec!["-ば", "-ゃ"],
        },
        LanguageTransformerTestCase {
            inner: "買っちゃ",
            rule: "v5",
            reasons: vec!["-ちゃ"],
        },
        LanguageTransformerTestCase {
            inner: "買っちゃう",
            rule: "v5",
            reasons: vec!["-ちゃう"],
        },
        LanguageTransformerTestCase {
            inner: "買っちまう",
            rule: "v5",
            reasons: vec!["-ちまう"],
        },
        LanguageTransformerTestCase {
            inner: "買いなさい",
            rule: "v5",
            reasons: vec!["-なさい"],
        },
        LanguageTransformerTestCase {
            inner: "買いそう",
            rule: "v5",
            reasons: vec!["-そう"],
        },
        LanguageTransformerTestCase {
            inner: "買いすぎる",
            rule: "v5",
            reasons: vec!["-すぎる"],
        },
        LanguageTransformerTestCase {
            inner: "買い過ぎる",
            rule: "v5",
            reasons: vec!["-過ぎる"],
        },
        LanguageTransformerTestCase {
            inner: "買いたい",
            rule: "v5",
            reasons: vec!["-たい"],
        },
        LanguageTransformerTestCase {
            inner: "買いたがる",
            rule: "v5",
            reasons: vec!["-たい", "-がる"],
        },
        LanguageTransformerTestCase {
            inner: "買ったら",
            rule: "v5",
            reasons: vec!["-たら"],
        },
        LanguageTransformerTestCase {
            inner: "買ったり",
            rule: "v5",
            reasons: vec!["-たり"],
        },
        LanguageTransformerTestCase {
            inner: "買わず",
            rule: "v5",
            reasons: vec!["-ず"],
        },
        LanguageTransformerTestCase {
            inner: "買わぬ",
            rule: "v5",
            reasons: vec!["-ぬ"],
        },
        LanguageTransformerTestCase {
            inner: "買わん",
            rule: "v5",
            reasons: vec!["-ん"],
        },
        LanguageTransformerTestCase {
            inner: "買わんかった",
            rule: "v5",
            reasons: vec!["-ん", "-た"],
        },
        LanguageTransformerTestCase {
            inner: "買わんばかり",
            rule: "v5",
            reasons: vec!["-んばかり"],
        },
        LanguageTransformerTestCase {
            inner: "買わんとする",
            rule: "v5",
            reasons: vec!["-んとする"],
        },
        LanguageTransformerTestCase {
            inner: "買わざる",
            rule: "v5",
            reasons: vec!["-ざる"],
        },
        LanguageTransformerTestCase {
            inner: "買わねば",
            rule: "v5",
            reasons: vec!["-ねば"],
        },
        LanguageTransformerTestCase {
            inner: "買わにゃ",
            rule: "v5",
            reasons: vec!["-ねば", "-ゃ"],
        },
        LanguageTransformerTestCase {
            inner: "買い",
            rule: "v5",
            reasons: vec!["continuative"],
        },
        LanguageTransformerTestCase {
            inner: "買いましょう",
            rule: "v5",
            reasons: vec!["-ます", "volitional"],
        },
        LanguageTransformerTestCase {
            inner: "買いましょっか",
            rule: "v5",
            reasons: vec!["-ます", "volitional slang"],
        },
        LanguageTransformerTestCase {
            inner: "買おう",
            rule: "v5",
            reasons: vec!["volitional"],
        },
        LanguageTransformerTestCase {
            inner: "買おっか",
            rule: "v5",
            reasons: vec!["volitional slang"],
        },
        LanguageTransformerTestCase {
            inner: "買うまい",
            rule: "v5",
            reasons: vec!["-まい"],
        },
        LanguageTransformerTestCase {
            inner: "買わされる",
            rule: "v5",
            reasons: vec!["short causative", "passive"],
        },
        LanguageTransformerTestCase {
            inner: "買っておく",
            rule: "v5",
            reasons: vec!["-て", "-おく"],
        },
        LanguageTransformerTestCase {
            inner: "買っとく",
            rule: "v5",
            reasons: vec!["-て", "-おく"],
        },
        LanguageTransformerTestCase {
            inner: "買わないでおく",
            rule: "v5",
            reasons: vec!["negative", "-おく"],
        },
        LanguageTransformerTestCase {
            inner: "買わないどく",
            rule: "v5",
            reasons: vec!["negative", "-おく"],
        },
        LanguageTransformerTestCase {
            inner: "買っている",
            rule: "v5",
            reasons: vec!["-て", "-いる"],
        },
        LanguageTransformerTestCase {
            inner: "買っておる",
            rule: "v5",
            reasons: vec!["-て", "-いる"],
        },
        LanguageTransformerTestCase {
            inner: "買ってる",
            rule: "v5",
            reasons: vec!["-て", "-いる"],
        },
        LanguageTransformerTestCase {
            inner: "買っとる",
            rule: "v5",
            reasons: vec!["-て", "-いる"],
        },
        LanguageTransformerTestCase {
            inner: "買ってしまう",
            rule: "v5",
            reasons: vec!["-て", "-しまう"],
        },
        LanguageTransformerTestCase {
            inner: "買いますまい",
            rule: "v5",
            reasons: vec!["-ます", "-まい"],
        },
        LanguageTransformerTestCase {
            inner: "買いましたら",
            rule: "v5",
            reasons: vec!["-ます", "-たら"],
        },
        LanguageTransformerTestCase {
            inner: "買いますれば",
            rule: "v5",
            reasons: vec!["-ます", "-ば"],
        },
        LanguageTransformerTestCase {
            inner: "買いませんかった",
            rule: "v5",
            reasons: vec!["-ます", "negative", "-た"],
        },
    ],
});
pub(crate) static JP_ICHIDAN_VERB_TESTS: LazyLock<TransformTest> =
    LazyLock::new(|| TransformTest {
        term: "食べる",
        sources: vec![
            LanguageTransformerTestCase {
                inner: "食べる",
                rule: "v1",
                reasons: vec![],
            },
            LanguageTransformerTestCase {
                inner: "食べます",
                rule: "v1",
                reasons: vec!["-ます"],
            },
            LanguageTransformerTestCase {
                inner: "食べた",
                rule: "v1",
                reasons: vec!["-た"],
            },
            LanguageTransformerTestCase {
                inner: "食べました",
                rule: "v1",
                reasons: vec!["-ます", "-た"],
            },
            LanguageTransformerTestCase {
                inner: "食べて",
                rule: "v1",
                reasons: vec!["-て"],
            },
            LanguageTransformerTestCase {
                inner: "食べられる",
                rule: "v1",
                reasons: vec!["potential or passive"],
            },
            LanguageTransformerTestCase {
                inner: "食べられる",
                rule: "v1",
                reasons: vec!["potential or passive"],
            },
            LanguageTransformerTestCase {
                inner: "食べさせる",
                rule: "v1",
                reasons: vec!["causative"],
            },
            LanguageTransformerTestCase {
                inner: "食べさす",
                rule: "v1",
                reasons: vec!["short causative"],
            },
            LanguageTransformerTestCase {
                inner: "食べさします",
                rule: "v1",
                reasons: vec!["short causative", "-ます"],
            },
            LanguageTransformerTestCase {
                inner: "食べさせられる",
                rule: "v1",
                reasons: vec!["causative", "potential or passive"],
            },
            LanguageTransformerTestCase {
                inner: "食べろ",
                rule: "v1",
                reasons: vec!["imperative"],
            },
            LanguageTransformerTestCase {
                inner: "食べない",
                rule: "v1",
                reasons: vec!["negative"],
            },
            LanguageTransformerTestCase {
                inner: "食べません",
                rule: "v1",
                reasons: vec!["-ます", "negative"],
            },
            LanguageTransformerTestCase {
                inner: "食べなかった",
                rule: "v1",
                reasons: vec!["negative", "-た"],
            },
            LanguageTransformerTestCase {
                inner: "食べませんでした",
                rule: "v1",
                reasons: vec!["-ます", "negative", "-た"],
            },
            LanguageTransformerTestCase {
                inner: "食べなくて",
                rule: "v1",
                reasons: vec!["negative", "-て"],
            },
            LanguageTransformerTestCase {
                inner: "食べられない",
                rule: "v1",
                reasons: vec!["potential or passive", "negative"],
            },
            LanguageTransformerTestCase {
                inner: "食べられない",
                rule: "v1",
                reasons: vec!["potential or passive", "negative"],
            },
            LanguageTransformerTestCase {
                inner: "食べさせない",
                rule: "v1",
                reasons: vec!["causative", "negative"],
            },
            LanguageTransformerTestCase {
                inner: "食べささない",
                rule: "v1",
                reasons: vec!["short causative", "negative"],
            },
            LanguageTransformerTestCase {
                inner: "食べさせられない",
                rule: "v1",
                reasons: vec!["causative", "potential or passive", "negative"],
            },
            LanguageTransformerTestCase {
                inner: "食べまして",
                rule: "v1",
                reasons: vec!["-ます", "-て"],
            },
            LanguageTransformerTestCase {
                inner: "食べれば",
                rule: "v1",
                reasons: vec!["-ば"],
            },
            LanguageTransformerTestCase {
                inner: "食べりゃ",
                rule: "v1",
                reasons: vec!["-ば", "-ゃ"],
            },
            LanguageTransformerTestCase {
                inner: "食べちゃ",
                rule: "v1",
                reasons: vec!["-ちゃ"],
            },
            LanguageTransformerTestCase {
                inner: "食べちゃう",
                rule: "v1",
                reasons: vec!["-ちゃう"],
            },
            LanguageTransformerTestCase {
                inner: "食べちまう",
                rule: "v1",
                reasons: vec!["-ちまう"],
            },
            LanguageTransformerTestCase {
                inner: "食べなさい",
                rule: "v1",
                reasons: vec!["-なさい"],
            },
            LanguageTransformerTestCase {
                inner: "食べそう",
                rule: "v1",
                reasons: vec!["-そう"],
            },
            LanguageTransformerTestCase {
                inner: "食べすぎる",
                rule: "v1",
                reasons: vec!["-すぎる"],
            },
            LanguageTransformerTestCase {
                inner: "食べ過ぎる",
                rule: "v1",
                reasons: vec!["-過ぎる"],
            },
            LanguageTransformerTestCase {
                inner: "食べたい",
                rule: "v1",
                reasons: vec!["-たい"],
            },
            LanguageTransformerTestCase {
                inner: "食べたがる",
                rule: "v1",
                reasons: vec!["-たい", "-がる"],
            },
            LanguageTransformerTestCase {
                inner: "食べたら",
                rule: "v1",
                reasons: vec!["-たら"],
            },
            LanguageTransformerTestCase {
                inner: "食べたり",
                rule: "v1",
                reasons: vec!["-たり"],
            },
            LanguageTransformerTestCase {
                inner: "食べず",
                rule: "v1",
                reasons: vec!["-ず"],
            },
            LanguageTransformerTestCase {
                inner: "食べぬ",
                rule: "v1",
                reasons: vec!["-ぬ"],
            },
            LanguageTransformerTestCase {
                inner: "食べん",
                rule: "v1",
                reasons: vec!["-ん"],
            },
            LanguageTransformerTestCase {
                inner: "食べんかった",
                rule: "v1",
                reasons: vec!["-ん", "-た"],
            },
            LanguageTransformerTestCase {
                inner: "食べんばかり",
                rule: "v1",
                reasons: vec!["-んばかり"],
            },
            LanguageTransformerTestCase {
                inner: "食べんとする",
                rule: "v1",
                reasons: vec!["-んとする"],
            },
            LanguageTransformerTestCase {
                inner: "食べざる",
                rule: "v1",
                reasons: vec!["-ざる"],
            },
            LanguageTransformerTestCase {
                inner: "食べねば",
                rule: "v1",
                reasons: vec!["-ねば"],
            },
            LanguageTransformerTestCase {
                inner: "食べにゃ",
                rule: "v1",
                reasons: vec!["-ねば", "-ゃ"],
            },
            LanguageTransformerTestCase {
                inner: "食べ",
                rule: "v1d",
                reasons: vec!["continuative"],
            },
            LanguageTransformerTestCase {
                inner: "食べましょう",
                rule: "v1",
                reasons: vec!["-ます", "volitional"],
            },
            LanguageTransformerTestCase {
                inner: "食べましょっか",
                rule: "v1",
                reasons: vec!["-ます", "volitional slang"],
            },
            LanguageTransformerTestCase {
                inner: "食べよう",
                rule: "v1",
                reasons: vec!["volitional"],
            },
            LanguageTransformerTestCase {
                inner: "食べよっか",
                rule: "v1",
                reasons: vec!["volitional slang"],
            },
            LanguageTransformerTestCase {
                inner: "食べるまい",
                rule: "v1",
                reasons: vec!["-まい"],
            },
            LanguageTransformerTestCase {
                inner: "食べまい",
                rule: "v1",
                reasons: vec!["-まい"],
            },
            LanguageTransformerTestCase {
                inner: "食べておく",
                rule: "v1",
                reasons: vec!["-て", "-おく"],
            },
            LanguageTransformerTestCase {
                inner: "食べとく",
                rule: "v1",
                reasons: vec!["-て", "-おく"],
            },
            LanguageTransformerTestCase {
                inner: "食べないでおく",
                rule: "v1",
                reasons: vec!["negative", "-おく"],
            },
            LanguageTransformerTestCase {
                inner: "食べないどく",
                rule: "v1",
                reasons: vec!["negative", "-おく"],
            },
            LanguageTransformerTestCase {
                inner: "食べている",
                rule: "v1",
                reasons: vec!["-て", "-いる"],
            },
            LanguageTransformerTestCase {
                inner: "食べておる",
                rule: "v1",
                reasons: vec!["-て", "-いる"],
            },
            LanguageTransformerTestCase {
                inner: "食べてる",
                rule: "v1",
                reasons: vec!["-て", "-いる"],
            },
            LanguageTransformerTestCase {
                inner: "食べとる",
                rule: "v1",
                reasons: vec!["-て", "-いる"],
            },
            LanguageTransformerTestCase {
                inner: "食べてしまう",
                rule: "v1",
                reasons: vec!["-て", "-しまう"],
            },
        ],
    });
pub(crate) static JP_VERB_KU_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "行く",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "行く",
            rule: "v5",
            reasons: vec![],
        },
        LanguageTransformerTestCase {
            inner: "行きます",
            rule: "v5",
            reasons: vec!["-ます"],
        },
        LanguageTransformerTestCase {
            inner: "行った",
            rule: "v5",
            reasons: vec!["-た"],
        },
        LanguageTransformerTestCase {
            inner: "行きました",
            rule: "v5",
            reasons: vec!["-ます", "-た"],
        },
        LanguageTransformerTestCase {
            inner: "行って",
            rule: "v5",
            reasons: vec!["-て"],
        },
        LanguageTransformerTestCase {
            inner: "行ける",
            rule: "v5",
            reasons: vec!["potential"],
        },
        LanguageTransformerTestCase {
            inner: "行かれる",
            rule: "v5",
            reasons: vec!["passive"],
        },
        LanguageTransformerTestCase {
            inner: "行かせる",
            rule: "v5",
            reasons: vec!["causative"],
        },
        LanguageTransformerTestCase {
            inner: "行かす",
            rule: "v5",
            reasons: vec!["short causative"],
        },
        LanguageTransformerTestCase {
            inner: "行かします",
            rule: "v5",
            reasons: vec!["short causative", "-ます"],
        },
        LanguageTransformerTestCase {
            inner: "行かせられる",
            rule: "v5",
            reasons: vec!["causative", "potential or passive"],
        },
        LanguageTransformerTestCase {
            inner: "行け",
            rule: "v5",
            reasons: vec!["imperative"],
        },
        LanguageTransformerTestCase {
            inner: "行かない",
            rule: "v5",
            reasons: vec!["negative"],
        },
        LanguageTransformerTestCase {
            inner: "行きません",
            rule: "v5",
            reasons: vec!["-ます", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "行かなかった",
            rule: "v5",
            reasons: vec!["negative", "-た"],
        },
        LanguageTransformerTestCase {
            inner: "行きませんでした",
            rule: "v5",
            reasons: vec!["-ます", "negative", "-た"],
        },
        LanguageTransformerTestCase {
            inner: "行かなくて",
            rule: "v5",
            reasons: vec!["negative", "-て"],
        },
        LanguageTransformerTestCase {
            inner: "行けない",
            rule: "v5",
            reasons: vec!["potential", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "行かれない",
            rule: "v5",
            reasons: vec!["passive", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "行かせない",
            rule: "v5",
            reasons: vec!["causative", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "行かさない",
            rule: "v5",
            reasons: vec!["short causative", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "行かせられない",
            rule: "v5",
            reasons: vec!["causative", "potential or passive", "negative"],
        },
        LanguageTransformerTestCase {
            inner: "行きまして",
            rule: "v5",
            reasons: vec!["-ます", "-て"],
        },
        LanguageTransformerTestCase {
            inner: "行けば",
            rule: "v5",
            reasons: vec!["-ば"],
        },
        LanguageTransformerTestCase {
            inner: "行きゃ",
            rule: "v5",
            reasons: vec!["-ば", "-ゃ"],
        },
        LanguageTransformerTestCase {
            inner: "行っちゃ",
            rule: "v5",
            reasons: vec!["-ちゃ"],
        },
        LanguageTransformerTestCase {
            inner: "行っちゃう",
            rule: "v5",
            reasons: vec!["-ちゃう"],
        },
        LanguageTransformerTestCase {
            inner: "行っちまう",
            rule: "v5",
            reasons: vec!["-ちまう"],
        },
        LanguageTransformerTestCase {
            inner: "行きなさい",
            rule: "v5",
            reasons: vec!["-なさい"],
        },
        LanguageTransformerTestCase {
            inner: "行きそう",
            rule: "v5",
            reasons: vec!["-そう"],
        },
        LanguageTransformerTestCase {
            inner: "行きすぎる",
            rule: "v5",
            reasons: vec!["-すぎる"],
        },
        LanguageTransformerTestCase {
            inner: "行き過ぎる",
            rule: "v5",
            reasons: vec!["-過ぎる"],
        },
        LanguageTransformerTestCase {
            inner: "行きたい",
            rule: "v5",
            reasons: vec!["-たい"],
        },
        LanguageTransformerTestCase {
            inner: "行きたがる",
            rule: "v5",
            reasons: vec!["-たい", "-がる"],
        },
        LanguageTransformerTestCase {
            inner: "行ったら",
            rule: "v5",
            reasons: vec!["-たら"],
        },
        LanguageTransformerTestCase {
            inner: "行ったり",
            rule: "v5",
            reasons: vec!["-たり"],
        },
        LanguageTransformerTestCase {
            inner: "行かず",
            rule: "v5",
            reasons: vec!["-ず"],
        },
        LanguageTransformerTestCase {
            inner: "行かぬ",
            rule: "v5",
            reasons: vec!["-ぬ"],
        },
        LanguageTransformerTestCase {
            inner: "行かん",
            rule: "v5",
            reasons: vec!["-ん"],
        },
        LanguageTransformerTestCase {
            inner: "行かんかった",
            rule: "v5",
            reasons: vec!["-ん", "-た"],
        },
        LanguageTransformerTestCase {
            inner: "行かんばかり",
            rule: "v5",
            reasons: vec!["-んばかり"],
        },
        LanguageTransformerTestCase {
            inner: "行かんとする",
            rule: "v5",
            reasons: vec!["-んとする"],
        },
        LanguageTransformerTestCase {
            inner: "行かざる",
            rule: "v5",
            reasons: vec!["-ざる"],
        },
        LanguageTransformerTestCase {
            inner: "行かねば",
            rule: "v5",
            reasons: vec!["-ねば"],
        },
        LanguageTransformerTestCase {
            inner: "行かにゃ",
            rule: "v5",
            reasons: vec!["-ねば", "-ゃ"],
        },
        LanguageTransformerTestCase {
            inner: "行き",
            rule: "v5",
            reasons: vec!["continuative"],
        },
        LanguageTransformerTestCase {
            inner: "行きましょう",
            rule: "v5",
            reasons: vec!["-ます", "volitional"],
        },
        LanguageTransformerTestCase {
            inner: "行きましょっか",
            rule: "v5",
            reasons: vec!["-ます", "volitional slang"],
        },
        LanguageTransformerTestCase {
            inner: "行こう",
            rule: "v5",
            reasons: vec!["volitional"],
        },
        LanguageTransformerTestCase {
            inner: "行こっか",
            rule: "v5",
            reasons: vec!["volitional slang"],
        },
        LanguageTransformerTestCase {
            inner: "行くまい",
            rule: "v5",
            reasons: vec!["-まい"],
        },
        LanguageTransformerTestCase {
            inner: "行かされる",
            rule: "v5",
            reasons: vec!["short causative", "passive"],
        },
        LanguageTransformerTestCase {
            inner: "行っておく",
            rule: "v5",
            reasons: vec!["-て", "-おく"],
        },
        LanguageTransformerTestCase {
            inner: "行っとく",
            rule: "v5",
            reasons: vec!["-て", "-おく"],
        },
        LanguageTransformerTestCase {
            inner: "行かないでおく",
            rule: "v5",
            reasons: vec!["negative", "-おく"],
        },
        LanguageTransformerTestCase {
            inner: "行かないどく",
            rule: "v5",
            reasons: vec!["negative", "-おく"],
        },
        LanguageTransformerTestCase {
            inner: "行っている",
            rule: "v5",
            reasons: vec!["-て", "-いる"],
        },
        LanguageTransformerTestCase {
            inner: "行っておる",
            rule: "v5",
            reasons: vec!["-て", "-いる"],
        },
        LanguageTransformerTestCase {
            inner: "行ってる",
            rule: "v5",
            reasons: vec!["-て", "-いる"],
        },
        LanguageTransformerTestCase {
            inner: "行っとる",
            rule: "v5",
            reasons: vec!["-て", "-いる"],
        },
        LanguageTransformerTestCase {
            inner: "行ってしまう",
            rule: "v5",
            reasons: vec!["-て", "-しまう"],
        },
        LanguageTransformerTestCase {
            inner: "行きますまい",
            rule: "v5",
            reasons: vec!["-ます", "-まい"],
        },
        LanguageTransformerTestCase {
            inner: "行きましたら",
            rule: "v5",
            reasons: vec!["-ます", "-たら"],
        },
        LanguageTransformerTestCase {
            inner: "行きますれば",
            rule: "v5",
            reasons: vec!["-ます", "-ば"],
        },
        LanguageTransformerTestCase {
            inner: "行きませんかった",
            rule: "v5",
            reasons: vec!["-ます", "negative", "-た"],
        },
    ],
});
pub(crate) static JP_VERB_GU_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "泳ぐ",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "泳ぐ",
            rule: "v5",
            reasons: vec![],
        },
        LanguageTransformerTestCase {
            inner: "泳ぎます",
            rule: "v5",
            reasons: vec!["-ます"],
        },
        LanguageTransformerTestCase {
            inner: "泳いだ",
            rule: "v5",
            reasons: vec!["-た"],
        },
        // Add all other test cases from the JavaScript 'ぐ verbs' category
    ],
});
pub(crate) static JP_VERB_SU_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "話す",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "話す",
            rule: "v5",
            reasons: vec![],
        },
        LanguageTransformerTestCase {
            inner: "話します",
            rule: "v5",
            reasons: vec!["-ます"],
        },
        // Add all other test cases from the JavaScript 'す verbs' category
    ],
});
pub(crate) static JP_VERB_TSU_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "待つ",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "待つ",
            rule: "v5",
            reasons: vec![],
        },
        LanguageTransformerTestCase {
            inner: "待ちます",
            rule: "v5",
            reasons: vec!["-ます"],
        },
        // Add all other test cases
    ],
});
pub(crate) static JP_VERB_NU_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "死ぬ",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "死ぬ",
            rule: "v5",
            reasons: vec![],
        },
        LanguageTransformerTestCase {
            inner: "死にます",
            rule: "v5",
            reasons: vec!["-ます"],
        },
        // Add all test cases
    ],
});
pub(crate) static JP_VERB_BU_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "遊ぶ",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "遊ぶ",
            rule: "v5",
            reasons: vec![],
        },
        LanguageTransformerTestCase {
            inner: "遊びます",
            rule: "v5",
            reasons: vec!["-ます"],
        },
        // Add all cases
    ],
});
pub(crate) static JP_VERB_MU_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "飲む",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "飲む",
            rule: "v5",
            reasons: vec![],
        },
        LanguageTransformerTestCase {
            inner: "飲みます",
            rule: "v5",
            reasons: vec!["-ます"],
        },
        // Add all cases
    ],
});
pub(crate) static JP_IRREGULAR_VERB_SURU_TESTS: LazyLock<TransformTest> =
    LazyLock::new(|| TransformTest {
        term: "為る",
        sources: vec![
            LanguageTransformerTestCase {
                inner: "為る",
                rule: "vs",
                reasons: vec![],
            },
            LanguageTransformerTestCase {
                inner: "為ます",
                rule: "vs",
                reasons: vec!["-ます"],
            },
            // Add all 為る and する cases
        ],
    });
pub(crate) static JP_IRREGULAR_VERB_KURU_TESTS: LazyLock<TransformTest> =
    LazyLock::new(|| TransformTest {
        term: "来る",
        sources: vec![
            LanguageTransformerTestCase {
                inner: "来る",
                rule: "vk",
                reasons: vec![],
            },
            LanguageTransformerTestCase {
                inner: "来ます",
                rule: "vk",
                reasons: vec!["-ます"],
            },
            // Add all 来る, 來る, くる cases
        ],
    });
pub(crate) static JP_ZURU_VERB_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "論ずる",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "論ずる",
            rule: "vz",
            reasons: vec![],
        },
        LanguageTransformerTestCase {
            inner: "論じます",
            rule: "vz",
            reasons: vec!["-ます"],
        },
        // Add all cases
    ],
});
pub(crate) static JP_EE_ENDING_TESTS: LazyLock<TransformTest> = LazyLock::new(|| TransformTest {
    term: "すごい",
    sources: vec![
        LanguageTransformerTestCase {
            inner: "すげえ",
            rule: "adj-i",
            reasons: vec!["-え"],
        },
        LanguageTransformerTestCase {
            inner: "すげぇ",
            rule: "adj-i",
            reasons: vec!["-え"],
        },
        // Add all え ending cases
    ],
});

/// https://raw.githubusercontent.com/yomidevs/yomitan/c3bec65bc44a33b1b1686e5d81a6910e42889174/ext/js/language/ja/japanese-transforms.js
use indexmap::IndexMap;

use crate::transformer::{LanguageTransformDescriptor, Transform, TransformI18n, TransformMap};

pub(crate) const SHIMAU_ENGLISH_DESCRIPTION: &str = "1. Shows a sense of regret/surprise when you did have volition in doing something, but it turned out to be bad to do.\n2. Shows perfective/punctual achievement. This shows that an action has been completed.\n 3. Shows unintentional action–“accidentally”.\n";
pub(crate) const PASSIVE_ENGLISH_DESCRIPTION: &str = "1. Indicates an action received from an action performer.\n2. Expresses respect for the subject of action performer.\n";
pub(crate) const IKU_VERBS: [&str; 4] = ["いく", "行く", "逝く", "往く"];
#[rustfmt::skip]
pub(crate) const GODAN_U_SPECIAL_VERBS: [&str; 12] = [
    "こう", "とう", "請う", "乞う", "恋う", "問う", "訪う",
    "宣う", "曰う", "給う", "賜う", "揺蕩う",
];
#[rustfmt::skip]
pub(crate) const FU_VERB_TE_CONJUGATIONS: [[&str; 2]; 3] = [
  ["のたまう", "のたもう"],
  ["たまう", "たもう"],
  ["たゆたう", "たゆとう"]
];

pub static JAPANESE_TRANSFORMS: LazyLock<LanguageTransformDescriptor> =
    LazyLock::new(|| LanguageTransformDescriptor {
        language: "ja".to_string(),
        conditions: &JP_CONDITIONS,
        transforms: &JP_TRANSFORMS,
    });

#[cfg(test)]
pub(crate) mod jp_transforms {
    use crate::transformer::LanguageTransformer;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn len() {
        assert_eq!(JAPANESE_TRANSFORMS.transforms.len(), 53);
        assert_eq!(JAPANESE_TRANSFORMS.conditions.len(), 22);
    }

    #[test]
    fn transforms() {
        let mut lt = LanguageTransformer::new();
        lt.add_descriptor(&JAPANESE_TRANSFORMS).unwrap();

        for (i, test) in TRANSFORM_TESTS.iter().enumerate() {
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

pub(crate) fn has_term_reasons(
    lt: &LanguageTransformer,
    source: &str,
    expected_term: &str,
    expected_condition_name: Option<&str>,
    expected_reasons: Option<&[&str]>,
) -> Result<HasTermReasons, HasTermReasonsError> {
    let results = lt.transform(source);
    let rule = expected_condition_name.unwrap_or("");
    let mut rejected = Vec::new();

    for result in results {
        let mut rejection_reasons = Vec::new();

        // Check term match
        if result.text != expected_term {
            rejection_reasons.push(format!(
                "Term mismatch: expected '{}', got '{}'",
                expected_term, result.text
            ));
        }

        // Check rule match if term matched
        if result.text == expected_term {
            if let Some(expected_name) = expected_condition_name {
                let expected_conditions =
                    lt.get_condition_flags_from_single_condition_type(expected_name);
                if !LanguageTransformer::conditions_match(result.conditions, expected_conditions) {
                    rejection_reasons.push(format!(
                        "Condition mismatch: expected {}({:b}), got {:b}",
                        expected_name, expected_conditions, result.conditions
                    ));
                }
            }
        }

        // If we had any rejection reasons, log and continue
        if !rejection_reasons.is_empty() {
            rejected.push(format!(
                "Candidate '{}' [conditions {:b}] rejected because:\n  {}",
                result.text,
                result.conditions,
                rejection_reasons.join("\n  ")
            ));
            continue;
        }

        // check trace reasons if we got this far
        if let Some(expected) = expected_reasons {
            if result.trace.len() != expected.len() {
                return Err(HasTermReasonsError::TraceLengthMismatch {
                    expected: expected.len(),
                    found: result.trace.len(),
                });
            }

            // Check individual reasons
            for (i, (actual, expected)) in result.trace.iter().zip(expected.iter()).enumerate() {
                if &actual.transform != expected {
                    return Err(HasTermReasonsError::ReasonMismatch {
                        index: i,
                        expected: (*expected).to_string(),
                        found: actual.transform.clone(),
                    });
                }
            }
        }

        // Success case
        return Ok(HasTermReasons {
            reasons: result.trace.iter().map(|f| f.transform.clone()).collect(),
            rules: result.conditions as usize,
        });
    }

    // No matches found - return all rejection reasons
    Err(HasTermReasonsError::NoMatch {
        src: source.to_string(),
        term: expected_term.to_string(),
        rule: rule.to_string(),
        rejected,
    })
}

pub static JP_TRANSFORMS: LazyLock<TransformMap> = LazyLock::new(|| {
    TransformMap(IndexMap::from([
        (
            "-ば",
            Transform {
                name: "-ば",
                description: Some(
                    "1. Conditional form; shows that the previous stated condition's establishment is the condition for the latter stated condition to occur.\n2. Shows a trigger for a latter stated perception or judgment.\nUsage: Attach ば to the hypothetical form (仮定形) of verbs and i-adjectives.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ば",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("ければ", "い", &["-ば"], &["adj-i"]).into(),
                    suffix_inflection("えば", "う", &["-ば"], &["v5"]).into(),
                    suffix_inflection("けば", "く", &["-ば"], &["v5"]).into(),
                    suffix_inflection("げば", "ぐ", &["-ば"], &["v5"]).into(),
                    suffix_inflection("せば", "す", &["-ば"], &["v5"]).into(),
                    suffix_inflection("てば", "つ", &["-ば"], &["v5"]).into(),
                    suffix_inflection("ねば", "ぬ", &["-ば"], &["v5"]).into(),
                    suffix_inflection("べば", "ぶ", &["-ば"], &["v5"]).into(),
                    suffix_inflection("めば", "む", &["-ば"], &["v5"]).into(),
                    suffix_inflection("れば", "る", &["-ば"], &["v1", "v5", "vk", "vs", "vz"]).into(),
                    suffix_inflection("れば", "",   &["-ば"], &["-ます"]).into(),
                ],
            },
        ),
        (
            "-ゃ",
            Transform {
                name: "-ゃ",
                description: Some("Contraction of -ば."),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ゃ",
                    description: Some("「～ば」の短縮"),
                }]),
                rules: vec![
                    suffix_inflection("けりゃ", "ければ", &["-ゃ"], &["-ば"]).into(),
                    suffix_inflection("きゃ", "ければ", &["-ゃ"], &["-ば"]).into(),
                    suffix_inflection("や", "えば", &["-ゃ"], &["-ば"]).into(),
                    suffix_inflection("きゃ", "けば", &["-ゃ"], &["-ば"]).into(),
                    suffix_inflection("ぎゃ", "げば", &["-ゃ"], &["-ば"]).into(),
                    suffix_inflection("しゃ", "せば", &["-ゃ"], &["-ば"]).into(),
                    suffix_inflection("ちゃ", "てば", &["-ゃ"], &["-ば"]).into(),
                    suffix_inflection("にゃ", "ねば", &["-ゃ"], &["-ば"]).into(),
                    suffix_inflection("びゃ", "べば", &["-ゃ"], &["-ば"]).into(),
                    suffix_inflection("みゃ", "めば", &["-ゃ"], &["-ば"]).into(),
                    suffix_inflection("りゃ", "れば", &["-ゃ"], &["-ば"]).into(),
                ],
            },
        ),
        (
            "-ちゃ",
            Transform {
                name: "-ちゃ",
                description: Some(
                    "Contraction of ～ては.\n1. Explains how something always happens under the condition that it marks.\n2. Expresses the repetition (of a series of) actions.\n3. Indicates a hypothetical situation in which the speaker gives a (negative) evaluation about the other party's intentions.\n4. Used in \"Must Not\" patterns like ～てはいけない.\nUsage: Attach は after the て-form of verbs, contract ては into ちゃ.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ちゃ",
                    description: Some("「～ては」の短縮"),
                }]),
                rules: vec![
                    suffix_inflection("ちゃ", "る", &["v5"], &["v1"]).into(),
                    suffix_inflection("いじゃ", "ぐ", &["v5"], &["v5"]).into(),
                    suffix_inflection("いちゃ", "く", &["v5"], &["v5"]).into(),
                    suffix_inflection("しちゃ", "す", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちゃ", "う", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちゃ", "く", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちゃ", "つ", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちゃ", "る", &["v5"], &["v5"]).into(),
                    suffix_inflection("んじゃ", "ぬ", &["v5"], &["v5"]).into(),
                    suffix_inflection("んじゃ", "ぶ", &["v5"], &["v5"]).into(),
                    suffix_inflection("んじゃ", "む", &["v5"], &["v5"]).into(),
                    suffix_inflection("じちゃ", "ずる", &["v5"], &["vz"]).into(),
                    suffix_inflection("しちゃ", "する", &["v5"], &["vs"]).into(),
                    suffix_inflection("為ちゃ", "為る", &["v5"], &["vs"]).into(),
                    suffix_inflection("きちゃ", "くる", &["v5"], &["vk"]).into(),
                    suffix_inflection("来ちゃ", "来る", &["v5"], &["vk"]).into(),
                    suffix_inflection("來ちゃ", "來る", &["v5"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-ちゃう",
            Transform {
                name: "-ちゃう",
                description: Some(
                    "Contraction of -しまう.\nShows completion of an action with regret or accidental completion.\nUsage: Attach しまう after the て-form of verbs, contract てしまう into ちゃう.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ちゃう",
                    description: Some("「～てしまう」のややくだけた口頭語的表現"),
                }]),
                rules: vec![
                    suffix_inflection("ちゃう", "る", &["v5"], &["v1"]).into(),
                    suffix_inflection("いじゃう", "ぐ", &["v5"], &["v5"]).into(),
                    suffix_inflection("いちゃう", "く", &["v5"], &["v5"]).into(),
                    suffix_inflection("しちゃう", "す", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちゃう", "う", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちゃう", "く", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちゃう", "つ", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちゃう", "る", &["v5"], &["v5"]).into(),
                    suffix_inflection("んじゃう", "ぬ", &["v5"], &["v5"]).into(),
                    suffix_inflection("んじゃう", "ぶ", &["v5"], &["v5"]).into(),
                    suffix_inflection("んじゃう", "む", &["v5"], &["v5"]).into(),
                    suffix_inflection("じちゃう", "ずる", &["v5"], &["vz"]).into(),
                    suffix_inflection("しちゃう", "する", &["v5"], &["vs"]).into(),
                    suffix_inflection("為ちゃう", "為る", &["v5"], &["vs"]).into(),
                    suffix_inflection("きちゃう", "くる", &["v5"], &["vk"]).into(),
                    suffix_inflection("来ちゃう", "来る", &["v5"], &["vk"]).into(),
                    suffix_inflection("來ちゃう", "來る", &["v5"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-ちまう",
            Transform {
                name: "-ちまう",
                description: Some(
                    "Contraction of -しまう.\nShows completion of an action with regret or accidental completion.\nUsage: Attach しまう after the て-form of verbs, contract てしまう into ちまう.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ちまう",
                    description: Some("「～てしまう」の音変化"),
                }]),
                rules: vec![
                    suffix_inflection("ちまう", "る", &["v5"], &["v1"]).into(),
                    suffix_inflection("いじまう", "ぐ", &["v5"], &["v5"]).into(),
                    suffix_inflection("いちまう", "く", &["v5"], &["v5"]).into(),
                    suffix_inflection("しちまう", "す", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちまう", "う", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちまう", "く", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちまう", "つ", &["v5"], &["v5"]).into(),
                    suffix_inflection("っちまう", "る", &["v5"], &["v5"]).into(),
                    suffix_inflection("んじまう", "ぬ", &["v5"], &["v5"]).into(),
                    suffix_inflection("んじまう", "ぶ", &["v5"], &["v5"]).into(),
                    suffix_inflection("んじまう", "む", &["v5"], &["v5"]).into(),
                    suffix_inflection("じちまう", "ずる", &["v5"], &["vz"]).into(),
                    suffix_inflection("しちまう", "する", &["v5"], &["vs"]).into(),
                    suffix_inflection("為ちまう", "為る", &["v5"], &["vs"]).into(),
                    suffix_inflection("きちまう", "くる", &["v5"], &["vk"]).into(),
                    suffix_inflection("来ちまう", "来る", &["v5"], &["vk"]).into(),
                    suffix_inflection("來ちまう", "來る", &["v5"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-しまう",
            Transform {
                name: "-しまう",
                description: Some(
                    "Shows completion of an action with regret or accidental completion.\nUsage: Attach しまう after the て-form of verbs.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～しまう",
                    description: Some(
                        "その動作がすっかり終わる、その状態が完成することを表す。終わったことを強調したり、不本意である、困ったことになった、などの気持ちを添えたりすることもある。",
                    ),
                }]),
                rules: vec![
                    suffix_inflection("てしまう", "て", &["v5"], &["-て"]).into(),
                    suffix_inflection("でしまう", "で", &["v5"], &["-て"]).into(),
                ],
            },
        ),
        (
            "-なさい",
            Transform {
                name: "-なさい",
                description: Some(
                    "Polite imperative suffix.\nUsage: Attach なさい after the continuative form (連用形) of verbs.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～なさい",
                    description: Some("動詞「なさる」の命令形"),
                }]),
                rules: vec![
                    suffix_inflection("なさい", "る", &["-なさい"], &["v1"]).into(),
                    suffix_inflection("いなさい", "う", &["-なさい"], &["v5"]).into(),
                    suffix_inflection("きなさい", "く", &["-なさい"], &["v5"]).into(),
                    suffix_inflection("ぎなさい", "ぐ", &["-なさい"], &["v5"]).into(),
                    suffix_inflection("しなさい", "す", &["-なさい"], &["v5"]).into(),
                    suffix_inflection("ちなさい", "つ", &["-なさい"], &["v5"]).into(),
                    suffix_inflection("になさい", "ぬ", &["-なさい"], &["v5"]).into(),
                    suffix_inflection("びなさい", "ぶ", &["-なさい"], &["v5"]).into(),
                    suffix_inflection("みなさい", "む", &["-なさい"], &["v5"]).into(),
                    suffix_inflection("りなさい", "る", &["-なさい"], &["v5"]).into(),
                    suffix_inflection("じなさい", "ずる", &["-なさい"], &["vz"]).into(),
                    suffix_inflection("しなさい", "する", &["-なさい"], &["vs"]).into(),
                    suffix_inflection("為なさい", "為る", &["-なさい"], &["vs"]).into(),
                    suffix_inflection("きなさい", "くる", &["-なさい"], &["vk"]).into(),
                    suffix_inflection("来なさい", "来る", &["-なさい"], &["vk"]).into(),
                    suffix_inflection("來なさい", "來る", &["-なさい"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-そう",
            Transform {
                name: "-そう",
                description: Some(
                    "Appearing that; looking like.\nUsage: Attach そう to the continuative form (連用形) of verbs, or to the stem of adjectives.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～そう",
                    description: Some("そういう様子だ、そうなる様子だということ、すなわち様態を表す助動詞。"),
                }]),
                rules: vec![
                    suffix_inflection("そう", "い", &[], &["adj-i"]).into(),
                    suffix_inflection("そう", "る", &[], &["v1"]).into(),
                    suffix_inflection("いそう", "う", &[], &["v5"]).into(),
                    suffix_inflection("きそう", "く", &[], &["v5"]).into(),
                    suffix_inflection("ぎそう", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("しそう", "す", &[], &["v5"]).into(),
                    suffix_inflection("ちそう", "つ", &[], &["v5"]).into(),
                    suffix_inflection("にそう", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("びそう", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("みそう", "む", &[], &["v5"]).into(),
                    suffix_inflection("りそう", "る", &[], &["v5"]).into(),
                    suffix_inflection("じそう", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("しそう", "する", &[], &["vs"]).into(),
                    suffix_inflection("為そう", "為る", &[], &["vs"]).into(),
                    suffix_inflection("きそう", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来そう", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來そう", "來る", &[], &["vk"]).into(),
                ],
            },
        ),
        (
            "-すぎる",
            Transform {
                name: "-すぎる",
                description: Some(
                    "Shows something \"is too...\" or someone is doing something \"too much\".\nUsage: Attach すぎる to the continuative form (連用形) of verbs, or to the stem of adjectives.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～すぎる",
                    description: Some("程度や限度を超える"),
                }]),
                rules: vec![
                    suffix_inflection("すぎる", "い", &["v1"], &["adj-i"]).into(),
                    suffix_inflection("すぎる", "る", &["v1"], &["v1"]).into(),
                    suffix_inflection("いすぎる", "う", &["v1"], &["v5"]).into(),
                    suffix_inflection("きすぎる", "く", &["v1"], &["v5"]).into(),
                    suffix_inflection("ぎすぎる", "ぐ", &["v1"], &["v5"]).into(),
                    suffix_inflection("しすぎる", "す", &["v1"], &["v5"]).into(),
                    suffix_inflection("ちすぎる", "つ", &["v1"], &["v5"]).into(),
                    suffix_inflection("にすぎる", "ぬ", &["v1"], &["v5"]).into(),
                    suffix_inflection("びすぎる", "ぶ", &["v1"], &["v5"]).into(),
                    suffix_inflection("みすぎる", "む", &["v1"], &["v5"]).into(),
                    suffix_inflection("りすぎる", "る", &["v1"], &["v5"]).into(),
                    suffix_inflection("じすぎる", "ずる", &["v1"], &["vz"]).into(),
                    suffix_inflection("しすぎる", "する", &["v1"], &["vs"]).into(),
                    suffix_inflection("為すぎる", "為る", &["v1"], &["vs"]).into(),
                    suffix_inflection("きすぎる", "くる", &["v1"], &["vk"]).into(),
                    suffix_inflection("来すぎる", "来る", &["v1"], &["vk"]).into(),
                    suffix_inflection("來すぎる", "來る", &["v1"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-過ぎる",
            Transform {
                name: "-過ぎる",
                description: Some(
                    "Shows something \"is too...\" or someone is doing something \"too much\".\nUsage: Attach 過ぎる to the continuative form (連用形) of verbs, or to the stem of adjectives.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～過ぎる",
                    description: Some("程度や限度を超える"),
                }]),
                rules: vec![
                    suffix_inflection("過ぎる", "い", &["v1"], &["adj-i"]).into(),
                    suffix_inflection("過ぎる", "る", &["v1"], &["v1"]).into(),
                    suffix_inflection("い過ぎる", "う", &["v1"], &["v5"]).into(),
                    suffix_inflection("き過ぎる", "く", &["v1"], &["v5"]).into(),
                    suffix_inflection("ぎ過ぎる", "ぐ", &["v1"], &["v5"]).into(),
                    suffix_inflection("し過ぎる", "す", &["v1"], &["v5"]).into(),
                    suffix_inflection("ち過ぎる", "つ", &["v1"], &["v5"]).into(),
                    suffix_inflection("に過ぎる", "ぬ", &["v1"], &["v5"]).into(),
                    suffix_inflection("び過ぎる", "ぶ", &["v1"], &["v5"]).into(),
                    suffix_inflection("み過ぎる", "む", &["v1"], &["v5"]).into(),
                    suffix_inflection("り過ぎる", "る", &["v1"], &["v5"]).into(),
                    suffix_inflection("じ過ぎる", "ずる", &["v1"], &["vz"]).into(),
                    suffix_inflection("し過ぎる", "する", &["v1"], &["vs"]).into(),
                    suffix_inflection("為過ぎる", "為る", &["v1"], &["vs"]).into(),
                    suffix_inflection("き過ぎる", "くる", &["v1"], &["vk"]).into(),
                    suffix_inflection("来過ぎる", "来る", &["v1"], &["vk"]).into(),
                    suffix_inflection("來過ぎる", "來る", &["v1"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-たい",
            Transform {
                name: "-たい",
                description: Some(
                    "1. Expresses the feeling of desire or hope.\n2. Used in ...たいと思います, an indirect way of saying what the speaker intends to do.\nUsage: Attach たい to the continuative form (連用形) of verbs. たい itself conjugates as i-adjective.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～たい",
                    description: Some("することをのぞんでいる、という、希望や願望の気持ちをあらわす。"),
                }]),
                rules: vec![
                    suffix_inflection("たい", "る", &["adj-i"], &["v1"]).into(),
                    suffix_inflection("いたい", "う", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("きたい", "く", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("ぎたい", "ぐ", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("したい", "す", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("ちたい", "つ", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("にたい", "ぬ", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("びたい", "ぶ", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("みたい", "む", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("りたい", "る", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("じたい", "ずる", &["adj-i"], &["vz"]).into(),
                    suffix_inflection("したい", "する", &["adj-i"], &["vs"]).into(),
                    suffix_inflection("為たい", "為る", &["adj-i"], &["vs"]).into(),
                    suffix_inflection("きたい", "くる", &["adj-i"], &["vk"]).into(),
                    suffix_inflection("来たい", "来る", &["adj-i"], &["vk"]).into(),
                    suffix_inflection("來たい", "來る", &["adj-i"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-たら",
            Transform {
                name: "-たら",
                description: Some(
                    "1. Denotes the latter stated event is a continuation of the previous stated event.\n2. Assumes that a matter has been completed or concluded.\nUsage: Attach たら to the continuative form (連用形) of verbs after euphonic change form, かったら to the stem of i-adjectives.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～たら",
                    description: Some("仮定をあらわす・…すると・したあとに"),
                }]),
                rules: vec![
                    suffix_inflection("かったら", "い", &[], &["adj-i"]).into(),
                    suffix_inflection("たら",  "る", &[], &["v1"]).into(),
                    suffix_inflection("いたら", "く", &[], &["v5"]).into(),
                    suffix_inflection("いだら", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("したら", "す", &[], &["v5"]).into(),
                    suffix_inflection("ったら", "う", &[], &["v5"]).into(),
                    suffix_inflection("ったら", "つ", &[], &["v5"]).into(),
                    suffix_inflection("ったら", "る", &[], &["v5"]).into(),
                    suffix_inflection("んだら", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("んだら", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("んだら", "む", &[], &["v5"]).into(),
                    suffix_inflection("じたら", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("したら", "する", &[], &["vs"]).into(),
                    suffix_inflection("為たら", "為る", &[], &["vs"]).into(),
                    suffix_inflection("きたら", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来たら", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來たら", "來る", &[], &["vk"]).into(),
                ].into_iter().chain(irregular_verb_suffix_inflections(
                    IrregularVerbSuffix::たら,
                    &[],
                    &["v5"]
                ).into_iter().map(Into::into)).chain(std::iter::once(suffix_inflection("ましたら", "ます", &[], &["-ます"]).into())).collect(),
            },
        ),
        (
            "-たり",
            Transform {
                name: "-たり",
                description: Some(
                    "1. Shows two actions occurring back and forth (when used with two verbs).\n2. Shows examples of actions and states (when used with multiple verbs and adjectives).\nUsage: Attach たり to the continuative form (連用形) of verbs after euphonic change form, かったり to the stem of i-adjectives.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～たり",
                    description: Some("ある動作を例示的にあげることを表わす。"),
                }]),
                rules: vec![
                    suffix_inflection("かったり", "い", &[], &["adj-i"]).into(),
                    suffix_inflection("たり", "る", &[], &["v1"]).into(),
                    suffix_inflection("いたり", "く", &[], &["v5"]).into(),
                    suffix_inflection("いだり", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("したり", "す", &[], &["v5"]).into(),
                    suffix_inflection("ったり", "う", &[], &["v5"]).into(),
                    suffix_inflection("ったり", "つ", &[], &["v5"]).into(),
                    suffix_inflection("ったり", "る", &[], &["v5"]).into(),
                    suffix_inflection("んだり", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("んだり", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("んだり", "む", &[], &["v5"]).into(),
                    suffix_inflection("じたり", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("したり", "する", &[], &["vs"]).into(),
                    suffix_inflection("為たり", "為る", &[], &["vs"]).into(),
                    suffix_inflection("きたり", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来たり", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來たり", "來る", &[], &["vk"]).into(),
                ].into_iter().chain(irregular_verb_suffix_inflections(IrregularVerbSuffix::たり, &[], &["v5"]).into_iter().map(Into::into)).collect(),
            },
        ),
        (
            "-て",
            Transform {
                name: "-て",
                description: Some(
                    "て-form.\nIt has a myriad of meanings. Primarily, it is a conjunctive particle that connects two clauses together.\nUsage: Attach て to the continuative form (連用形) of verbs after euphonic change form, くて to the stem of i-adjectives.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～て",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("くて", "い", &["-て"], &["adj-i"]).into(),
                    suffix_inflection("て", "る", &["-て"], &["v1"]).into(),
                    suffix_inflection("いて", "く", &["-て"], &["v5"]).into(),
                    suffix_inflection("いで", "ぐ", &["-て"], &["v5"]).into(),
                    suffix_inflection("して", "す", &["-て"], &["v5"]).into(),
                    suffix_inflection("って", "う", &["-て"], &["v5"]).into(),
                    suffix_inflection("って", "つ", &["-て"], &["v5"]).into(),
                    suffix_inflection("って", "る", &["-て"], &["v5"]).into(),
                    suffix_inflection("んで", "ぬ", &["-て"], &["v5"]).into(),
                    suffix_inflection("んで", "ぶ", &["-て"], &["v5"]).into(),
                    suffix_inflection("んで", "む", &["-て"], &["v5"]).into(),
                    suffix_inflection("じて", "ずる", &["-て"], &["vz"]).into(),
                    suffix_inflection("して", "する", &["-て"], &["vs"]).into(),
                    suffix_inflection("為て", "為る", &["-て"], &["vs"]).into(),
                    suffix_inflection("きて", "くる", &["-て"], &["vk"]).into(),
                    suffix_inflection("来て", "来る", &["-て"], &["vk"]).into(),
                    suffix_inflection("來て", "來る", &["-て"], &["vk"]).into(),
                ].into_iter()
                    .chain(irregular_verb_suffix_inflections(
                        IrregularVerbSuffix::て,
                        &["-て"],
                        &["v5"]
                    ).into_iter().map(Into::into))
                    .chain(Vec::from_iter([suffix_inflection("まして", "ます", &[], &["-ます"]).into()]))
                    .collect(),
            },
        ),
        (
            "-ず",
            Transform {
                name: "-ず",
                description: Some(
                    "1. Negative form of verbs.\n2. Continuative form (連用形) of the particle ぬ (nu).\nUsage: Attach ず to the irrealis form (未然形) of verbs.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ず",
                    description: Some("～ない"),
                }]),
                rules: vec![
                    suffix_inflection("ず", "る", &[], &["v1"]).into(),
                    suffix_inflection("かず", "く", &[], &["v5"]).into(),
                    suffix_inflection("がず", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("さず", "す", &[], &["v5"]).into(),
                    suffix_inflection("たず", "つ", &[], &["v5"]).into(),
                    suffix_inflection("なず", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("ばず", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("まず", "む", &[], &["v5"]).into(),
                    suffix_inflection("らず", "る", &[], &["v5"]).into(),
                    suffix_inflection("わず", "う", &[], &["v5"]).into(),
                    suffix_inflection("ぜず", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("せず", "する", &[], &["vs"]).into(),
                    suffix_inflection("為ず", "為る", &[], &["vs"]).into(),
                    suffix_inflection("こず", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来ず", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來ず", "來る", &[], &["vk"]).into(),
                ],
            },
        ),
        (
            "-ぬ",
            Transform {
                name: "-ぬ",
                description: Some(
                    "Negative form of verbs.\nUsage: Attach ぬ to the irrealis form (未然形) of verbs.\nする becomes せぬ",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ぬ",
                    description: Some("～ない"),
                }]),
                rules: vec![
                    suffix_inflection("ぬ", "る", &[], &["v1"]).into(),
                    suffix_inflection("かぬ", "く", &[], &["v5"]).into(),
                    suffix_inflection("がぬ", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("さぬ", "す", &[], &["v5"]).into(),
                    suffix_inflection("たぬ", "つ", &[], &["v5"]).into(),
                    suffix_inflection("なぬ", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("ばぬ", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("まぬ", "む", &[], &["v5"]).into(),
                    suffix_inflection("らぬ", "る", &[], &["v5"]).into(),
                    suffix_inflection("わぬ", "う", &[], &["v5"]).into(),
                    suffix_inflection("ぜぬ", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("せぬ", "する", &[], &["vs"]).into(),
                    suffix_inflection("為ぬ", "為る", &[], &["vs"]).into(),
                    suffix_inflection("こぬ", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来ぬ", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來ぬ", "來る", &[], &["vk"]).into(),
                ],
            },
        ),
        (
            "-ん",
            Transform {
                name: "-ん",
                description: Some(
                    "Negative form of verbs; a sound change of ぬ.\nUsage: Attach ん to the irrealis form (未然形) of verbs.\nする becomes せん",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ん",
                    description: Some("～ない"),
                }]),
                rules: vec![
                    suffix_inflection("ん", "る", &["-ん"], &["v1"]).into(),
                    suffix_inflection("かん", "く", &["-ん"], &["v5"]).into(),
                    suffix_inflection("がん", "ぐ", &["-ん"], &["v5"]).into(),
                    suffix_inflection("さん", "す", &["-ん"], &["v5"]).into(),
                    suffix_inflection("たん", "つ", &["-ん"], &["v5"]).into(),
                    suffix_inflection("なん", "ぬ", &["-ん"], &["v5"]).into(),
                    suffix_inflection("ばん", "ぶ", &["-ん"], &["v5"]).into(),
                    suffix_inflection("まん", "む", &["-ん"], &["v5"]).into(),
                    suffix_inflection("らん", "る", &["-ん"], &["v5"]).into(),
                    suffix_inflection("わん", "う", &["-ん"], &["v5"]).into(),
                    suffix_inflection("ぜん", "ずる", &["-ん"], &["vz"]).into(),
                    suffix_inflection("せん", "する", &["-ん"], &["vs"]).into(),
                    suffix_inflection("為ん", "為る", &["-ん"], &["vs"]).into(),
                    suffix_inflection("こん", "くる", &["-ん"], &["vk"]).into(),
                    suffix_inflection("来ん", "来る", &["-ん"], &["vk"]).into(),
                    suffix_inflection("來ん", "來る", &["-ん"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-んばかり",
            Transform {
                name: "-んばかり",
                description: Some(
                    "Shows an action or condition is on the verge of occurring, or an excessive/extreme degree.\nUsage: Attach んばかり to the irrealis form (未然形) of verbs.\nする becomes せんばかり",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～んばかり",
                    description: Some("今にもそうなりそうな、しかし辛うじてそうなっていないようなさまを指す表現"),
                }]),
                rules: vec![
                    suffix_inflection("んばかり", "る", &[], &["v1"]).into(),
                    suffix_inflection("かんばかり", "く", &[], &["v5"]).into(),
                    suffix_inflection("がんばかり", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("さんばかり", "す", &[], &["v5"]).into(),
                    suffix_inflection("たんばかり", "つ", &[], &["v5"]).into(),
                    suffix_inflection("なんばかり", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("ばんばかり", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("まんばかり", "む", &[], &["v5"]).into(),
                    suffix_inflection("らんばかり", "る", &[], &["v5"]).into(),
                    suffix_inflection("わんばかり", "う", &[], &["v5"]).into(),
                    suffix_inflection("ぜんばかり", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("せんばかり", "する", &[], &["vs"]).into(),
                    suffix_inflection("為んばかり", "為る", &[], &["vs"]).into(),
                    suffix_inflection("こんばかり", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来んばかり", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來んばかり", "來る", &[], &["vk"]).into(),
                ],
            },
        ),
        (
            "-んとする",
            Transform {
                name: "-んとする",
                description: Some(
                    "1. Shows the speaker's will or intention.\n2. Shows an action or condition is on the verge of occurring.\nUsage: Attach んとする to the irrealis form (未然形) of verbs.\nする becomes せんとする",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～んとする",
                    description: Some("…しようとする、…しようとしている"),
                }]),
                rules: vec![
                    suffix_inflection("んとする", "る", &["vs"], &["v1"]).into(),
                    suffix_inflection("かんとする", "く", &["vs"], &["v5"]).into(),
                    suffix_inflection("がんとする", "ぐ", &["vs"], &["v5"]).into(),
                    suffix_inflection("さんとする", "す", &["vs"], &["v5"]).into(),
                    suffix_inflection("たんとする", "つ", &["vs"], &["v5"]).into(),
                    suffix_inflection("なんとする", "ぬ", &["vs"], &["v5"]).into(),
                    suffix_inflection("ばんとする", "ぶ", &["vs"], &["v5"]).into(),
                    suffix_inflection("まんとする", "む", &["vs"], &["v5"]).into(),
                    suffix_inflection("らんとする", "る", &["vs"], &["v5"]).into(),
                    suffix_inflection("わんとする", "う", &["vs"], &["v5"]).into(),
                    suffix_inflection("ぜんとする", "ずる", &["vs"], &["vz"]).into(),
                    suffix_inflection("せんとする", "する", &["vs"], &["vs"]).into(),
                    suffix_inflection("為んとする", "為る", &["vs"], &["vs"]).into(),
                    suffix_inflection("こんとする", "くる", &["vs"], &["vk"]).into(),
                    suffix_inflection("来んとする", "来る", &["vs"], &["vk"]).into(),
                    suffix_inflection("來んとする", "來る", &["vs"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-む",
            Transform {
                name: "-む",
                description: Some(
                    "Archaic.\n1. Shows an inference of a certain matter.\n2. Shows speaker's intention.\nUsage: Attach む to the irrealis form (未然形) of verbs.\nする becomes せむ",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～む",
                    description: Some("…だろう"),
                }]),
                rules: vec![
                    suffix_inflection("む", "る", &[], &["v1"]).into(),
                    suffix_inflection("かむ", "く", &[], &["v5"]).into(),
                    suffix_inflection("がむ", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("さむ", "す", &[], &["v5"]).into(),
                    suffix_inflection("たむ", "つ", &[], &["v5"]).into(),
                    suffix_inflection("なむ", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("ばむ", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("まむ", "む", &[], &["v5"]).into(),
                    suffix_inflection("らむ", "る", &[], &["v5"]).into(),
                    suffix_inflection("わむ", "う", &[], &["v5"]).into(),
                    suffix_inflection("ぜむ", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("せむ", "する", &[], &["vs"]).into(),
                    suffix_inflection("為む", "為る", &[], &["vs"]).into(),
                    suffix_inflection("こむ", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来む", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來む", "來る", &[], &["vk"]).into(),
                ],
            },
        ),
        (
            "-ざる",
            Transform {
                name: "-ざる",
                description: Some(
                    "Negative form of verbs.\nUsage: Attach ざる to the irrealis form (未然形) of verbs.\nする becomes せざる",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ざる",
                    description: Some("…ない…"),
                }]),
                rules: vec![
                    suffix_inflection("ざる", "る", &[], &["v1"]).into(),
                    suffix_inflection("かざる", "く", &[], &["v5"]).into(),
                    suffix_inflection("がざる", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("さざる", "す", &[], &["v5"]).into(),
                    suffix_inflection("たざる", "つ", &[], &["v5"]).into(),
                    suffix_inflection("なざる", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("ばざる", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("まざる", "む", &[], &["v5"]).into(),
                    suffix_inflection("らざる", "る", &[], &["v5"]).into(),
                    suffix_inflection("わざる", "う", &[], &["v5"]).into(),
                    suffix_inflection("ぜざる", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("せざる", "する", &[], &["vs"]).into(),
                    suffix_inflection("為ざる", "為る", &[], &["vs"]).into(),
                    suffix_inflection("こざる", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来ざる", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來ざる", "來る", &[], &["vk"]).into(),
                ],
            },
        ),
        (
            "-ねば",
            Transform {
                name: "-ねば",
                description: Some(
                    "1. Shows a hypothetical negation; if not ...\n2. Shows a must. Used with or without ならぬ.\nUsage: Attach ねば to the irrealis form (未然形) of verbs.\nする becomes せねば",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ねば",
                    description: Some("もし…ないなら。…なければならない。"),
                }]),
                rules: vec![
                    suffix_inflection("ねば", "る", &["-ば"], &["v1"]).into(),
                    suffix_inflection("かねば", "く", &["-ば"], &["v5"]).into(),
                    suffix_inflection("がねば", "ぐ", &["-ば"], &["v5"]).into(),
                    suffix_inflection("さねば", "す", &["-ば"], &["v5"]).into(),
                    suffix_inflection("たねば", "つ", &["-ば"], &["v5"]).into(),
                    suffix_inflection("なねば", "ぬ", &["-ば"], &["v5"]).into(),
                    suffix_inflection("ばねば", "ぶ", &["-ば"], &["v5"]).into(),
                    suffix_inflection("まねば", "む", &["-ば"], &["v5"]).into(),
                    suffix_inflection("らねば", "る", &["-ば"], &["v5"]).into(),
                    suffix_inflection("わねば", "う", &["-ば"], &["v5"]).into(),
                    suffix_inflection("ぜねば", "ずる", &["-ば"], &["vz"]).into(),
                    suffix_inflection("せねば", "する", &["-ば"], &["vs"]).into(),
                    suffix_inflection("為ねば", "為る", &["-ば"], &["vs"]).into(),
                    suffix_inflection("こねば", "くる", &["-ば"], &["vk"]).into(),
                    suffix_inflection("来ねば", "来る", &["-ば"], &["vk"]).into(),
                    suffix_inflection("來ねば", "來る", &["-ば"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-く",
            Transform {
                name: "-く",
                description: Some(
                    "Adverbial form of i-adjectives.\n",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～く",
                    description: Some("〔形容詞で〕用言へ続く。例、「大きく育つ」の「大きく」。"),
                }]),
                rules: vec![
                    suffix_inflection("く", "い", &["-く"], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "causative",
            Transform {
                name: "causative",
                description: Some(
                    "Describes the intention to make someone do something.\nUsage: Attach させる to the irrealis form (未然形) of ichidan verbs and くる.\nAttach せる to the irrealis form (未然形) of godan verbs and する.\nIt itself conjugates as an ichidan verb.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～せる・させる",
                    description: Some("だれかにある行為をさせる意を表わす時の言い方。例、「行かせる」の「せる」。"),
                }]),
                rules: vec![
                    suffix_inflection("させる", "る", &["v1"], &["v1"]).into(),
                    suffix_inflection("かせる", "く", &["v1"], &["v5"]).into(),
                    suffix_inflection("がせる", "ぐ", &["v1"], &["v5"]).into(),
                    suffix_inflection("させる", "す", &["v1"], &["v5"]).into(),
                    suffix_inflection("たせる", "つ", &["v1"], &["v5"]).into(),
                    suffix_inflection("なせる", "ぬ", &["v1"], &["v5"]).into(),
                    suffix_inflection("ばせる", "ぶ", &["v1"], &["v5"]).into(),
                    suffix_inflection("ませる", "む", &["v1"], &["v5"]).into(),
                    suffix_inflection("らせる", "る", &["v1"], &["v5"]).into(),
                    suffix_inflection("わせる", "う", &["v1"], &["v5"]).into(),
                    suffix_inflection("じさせる", "ずる", &["v1"], &["vz"]).into(),
                    suffix_inflection("ぜさせる", "ずる", &["v1"], &["vz"]).into(),
                    suffix_inflection("させる", "する", &["v1"], &["vs"]).into(),
                    suffix_inflection("為せる", "為る", &["v1"], &["vs"]).into(),
                    suffix_inflection("せさせる", "する", &["v1"], &["vs"]).into(),
                    suffix_inflection("為させる", "為る", &["v1"], &["vs"]).into(),
                    suffix_inflection("こさせる", "くる", &["v1"], &["vk"]).into(),
                    suffix_inflection("来させる", "来る", &["v1"], &["vk"]).into(),
                    suffix_inflection("來させる", "來る", &["v1"], &["vk"]).into(),
                ],
            },
        ),
        (
            "short causative",
            Transform {
                name: "short causative",
                description: Some(
                    "Contraction of the causative form.\nDescribes the intention to make someone do something.\nUsage: Attach す to the irrealis form (未然形) of godan verbs.\nAttach さす to the dictionary form (終止形) of ichidan verbs.\nする becomes さす, くる becomes こさす.\nIt itself conjugates as an godan verb.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～す・さす",
                    description: Some("だれかにある行為をさせる意を表わす時の言い方。例、「食べさす」の「さす」。"),
                }]),
                rules: vec![
                    suffix_inflection("さす", "る", &["v5ss"], &["v1"]).into(),
                    suffix_inflection("かす", "く", &["v5sp"], &["v5"]).into(),
                    suffix_inflection("がす", "ぐ", &["v5sp"], &["v5"]).into(),
                    suffix_inflection("さす", "す", &["v5ss"], &["v5"]).into(),
                    suffix_inflection("たす", "つ", &["v5sp"], &["v5"]).into(),
                    suffix_inflection("なす", "ぬ", &["v5sp"], &["v5"]).into(),
                    suffix_inflection("ばす", "ぶ", &["v5sp"], &["v5"]).into(),
                    suffix_inflection("ます", "む", &["v5sp"], &["v5"]).into(),
                    suffix_inflection("らす", "る", &["v5sp"], &["v5"]).into(),
                    suffix_inflection("わす", "う", &["v5sp"], &["v5"]).into(),
                    suffix_inflection("じさす", "ずる", &["v5ss"], &["vz"]).into(),
                    suffix_inflection("ぜさす", "ずる", &["v5ss"], &["vz"]).into(),
                    suffix_inflection("さす", "する", &["v5ss"], &["vs"]).into(),
                    suffix_inflection("為す", "為る", &["v5ss"], &["vs"]).into(),
                    suffix_inflection("こさす", "くる", &["v5ss"], &["vk"]).into(),
                    suffix_inflection("来さす", "来る", &["v5ss"], &["vk"]).into(),
                    suffix_inflection("來さす", "來る", &["v5ss"], &["vk"]).into(),
                ],
            },
        ),
        (
            "imperative",
            Transform {
                name: "imperative",
                description: Some(
                    "1. To give orders.\n2. (As あれ) Represents the fact that it will never change no matter the circumstances.\n3. Express a feeling of hope.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "命令形",
                    description: Some("命令の意味を表わすときの形。例、「行け」。"),
                }]),
                rules: vec![
                    suffix_inflection("ろ", "る", &[], &["v1"]).into(),
                    suffix_inflection("よ", "る", &[], &["v1"]).into(),
                    suffix_inflection("え", "う", &[], &["v5"]).into(),
                    suffix_inflection("け", "く", &[], &["v5"]).into(),
                    suffix_inflection("げ", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("せ", "す", &[], &["v5"]).into(),
                    suffix_inflection("て", "つ", &[], &["v5"]).into(),
                    suffix_inflection("ね", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("べ", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("め", "む", &[], &["v5"]).into(),
                    suffix_inflection("れ", "る", &[], &["v5"]).into(),
                    suffix_inflection("じろ", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("ぜよ", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("しろ", "する", &[], &["vs"]).into(),
                    suffix_inflection("せよ", "する", &[], &["vs"]).into(),
                    suffix_inflection("為ろ", "為る", &[], &["vs"]).into(),
                    suffix_inflection("為よ", "為る", &[], &["vs"]).into(),
                    suffix_inflection("こい", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来い", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來い", "來る", &[], &["vk"]).into(),
                ],
            },
        ),
        (
            "continuative",
            Transform {
                name: "continuative",
                description: Some(
                    "Used to indicate actions that are (being) carried out.\nRefers to 連用形, the part of the verb after conjugating with -ます and dropping ます.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "連用形",
                    description: Some("〔動詞などで〕「ます」などに続く。例、「バスを降りて歩きます」の「降り」「歩き」。"),
                }]),
                rules: vec![
                    suffix_inflection("い", "いる", &[], &["v1d"]).into(),
                    suffix_inflection("え", "える", &[], &["v1d"]).into(),
                    suffix_inflection("き", "きる", &[], &["v1d"]).into(),
                    suffix_inflection("ぎ", "ぎる", &[], &["v1d"]).into(),
                    suffix_inflection("け", "ける", &[], &["v1d"]).into(),
                    suffix_inflection("げ", "げる", &[], &["v1d"]).into(),
                    suffix_inflection("じ", "じる", &[], &["v1d"]).into(),
                    suffix_inflection("せ", "せる", &[], &["v1d"]).into(),
                    suffix_inflection("ぜ", "ぜる", &[], &["v1d"]).into(),
                    suffix_inflection("ち", "ちる", &[], &["v1d"]).into(),
                    suffix_inflection("て", "てる", &[], &["v1d"]).into(),
                    suffix_inflection("で", "でる", &[], &["v1d"]).into(),
                    suffix_inflection("に", "にる", &[], &["v1d"]).into(),
                    suffix_inflection("ね", "ねる", &[], &["v1d"]).into(),
                    suffix_inflection("ひ", "ひる", &[], &["v1d"]).into(),
                    suffix_inflection("び", "びる", &[], &["v1d"]).into(),
                    suffix_inflection("へ", "へる", &[], &["v1d"]).into(),
                    suffix_inflection("べ", "べる", &[], &["v1d"]).into(),
                    suffix_inflection("み", "みる", &[], &["v1d"]).into(),
                    suffix_inflection("め", "める", &[], &["v1d"]).into(),
                    suffix_inflection("り", "りる", &[], &["v1d"]).into(),
                    suffix_inflection("れ", "れる", &[], &["v1d"]).into(),
                    suffix_inflection("い", "う", &[], &["v5"]).into(),
                    suffix_inflection("き", "く", &[], &["v5"]).into(),
                    suffix_inflection("ぎ", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("し", "す", &[], &["v5"]).into(),
                    suffix_inflection("ち", "つ", &[], &["v5"]).into(),
                    suffix_inflection("に", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("び", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("み", "む", &[], &["v5"]).into(),
                    suffix_inflection("り", "る", &[], &["v5"]).into(),
                    suffix_inflection("き", "くる", &[], &["vk"]).into(),
                    suffix_inflection("し", "する", &[], &["vs"]).into(),
                    suffix_inflection("来", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來", "來る", &[], &["vk"]).into(),
                ],
            },
        ),
        (
            "negative",
            Transform {
                name: "negative",
                description: Some(
                    "1. Negative form of verbs.\n2. Expresses a feeling of solicitation to the other party.\nUsage: Attach ない to the irrealis form (未然形) of verbs, くない to the stem of i-adjectives. ない itself conjugates as i-adjective. ます becomes ません.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ない",
                    description: Some("その動作・作用・状態の成立を否定することを表わす。"),
                }]),
                rules: vec![
                    suffix_inflection("くない", "い", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("ない", "る", &["adj-i"], &["v1"]).into(),
                    suffix_inflection("かない", "く", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("がない", "ぐ", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("さない", "す", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("たない", "つ", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("なない", "ぬ", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("ばない", "ぶ", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("まない", "む", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("らない", "る", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("わない", "う", &["adj-i"], &["v5"]).into(),
                    suffix_inflection("じない", "ずる", &["adj-i"], &["vz"]).into(),
                    suffix_inflection("しない", "する", &["adj-i"], &["vs"]).into(),
                    suffix_inflection("為ない", "為る", &["adj-i"], &["vs"]).into(),
                    suffix_inflection("こない", "くる", &["adj-i"], &["vk"]).into(),
                    suffix_inflection("来ない", "来る", &["adj-i"], &["vk"]).into(),
                    suffix_inflection("來ない", "來る", &["adj-i"], &["vk"]).into(),
                    suffix_inflection("ません", "ます", &["-ません"], &["-ます"]).into(),
                ],
            },
        ),
        (
            "-さ",
            Transform {
                name: "-さ",
                description: Some(
                    "Nominalizing suffix of i-adjectives indicating nature, state, mind or degree.\nUsage: Attach さ to the stem of i-adjectives.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～さ",
                    description: Some("こと。程度。"),
                }]),
                rules: vec![
                    suffix_inflection("さ", "い", &[], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "passive",
            Transform {
                name: "passive",
                description: Some(
                    "Indicates that the subject is affected by the action of the verb.\nUsage: Attach れる to the irrealis form (未然形) of godan verbs.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～れる",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("かれる", "く", &["v1"], &["v5"]).into(),
                    suffix_inflection("がれる", "ぐ", &["v1"], &["v5"]).into(),
                    suffix_inflection("される", "す", &["v1"], &["v5d", "v5sp"]).into(),
                    suffix_inflection("たれる", "つ", &["v1"], &["v5"]).into(),
                    suffix_inflection("なれる", "ぬ", &["v1"], &["v5"]).into(),
                    suffix_inflection("ばれる", "ぶ", &["v1"], &["v5"]).into(),
                    suffix_inflection("まれる", "む", &["v1"], &["v5"]).into(),
                    suffix_inflection("われる", "う", &["v1"], &["v5"]).into(),
                    suffix_inflection("られる", "る", &["v1"], &["v5"]).into(),
                    suffix_inflection("じされる", "ずる", &["v1"], &["vz"]).into(),
                    suffix_inflection("ぜされる", "ずる", &["v1"], &["vz"]).into(),
                    suffix_inflection("される", "する", &["v1"], &["vs"]).into(),
                    suffix_inflection("為れる", "為る", &["v1"], &["vs"]).into(),
                    suffix_inflection("こられる", "くる", &["v1"], &["vk"]).into(),
                    suffix_inflection("来られる", "来る", &["v1"], &["vk"]).into(),
                    suffix_inflection("來られる", "來る", &["v1"], &["vk"]).into(),
                ],
            },
        ),
        (
            "-た",
            Transform {
                name: "-た",
                description: Some(
                    "1. Indicates a reality that has happened in the past.\n2. Indicates the completion of an action.\n3. Indicates the confirmation of a matter.\n4. Indicates the speaker's confidence that the action will definitely be fulfilled.\n5. Indicates the events that occur before the main clause are represented as relative past.\n6. Indicates a mild imperative/command.\nUsage: Attach た to the continuative form (連用形) of verbs after euphonic change form, かった to the stem of i-adjectives.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～た",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("かった", "い", &["-た"], &["adj-i"]).into(),
                    suffix_inflection("た", "る", &["-た"], &["v1"]).into(),
                    suffix_inflection("いた", "く", &["-た"], &["v5"]).into(),
                    suffix_inflection("いだ", "ぐ", &["-た"], &["v5"]).into(),
                    suffix_inflection("した", "す", &["-た"], &["v5"]).into(),
                    suffix_inflection("った", "う", &["-た"], &["v5"]).into(),
                    suffix_inflection("った", "つ", &["-た"], &["v5"]).into(),
                    suffix_inflection("った", "る", &["-た"], &["v5"]).into(),
                    suffix_inflection("んだ", "ぬ", &["-た"], &["v5"]).into(),
                    suffix_inflection("んだ", "ぶ", &["-た"], &["v5"]).into(),
                    suffix_inflection("んだ", "む", &["-た"], &["v5"]).into(),
                    suffix_inflection("じた", "ずる", &["-た"], &["vz"]).into(),
                    suffix_inflection("した", "する", &["-た"], &["vs"]).into(),
                    suffix_inflection("為た", "為る", &["-た"], &["vs"]).into(),
                    suffix_inflection("きた", "くる", &["-た"], &["vk"]).into(),
                    suffix_inflection("来た", "来る", &["-た"], &["vk"]).into(),
                    suffix_inflection("來た", "來る", &["-た"], &["vk"]).into(),
                ]
                .into_iter()
                .chain(irregular_verb_suffix_inflections(IrregularVerbSuffix::た, &["-た"], &["v5"]).into_iter().map(Into::into))
                .chain([
                    suffix_inflection("ました", "ます", &["-た"], &["-ます"]).into(),
                    suffix_inflection("でした", "", &["-た"], &["-ません"]).into(),
                    suffix_inflection("かった", "", &["-た"], &["-ません", "-ん"]).into()
                ]).collect(),
            },
        ),
        (
            "-ます",
            Transform {
                name: "-ます",
                description: Some(
                    "Polite conjugation of verbs and adjectives.\nUsage: Attach ます to the continuative form (連用形) of verbs.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～ます",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("ます", "る", &["-ます"], &["v1"]).into(),
                    suffix_inflection("います", "う", &["-ます"], &["v5d"]).into(),
                    suffix_inflection("きます", "く", &["-ます"], &["v5d"]).into(),
                    suffix_inflection("ぎます", "ぐ", &["-ます"], &["v5d"]).into(),
                    suffix_inflection("します", "す", &["-ます"], &["v5d", "v5s"]).into(),
                    suffix_inflection("ちます", "つ", &["-ます"], &["v5d"]).into(),
                    suffix_inflection("にます", "ぬ", &["-ます"], &["v5d"]).into(),
                    suffix_inflection("びます", "ぶ", &["-ます"], &["v5d"]).into(),
                    suffix_inflection("みます", "む", &["-ます"], &["v5d"]).into(),
                    suffix_inflection("ります", "る", &["-ます"], &["v5d"]).into(),
                    suffix_inflection("じます", "ずる", &["-ます"], &["vz"]).into(),
                    suffix_inflection("します", "する", &["-ます"], &["vs"]).into(),
                    suffix_inflection("為ます", "為る", &["-ます"], &["vs"]).into(),
                    suffix_inflection("きます", "くる", &["-ます"], &["vk"]).into(),
                    suffix_inflection("来ます", "来る", &["-ます"], &["vk"]).into(),
                    suffix_inflection("來ます", "來る", &["-ます"], &["vk"]).into(),
                    suffix_inflection("くあります", "い", &["-ます"], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "potential",
            Transform {
                name: "potential",
                description: Some(
                    "Indicates a state of being (naturally) capable of doing an action.\nUsage: Attach (ら)れる to the irrealis form (未然形) of ichidan verbs.\nAttach る to the imperative form (命令形) of godan verbs.\nする becomes できる, くる becomes こ(ら)れる.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～(ら)れる",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("れる", "る", &["v1"], &["v1", "v5d"]).into(),
                    suffix_inflection("える", "う", &["v1"], &["v5d"]).into(),
                    suffix_inflection("ける", "く", &["v1"], &["v5d"]).into(),
                    suffix_inflection("げる", "ぐ", &["v1"], &["v5d"]).into(),
                    suffix_inflection("せる", "す", &["v1"], &["v5d"]).into(),
                    suffix_inflection("てる", "つ", &["v1"], &["v5d"]).into(),
                    suffix_inflection("ねる", "ぬ", &["v1"], &["v5d"]).into(),
                    suffix_inflection("べる", "ぶ", &["v1"], &["v5d"]).into(),
                    suffix_inflection("める", "む", &["v1"], &["v5d"]).into(),
                    suffix_inflection("できる", "する", &["v1"], &["vs"]).into(),
                    suffix_inflection("出来る", "する", &["v1"], &["vs"]).into(),
                    suffix_inflection("これる", "くる", &["v1"], &["vk"]).into(),
                    suffix_inflection("来れる", "来る", &["v1"], &["vk"]).into(),
                    suffix_inflection("來れる", "來る", &["v1"], &["vk"]).into(),
                ],
            },
        ),
        (
            "potential or passive",
            Transform {
                name: "potential or passive",
                description: Some(
                    "Indicates that the subject is affected by the action of the verb.\n3. Indicates a state of being (naturally) capable of doing an action.\nUsage: Attach られる to the irrealis form (未然形) of ichidan verbs.\nする becomes せられる, くる becomes こられる.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～られる",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("られる", "る", &["v1"], &["v1"]).into(),
                    suffix_inflection("ざれる", "ずる", &["v1"], &["vz"]).into(),
                    suffix_inflection("ぜられる", "ずる", &["v1"], &["vz"]).into(),
                    suffix_inflection("せられる", "する", &["v1"], &["vs"]).into(),
                    suffix_inflection("為られる", "為る", &["v1"], &["vs"]).into(),
                    suffix_inflection("こられる", "くる", &["v1"], &["vk"]).into(),
                    suffix_inflection("来られる", "来る", &["v1"], &["vk"]).into(),
                    suffix_inflection("來られる", "來る", &["v1"], &["vk"]).into(),
                ],
            },
        ),
        (
            "volitional",
            Transform {
                name: "volitional",
                description: Some(
                    "1. Expresses speaker's will or intention.\n2. Expresses an invitation to the other party.\n3. (Used in …ようとする) Indicates being on the verge of initiating an action or transforming a state.\n4. Indicates an inference of a matter.\nUsage: Attach よう to the irrealis form (未然形) of ichidan verbs.\nAttach う to the irrealis form (未然形) of godan verbs after -o euphonic change form.\nAttach かろう to the stem of i-adjectives (4th meaning only).",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～う・よう",
                    description: Some("主体の意志を表わす"),
                }]),
                rules: vec![
                    suffix_inflection("よう", "る", &[], &["v1"]).into(),
                    suffix_inflection("おう", "う", &[], &["v5"]).into(),
                    suffix_inflection("こう", "く", &[], &["v5"]).into(),
                    suffix_inflection("ごう", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("そう", "す", &[], &["v5"]).into(),
                    suffix_inflection("とう", "つ", &[], &["v5"]).into(),
                    suffix_inflection("のう", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("ぼう", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("もう", "む", &[], &["v5"]).into(),
                    suffix_inflection("ろう", "る", &[], &["v5"]).into(),
                    suffix_inflection("じよう", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("しよう", "する", &[], &["vs"]).into(),
                    suffix_inflection("為よう", "為る", &[], &["vs"]).into(),
                    suffix_inflection("こよう", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来よう", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來よう", "來る", &[], &["vk"]).into(),
                    suffix_inflection("ましょう", "ます", &[], &["-ます"]).into(),
                    suffix_inflection("かろう", "い", &[], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "volitional slang",
            Transform {
                name: "volitional slang",
                description: Some(
                    "Contraction of volitional form + か\n1. Expresses speaker's will or intention.\n2. Expresses an invitation to the other party.\nUsage: Replace final う with っ of volitional form then add か.\nFor example: 行こうか -> 行こっか.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～っか・よっか",
                    description: Some("「うか・ようか」の短縮"),
                }]),
                rules: vec![
                    suffix_inflection("よっか", "る", &[], &["v1"]).into(),
                    suffix_inflection("おっか", "う", &[], &["v5"]).into(),
                    suffix_inflection("こっか", "く", &[], &["v5"]).into(),
                    suffix_inflection("ごっか", "ぐ", &[], &["v5"]).into(),
                    suffix_inflection("そっか", "す", &[], &["v5"]).into(),
                    suffix_inflection("とっか", "つ", &[], &["v5"]).into(),
                    suffix_inflection("のっか", "ぬ", &[], &["v5"]).into(),
                    suffix_inflection("ぼっか", "ぶ", &[], &["v5"]).into(),
                    suffix_inflection("もっか", "む", &[], &["v5"]).into(),
                    suffix_inflection("ろっか", "る", &[], &["v5"]).into(),
                    suffix_inflection("じよっか", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("しよっか", "する", &[], &["vs"]).into(),
                    suffix_inflection("為よっか", "為る", &[], &["vs"]).into(),
                    suffix_inflection("こよっか", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来よっか", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來よっか", "來る", &[], &["vk"]).into(),
                    suffix_inflection("ましょっか", "ます", &[], &["-ます"]).into(),
                ],
            },
        ),
        (
            "-まい",
            Transform {
                name: "-まい",
                description: Some(
                    "Negative volitional form of verbs.\n1. Expresses speaker's assumption that something is likely not true.\n2. Expresses speaker's will or intention not to do something.\nUsage: Attach まい to the dictionary form (終止形) of verbs.\nAttach まい to the irrealis form (未然形) of ichidan verbs.\nする becomes しまい, くる becomes こまい.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～まい",
                    description: Some(
                        "1. 打うち消けしの推量すいりょう 「～ないだろう」と想像する\n2. 打うち消けしの意志いし「～ないつもりだ」という気持ち",
                    ),
                }]),
                rules: vec![
                    suffix_inflection("まい", "", &[], &["v"]).into(),
                    suffix_inflection("まい", "る", &[], &["v1"]).into(),
                    suffix_inflection("じまい", "ずる", &[], &["vz"]).into(),
                    suffix_inflection("しまい", "する", &[], &["vs"]).into(),
                    suffix_inflection("為まい", "為る", &[], &["vs"]).into(),
                    suffix_inflection("こまい", "くる", &[], &["vk"]).into(),
                    suffix_inflection("来まい", "来る", &[], &["vk"]).into(),
                    suffix_inflection("來まい", "來る", &[], &["vk"]).into(),
                    suffix_inflection("まい", "", &[], &["-ます"]).into(),
                ],
            },
        ),
        (
            "-おく",
            Transform {
                name: "-おく",
                description: Some(
                    "To do certain things in advance in preparation (or in anticipation) of latter needs.\nUsage: Attach おく to the て-form of verbs.\nAttach でおく after ない negative form of verbs.\nContracts to とく・どく in speech.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～おく",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("ておく", "て", &["v5"], &["-て"]).into(),
                    suffix_inflection("でおく", "で", &["v5"], &["-て"]).into(),
                    suffix_inflection("とく", "て", &["v5"], &["-て"]).into(),
                    suffix_inflection("どく", "で", &["v5"], &["-て"]).into(),
                    suffix_inflection("ないでおく", "ない", &["v5"], &["adj-i"]).into(),
                    suffix_inflection("ないどく", "ない", &["v5"], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "-いる",
            Transform {
                name: "-いる",
                description: Some(
                    "1. Indicates an action continues or progresses to a point in time.\n2. Indicates an action is completed and remains as is.\n3. Indicates a state or condition that can be taken to be the result of undergoing some change.\nUsage: Attach いる to the て-form of verbs. い can be dropped in speech.\nAttach でいる after ない negative form of verbs.\n(Slang) Attach おる to the て-form of verbs. Contracts to とる・でる in speech.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～いる",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("ている", "て", &["v1"], &["-て"]).into(),
                    suffix_inflection("ておる", "て", &["v5"], &["-て"]).into(),
                    suffix_inflection("てる", "て", &["v1p"], &["-て"]).into(),
                    suffix_inflection("でいる", "で", &["v1"], &["-て"]).into(),
                    suffix_inflection("でおる", "で", &["v5"], &["-て"]).into(),
                    suffix_inflection("でる", "で", &["v1p"], &["-て"]).into(),
                    suffix_inflection("とる", "て", &["v5"], &["-て"]).into(),
                    suffix_inflection("ないでいる", "ない", &["v1"], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "-き",
            Transform {
                name: "-き",
                description: Some(
                    "Attributive form (連体形) of i-adjectives. An archaic form that remains in modern Japanese.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～き",
                    description: Some("連体形"),
                }]),
                rules: vec![
                    suffix_inflection("き", "い", &[], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "-げ",
            Transform {
                name: "-げ",
                description: Some(
                    "Describes a person's appearance. Shows feelings of the person.\nUsage: Attach げ or 気 to the stem of i-adjectives.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～げ",
                    description: Some("…でありそうな様子。いかにも…らしいさま。"),
                }]),
                rules: vec![
                    suffix_inflection("げ", "い", &[], &["adj-i"]).into(),
                    suffix_inflection("気", "い", &[], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "-がる",
            Transform {
                name: "-がる",
                description: Some(
                    "1. Shows subject’s feelings contrast with what is thought/known about them.\n2. Indicates subject's behavior (stands out).\nUsage: Attach がる to the stem of i-adjectives. It itself conjugates as a godan verb.",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～がる",
                    description: Some("いかにもその状態にあるという印象を相手に与えるような言動をする。"),
                }]),
                rules: vec![
                    suffix_inflection("がる", "い", &["v5"], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "-え",
            Transform {
                name: "-え",
                description: Some(
                    "Slang. A sound change of i-adjectives.\nai：やばい → やべぇ\nui：さむい → さみぃ/さめぇ\noi：すごい → すげぇ",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～え",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("ねえ", "ない", &[], &["adj-i"]).into(),
                    suffix_inflection("めえ", "むい", &[], &["adj-i"]).into(),
                    suffix_inflection("みい", "むい", &[], &["adj-i"]).into(),
                    suffix_inflection("ちぇえ", "つい", &[], &["adj-i"]).into(),
                    suffix_inflection("ちい", "つい", &[], &["adj-i"]).into(),
                    suffix_inflection("せえ", "すい", &[], &["adj-i"]).into(),
                    suffix_inflection("ええ", "いい", &[], &["adj-i"]).into(),
                    suffix_inflection("ええ", "わい", &[], &["adj-i"]).into(),
                    suffix_inflection("ええ", "よい", &[], &["adj-i"]).into(),
                    suffix_inflection("いぇえ", "よい", &[], &["adj-i"]).into(),
                    suffix_inflection("うぇえ", "わい", &[], &["adj-i"]).into(),
                    suffix_inflection("けえ", "かい", &[], &["adj-i"]).into(),
                    suffix_inflection("げえ", "がい", &[], &["adj-i"]).into(),
                    suffix_inflection("げえ", "ごい", &[], &["adj-i"]).into(),
                    suffix_inflection("せえ", "さい", &[], &["adj-i"]).into(),
                    suffix_inflection("めえ", "まい", &[], &["adj-i"]).into(),
                    suffix_inflection("ぜえ", "ずい", &[], &["adj-i"]).into(),
                    suffix_inflection("っぜえ", "ずい", &[], &["adj-i"]).into(),
                    suffix_inflection("れえ", "らい", &[], &["adj-i"]).into(),
                    suffix_inflection("れえ", "らい", &[], &["adj-i"]).into(),
                    suffix_inflection("ちぇえ", "ちゃい", &[], &["adj-i"]).into(),
                    suffix_inflection("でえ", "どい", &[], &["adj-i"]).into(),
                    suffix_inflection("れえ", "れい", &[], &["adj-i"]).into(),
                    suffix_inflection("べえ", "ばい", &[], &["adj-i"]).into(),
                    suffix_inflection("てえ", "たい", &[], &["adj-i"]).into(),
                    suffix_inflection("ねぇ", "ない", &[], &["adj-i"]).into(),
                    suffix_inflection("めぇ", "むい", &[], &["adj-i"]).into(),
                    suffix_inflection("みぃ", "むい", &[], &["adj-i"]).into(),
                    suffix_inflection("ちぃ", "つい", &[], &["adj-i"]).into(),
                    suffix_inflection("せぇ", "すい", &[], &["adj-i"]).into(),
                    suffix_inflection("けぇ", "かい", &[], &["adj-i"]).into(),
                    suffix_inflection("げぇ", "がい", &[], &["adj-i"]).into(),
                    suffix_inflection("げぇ", "ごい", &[], &["adj-i"]).into(),
                    suffix_inflection("せぇ", "さい", &[], &["adj-i"]).into(),
                    suffix_inflection("めぇ", "まい", &[], &["adj-i"]).into(),
                    suffix_inflection("ぜぇ", "ずい", &[], &["adj-i"]).into(),
                    suffix_inflection("っぜぇ", "ずい", &[], &["adj-i"]).into(),
                    suffix_inflection("れぇ", "らい", &[], &["adj-i"]).into(),
                    suffix_inflection("でぇ", "どい", &[], &["adj-i"]).into(),
                    suffix_inflection("れぇ", "れい", &[], &["adj-i"]).into(),
                    suffix_inflection("べぇ", "ばい", &[], &["adj-i"]).into(),
                    suffix_inflection("てぇ", "たい", &[], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "n-slang",
            Transform {
                name: "n-slang",
                description: Some(
                    "Slang sound change of r-column syllables to n (when before an n-sound, usually の or な)",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～んな",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("んなさい", "りなさい", &[], &["-なさい"]).into(),
                    suffix_inflection("らんない", "られない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("んない", "らない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("んなきゃ", "らなきゃ", &[], &["-ゃ"]).into(),
                    suffix_inflection("んなきゃ", "れなきゃ", &[], &["-ゃ"]).into(),
                ],
            },
        ),
        (
            "imperative negative slang",
            Transform {
                name: "imperative negative slang",
                description: None,
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "～んな",
                    description: None,
                }]),
                rules: vec![
                    suffix_inflection("んな", "る", &[], &["v"]).into(),
                ],
            },
        ),
        (
            "kansai-ben negative",
            Transform {
                name: "kansai-ben negative",
                description: Some(
                    "Negative form of kansai-ben verbs",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "関西弁",
                    description: Some("～ない (関西弁)"),
                }]),
                rules: vec![
                    suffix_inflection("へん", "ない", &[], &["adj-i"]).into(),
                    suffix_inflection("ひん", "ない", &[], &["adj-i"]).into(),
                    suffix_inflection("せえへん", "しない", &[], &["adj-i"]).into(),
                    suffix_inflection("へんかった", "なかった", &["-た"], &["-た"]).into(),
                    suffix_inflection("ひんかった", "なかった", &["-た"], &["-た"]).into(),
                    suffix_inflection("うてへん", "ってない", &[], &["adj-i"]).into(),
                ],
            },
        ),
        (
            "kansai-ben -て",
            Transform {
                name: "kansai-ben -て",
                description: Some(
                    "-て form of kansai-ben verbs",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "関西弁",
                    description: Some("～て (関西弁)"),
                }]),
                rules: vec![
                    suffix_inflection("うて", "って", &["-て"], &["-て"]).into(),
                    suffix_inflection("おうて", "あって", &["-て"], &["-て"]).into(),
                    suffix_inflection("こうて", "かって", &["-て"], &["-て"]).into(),
                    suffix_inflection("ごうて", "がって", &["-て"], &["-て"]).into(),
                    suffix_inflection("そうて", "さって", &["-て"], &["-て"]).into(),
                    suffix_inflection("ぞうて", "ざって", &["-て"], &["-て"]).into(),
                    suffix_inflection("とうて", "たって", &["-て"], &["-て"]).into(),
                    suffix_inflection("どうて", "だって", &["-て"], &["-て"]).into(),
                    suffix_inflection("のうて", "なって", &["-て"], &["-て"]).into(),
                    suffix_inflection("ほうて", "はって", &["-て"], &["-て"]).into(),
                    suffix_inflection("ぼうて", "ばって", &["-て"], &["-て"]).into(),
                    suffix_inflection("もうて", "まって", &["-て"], &["-て"]).into(),
                    suffix_inflection("ろうて", "らって", &["-て"], &["-て"]).into(),
                    suffix_inflection("ようて", "やって", &["-て"], &["-て"]).into(),
                    suffix_inflection("ゆうて", "いって", &["-て"], &["-て"]).into(),
                ],
            },
        ),
        (
            "kansai-ben -た",
            Transform {
                name: "kansai-ben -た",
                description: Some(
                    "-た form of kansai-ben terms",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "関西弁",
                    description: Some("～た (関西弁)"),
                }]),
                rules: vec![
                    suffix_inflection("うた", "った", &["-た"], &["-た"]).into(),
                    suffix_inflection("おうた", "あった", &["-た"], &["-た"]).into(),
                    suffix_inflection("こうた", "かった", &["-た"], &["-た"]).into(),
                    suffix_inflection("ごうた", "がった", &["-た"], &["-た"]).into(),
                    suffix_inflection("そうた", "さった", &["-た"], &["-た"]).into(),
                    suffix_inflection("ぞうた", "ざった", &["-た"], &["-た"]).into(),
                    suffix_inflection("とうた", "たった", &["-た"], &["-た"]).into(),
                    suffix_inflection("どうた", "だった", &["-た"], &["-た"]).into(),
                    suffix_inflection("のうた", "なった", &["-た"], &["-た"]).into(),
                    suffix_inflection("ほうた", "はった", &["-た"], &["-た"]).into(),
                    suffix_inflection("ぼうた", "ばった", &["-た"], &["-た"]).into(),
                    suffix_inflection("もうた", "まった", &["-た"], &["-た"]).into(),
                    suffix_inflection("ろうた", "らった", &["-た"], &["-た"]).into(),
                    suffix_inflection("ようた", "やった", &["-た"], &["-た"]).into(),
                    suffix_inflection("ゆうた", "いった", &["-た"], &["-た"]).into(),
                ],
            },
        ),
        (
            "kansai-ben -たら",
            Transform {
                name: "kansai-ben -たら",
                description: Some(
                    "-たら form of kansai-ben terms",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "関西弁",
                    description: Some("～たら (関西弁)"),
                }]),
                rules: vec![
                    suffix_inflection("うたら", "ったら", &[], &[]).into(),
                    suffix_inflection("おうたら", "あったら", &[], &[]).into(),
                    suffix_inflection("こうたら", "かったら", &[], &[]).into(),
                    suffix_inflection("ごうたら", "がったら", &[], &[]).into(),
                    suffix_inflection("そうたら", "さったら", &[], &[]).into(),
                    suffix_inflection("ぞうたら", "ざったら", &[], &[]).into(),
                    suffix_inflection("とうたら", "たったら", &[], &[]).into(),
                    suffix_inflection("どうたら", "だったら", &[], &[]).into(),
                    suffix_inflection("のうたら", "なったら", &[], &[]).into(),
                    suffix_inflection("ほうたら", "はったら", &[], &[]).into(),
                    suffix_inflection("ぼうたら", "ばったら", &[], &[]).into(),
                    suffix_inflection("もうたら", "まったら", &[], &[]).into(),
                    suffix_inflection("ろうたら", "らったら", &[], &[]).into(),
                    suffix_inflection("ようたら", "やったら", &[], &[]).into(),
                    suffix_inflection("ゆうたら", "いったら", &[], &[]).into(),
                ],
            },
        ),
        (
            "kansai-ben -たり",
            Transform {
                name: "kansai-ben -たり",
                description: Some(
                    "-たり form of kansai-ben terms",
                ),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "関西弁",
                    description: Some("～たり (関西弁)"),
                }]),
                rules: vec![
                    suffix_inflection("うたり", "ったり", &[], &[]).into(),
                    suffix_inflection("おうたり", "あったり", &[], &[]).into(),
                    suffix_inflection("こうたり", "かったり", &[], &[]).into(),
                    suffix_inflection("ごうたり", "がったり", &[], &[]).into(),
                    suffix_inflection("そうたり", "さったり", &[], &[]).into(),
                    suffix_inflection("ぞうたり", "ざったり", &[], &[]).into(),
                    suffix_inflection("とうたり", "たったり", &[], &[]).into(),
                    suffix_inflection("どうたり", "だったり", &[], &[]).into(),
                    suffix_inflection("のうたり", "なったり", &[], &[]).into(),
                    suffix_inflection("ほうたり", "はったり", &[], &[]).into(),
                    suffix_inflection("ぼうたり", "ばったり", &[], &[]).into(),
                    suffix_inflection("もうたり", "まったり", &[], &[]).into(),
                    suffix_inflection("ろうたり", "らったり", &[], &[]).into(),
                    suffix_inflection("ようたり", "やったり", &[], &[]).into(),
                    suffix_inflection("ゆうたり", "いったり", &[], &[]).into(),
                ],
            },
        ),
        (
            "kansai-ben -く",
            Transform {
                name: "kansai-ben -く",
                description: Some("-く stem of kansai-ben adjectives"),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "関西弁",
                    description: Some("連用形 (関西弁)"),
                }]),
                rules: vec![
                    suffix_inflection("う", "く", &[], &["-く"]).into(),
                    suffix_inflection("こう", "かく", &[], &["-く"]).into(),
                    suffix_inflection("ごう", "がく", &[], &["-く"]).into(),
                    suffix_inflection("そう", "さく", &[], &["-く"]).into(),
                    suffix_inflection("とう", "たく", &[], &["-く"]).into(),
                    suffix_inflection("のう", "なく", &[], &["-く"]).into(),
                    suffix_inflection("ぼう", "ばく", &[], &["-く"]).into(),
                    suffix_inflection("もう", "まく", &[], &["-く"]).into(),
                    suffix_inflection("ろう", "らく", &[], &["-く"]).into(),
                    suffix_inflection("よう", "よく", &[], &["-く"]).into(),
                    suffix_inflection("しゅう", "しく", &[], &["-く"]).into(),
                ],
            },
        ),
        (
            "kansai-ben adjective -て",
            Transform {
                name: "kansai-ben adjective -て",
                description: Some("-て form of kansai-ben adjectives"),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "関西弁",
                    description: Some("～て (関西弁)"),
                }]),
                rules: vec![
                    suffix_inflection("うて", "くて", &["-て"], &["-て"]).into(),
                    suffix_inflection("こうて", "かくて", &["-て"], &["-て"]).into(),
                    suffix_inflection("ごうて", "がくて", &["-て"], &["-て"]).into(),
                    suffix_inflection("そうて", "さくて", &["-て"], &["-て"]).into(),
                    suffix_inflection("とうて", "たくて", &["-て"], &["-て"]).into(),
                    suffix_inflection("のうて", "なくて", &["-て"], &["-て"]).into(),
                    suffix_inflection("ぼうて", "ばくて", &["-て"], &["-て"]).into(),
                    suffix_inflection("もうて", "まくて", &["-て"], &["-て"]).into(),
                    suffix_inflection("ろうて", "らくて", &["-て"], &["-て"]).into(),
                    suffix_inflection("ようて", "よくて", &["-て"], &["-て"]).into(),
                    suffix_inflection("しゅうて", "しくて", &["-て"], &["-て"]).into(),
                ],
            },
        ),
        (
            "kansai-ben adjective negative",
            Transform {
                name: "kansai-ben adjective negative",
                description: Some("Negative form of kansai-ben adjectives"),
                i18n: Some(vec![TransformI18n {
                    language: "ja",
                    name: "関西弁",
                    description: Some("～ない (関西弁)"),
                }]),
                rules: vec![
                    suffix_inflection("うない", "くない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("こうない", "かくない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("ごうない", "がくない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("そうない", "さくない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("とうない", "たくない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("のうない", "なくない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("ぼうない", "ばくない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("もうない", "まくない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("ろうない", "らくない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("ようない", "よくない", &["adj-i"], &["adj-i"]).into(),
                    suffix_inflection("しゅうない", "しくない", &["adj-i"], &["adj-i"]).into(),
                ],
            },
        ),
    ]))
});

#[rustfmt::skip]
pub(crate) static JP_CONDITIONS: LazyLock<ConditionMap> = LazyLock::new(|| {    ConditionMap(IndexMap::from([            (                "v".to_string(),                Condition {                    name: "Verb".to_string(),                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "動詞".to_string(),                    }]),                    sub_conditions: Some(&[                        "v1",                        "v5",                        "vk",                        "vs",                        "vz",                    ]),                },            ),            (                "v1".to_string(),                Condition {                    name: "Ichidan verb".to_string(),                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "一段動詞".to_string(),                    }]),                    sub_conditions: Some(&["v1d", "v1p"]),                    },                ),            (                "v1d".to_string(),                Condition {                    name: "Ichidan verb, dictionary form".to_string(),                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "一段動詞、辞書形".to_string(),                    }]),                    sub_conditions: None,                },            ),            (                "v1p".to_string(),                Condition {                    name: "Ichidan verb, progressive or perfect form".to_string(),                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "一段動詞、～てる・でる".to_string(),                    }]),                    sub_conditions: None,                },            ),            (                "v5".to_string(),                Condition {                    name: "Godan verb".to_string(),                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "五段動詞".to_string(),                    }]),                    sub_conditions: Some(&["v5d", "v5s"]),                },            ),            (                "v5d".to_string(),                Condition {                    name: "Godan verb, dictionary form".to_string(),                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "五段動詞、終止形".to_string(),                    }]),                    sub_conditions: None,                },            ),            (                "v5s".to_string(),                Condition {                    name: "Godan verb, short causative form".to_string(),                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "五段動詞、～す・さす".to_string(),                    }]),                    sub_conditions: Some(&["v5ss", "v5sp"]),                },            ),            (                "v5ss".to_string(),                Condition {                    name: "Godan verb, short causative form having さす ending (cannot conjugate with passive form)".to_string(),                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "五段動詞、～さす".to_string(),                    }]),                    sub_conditions: None,                },            ),            (                "v5sp".to_string(),                Condition {                    name: "Godan verb, short causative form not having さす ending (can conjugate with passive form)".to_string(),                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "五段動詞、～す".to_string(),                    }]),                    sub_conditions: None,                },            ),            (                "vk".to_string(),                Condition {                    name: "Kuru verb".to_string(),                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "来る動詞".to_string(),                    }]),                    sub_conditions: None,                },            ),            (                "vs".to_string(),                Condition {                    name: "Suru verb".to_string(),                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "する動詞".to_string(),                    }]),                    sub_conditions: None,                },            ),            (                "vz".to_string(),                Condition {                    name: "Zuru verb".to_string(),                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "ずる動詞".to_string(),                    }]),                    sub_conditions: None,                },            ),            (                "adj-i".to_string(),                Condition {                    name: "Adjective with i ending".to_string(),                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja".to_string(),                        name: "形容詞".to_string(),                    }]),                    sub_conditions: None,                },            ),            (                "-ます".to_string(),                Condition {                    name: "Polite -ます ending".to_string(),                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-ません".to_string(),                Condition {                    name: "Polite negative -ません ending".to_string(),                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-て".to_string(),                Condition {                    name: "Intermediate -て endings for progressive or perfect tense".to_string(),                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-ば".to_string(),                Condition {                    name: "Intermediate -ば endings for conditional contraction".to_string(),                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-く".to_string(),                Condition {                    name: "Intermediate -く endings for adverbs".to_string(),                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-た".to_string(),                Condition {                    name: "-た form ending".to_string(),                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-ん".to_string(),                Condition {                    name: "-ん negative ending".to_string(),                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-なさい".to_string(),                Condition {                    name: "Intermediate -なさい ending (polite imperative)".to_string(),                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-ゃ".to_string(),                Condition {                    name: "Intermediate -や ending (conditional contraction)".to_string(),                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),        ]))});

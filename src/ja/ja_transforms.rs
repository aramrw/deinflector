use crate::{
    transformer::{
        Condition, ConditionMap, LanguageTransformer, Rule, RuleI18n, RuleType, SuffixRule,
    },
    transforms::inflection,
};
use fancy_regex::Regex;
use std::sync::LazyLock;

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
    pub(crate) inner: &'static str,
    pub(crate) rule: &'static str,
    pub(crate) reasons: Vec<&'static str>,
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

pub fn irregular_verb_inflections(
    suffix: IrregularVerbSuffix,
    conditions_in: &'static [&'static str],
    conditions_out: &'static [&'static str],
) -> Vec<SuffixRule> {
    let suffix_str = suffix.to_string();

    let iku_inflections = IKU_VERBS.iter().map(|verb| {
        let first_char = verb.chars().next().unwrap();
        let transformed: &'static str = format!("{first_char}っ{suffix_str}").leak();
        inflection(
            transformed,
            verb,
            conditions_in,
            conditions_out,
            RuleType::Suffix,
        )
    });

    let godan_inflections = GODAN_U_SPECIAL_VERBS.iter().map(|verb| {
        let transformed: &'static str = format!("{verb}{suffix_str}").leak();
        inflection(
            transformed,
            verb,
            conditions_in,
            conditions_out,
            RuleType::Suffix,
        )
    });

    let fu_inflections = FU_VERB_TE_CONJUGATIONS.iter().map(|[verb, te_root]| {
        let transformed: &'static str = format!("{te_root}{suffix_str}").leak();
        inflection(
            transformed,
            verb,
            conditions_in,
            conditions_out,
            RuleType::Suffix,
        )
    });

    iku_inflections
        .chain(godan_inflections)
        .chain(fu_inflections)
        .map(|v| v.into())
        .collect()
}

#[cfg(test)]
mod inflection_tests {

    use fancy_regex::Regex;
    use pretty_assertions::assert_eq;

    use crate::{
        ja::ja_transforms::irregular_verb_inflections,
        transformer::{DeinflectFnType, RuleDeinflectFnTrait, RuleType, SuffixRule},
        transforms::inflection,
    };

    // #[test]
    // pub fn irregular_verb_suffix() {
    //     #[rustfmt::skip]
    // let te_test = [
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("いって$").unwrap(), deinflected: "いく", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("行って$").unwrap(), deinflected: "行く", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("逝って$").unwrap(), deinflected: "逝く", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("往って$").unwrap(), deinflected: "往く", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("こうて$").unwrap(), deinflected: "こう", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("とうて$").unwrap(), deinflected: "とう", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("請うて$").unwrap(), deinflected: "請う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("乞うて$").unwrap(), deinflected: "乞う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("恋うて$").unwrap(), deinflected: "恋う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("問うて$").unwrap(), deinflected: "問う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("訪うて$").unwrap(), deinflected: "訪う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("宣うて$").unwrap(), deinflected: "宣う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("曰うて$").unwrap(), deinflected: "曰う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("給うて$").unwrap(), deinflected: "給う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("賜うて$").unwrap(), deinflected: "賜う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("揺蕩うて$").unwrap(), deinflected: "揺蕩う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("のたもうて$").unwrap(), deinflected: "のたまう", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("たもうて$").unwrap(), deinflected: "たまう", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("たゆとうて$").unwrap(), deinflected: "たゆたう", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-て"], conditions_out: &["v5"] }
    // ];
    //     #[rustfmt::skip]
    // let ta_test = [
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("いった$").unwrap(), deinflected: "いく", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("行った$").unwrap(), deinflected: "行く", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("逝った$").unwrap(), deinflected: "逝く", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("往った$").unwrap(), deinflected: "往く", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("こうた$").unwrap(), deinflected: "こう", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("とうた$").unwrap(), deinflected: "とう", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("請うた$").unwrap(), deinflected: "請う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("乞うた$").unwrap(), deinflected: "乞う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("恋うた$").unwrap(), deinflected: "恋う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("問うた$").unwrap(), deinflected: "問う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("訪うた$").unwrap(), deinflected: "訪う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("宣うた$").unwrap(), deinflected: "宣う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("曰うた$").unwrap(), deinflected: "曰う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("給うた$").unwrap(), deinflected: "給う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("賜うた$").unwrap(), deinflected: "賜う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("揺蕩うた$").unwrap(), deinflected: "揺蕩う", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("のたもうた$").unwrap(), deinflected: "のたまう", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("たもうた$").unwrap(), deinflected: "たまう", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] },
    //     SuffixRule { rule_type: RuleType::Suffix, is_inflected: Regex::new("たゆとうた$").unwrap(), deinflected: "たゆたう", deinflect_fn: DeinflectFnType::GenericSuffix, conditions_in: &["-た"], conditions_out: &["v5"] }
    // ];
    //     let て = irregular_verb_inflections(super::IrregularVerbSuffix::て, &["-て"], &["v5"]);
    //     assert_eq!(て, te_test);
    //     let た = irregular_verb_inflections(super::IrregularVerbSuffix::た, &["-た"], &["v5"]);
    //     assert_eq!(た, ta_test);
    // }

    #[test]
    pub fn suffix() {
        let test = SuffixRule {
            rule_type: RuleType::Suffix,
            is_inflected: Regex::new("ければ$").unwrap(),
            deinflected: "い",
            inflected_str: Some("ければ".to_string()),
            deinflect_fn: crate::transformer::DeinflectFnType::GenericSuffix,
            conditions_in: &["-ば"],
            conditions_out: &["adj-i"],
        };
        let sr = inflection("ければ", "い", &["-ば"], &["adj-i"], RuleType::Suffix);
        assert_eq!(sr, test.clone().into());
        assert_eq!(sr.deinflect("食べれば"), test.deinflect("食べれば"));
    }
}

pub(crate) static JP_TRANSFORM_TESTS: LazyLock<[&TransformTest; 14]> = LazyLock::new(|| {
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

pub static JAPANESE_TRANSFORMS_DESCRIPTOR: LazyLock<LanguageTransformDescriptor> =
    LazyLock::new(|| LanguageTransformDescriptor {
        language: "ja",
        conditions: &JP_CONDITIONS_MAP,
        transforms: &JP_TRANSFORMS_MAP,
    });

#[cfg(test)]
pub(crate) mod jp_transforms {
    use crate::transformer::LanguageTransformer;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn len() {
        assert_eq!(JAPANESE_TRANSFORMS_DESCRIPTOR.transforms.len(), 53);
        assert_eq!(JAPANESE_TRANSFORMS_DESCRIPTOR.conditions.len(), 22);
    }

    #[test]
    fn transforms() {
        let mut lt = LanguageTransformer::new();
        lt.add_descriptor(&JAPANESE_TRANSFORMS_DESCRIPTOR).unwrap();

        for test in JP_TRANSFORM_TESTS.iter() {
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

pub(crate) fn has_term_reasons(
    lt: &LanguageTransformer,
    source: &str,
    expected_term: &str,
    expected_condition_name: Option<&str>,
    expected_reasons: Option<&[&str]>,
) -> Result<HasTermReasons, HasTermReasonsError> {
    let results = lt.transform(source);
    //dbg!(&results);
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
            rules: result.conditions,
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

pub static JP_TRANSFORMS_MAP: LazyLock<TransformMap> = LazyLock::new(|| {
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
                    inflection("ければ", "い", &["-ば"], &["adj-i"], RuleType::Suffix),
                    inflection("えば", "う", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("けば", "く", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("げば", "ぐ", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("せば", "す", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("てば", "つ", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("ねば", "ぬ", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("べば", "ぶ", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("めば", "む", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("れば", "る", &["-ば"], &["v1", "v5", "vk", "vs", "vz"], RuleType::Suffix),
                    inflection("れば", "",   &["-ば"], &["-ます"], RuleType::Suffix),
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
                    inflection("けりゃ", "ければ", &["-ゃ"], &["-ば"], RuleType::Suffix),
                    inflection("きゃ", "ければ", &["-ゃ"], &["-ば"], RuleType::Suffix),
                    inflection("や", "えば", &["-ゃ"], &["-ば"], RuleType::Suffix),
                    inflection("きゃ", "けば", &["-ゃ"], &["-ば"], RuleType::Suffix),
                    inflection("ぎゃ", "げば", &["-ゃ"], &["-ば"], RuleType::Suffix),
                    inflection("しゃ", "せば", &["-ゃ"], &["-ば"], RuleType::Suffix),
                    inflection("ちゃ", "てば", &["-ゃ"], &["-ば"], RuleType::Suffix),
                    inflection("にゃ", "ねば", &["-ゃ"], &["-ば"], RuleType::Suffix),
                    inflection("びゃ", "べば", &["-ゃ"], &["-ば"], RuleType::Suffix),
                    inflection("みゃ", "めば", &["-ゃ"], &["-ば"], RuleType::Suffix),
                    inflection("りゃ", "れば", &["-ゃ"], &["-ば"], RuleType::Suffix),
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
                    inflection("ちゃ", "る", &["v5"], &["v1"], RuleType::Suffix),
                    inflection("いじゃ", "ぐ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("いちゃ", "く", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("しちゃ", "す", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちゃ", "う", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちゃ", "く", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちゃ", "つ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちゃ", "る", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("んじゃ", "ぬ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("んじゃ", "ぶ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("んじゃ", "む", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("じちゃ", "ずる", &["v5"], &["vz"], RuleType::Suffix),
                    inflection("しちゃ", "する", &["v5"], &["vs"], RuleType::Suffix),
                    inflection("為ちゃ", "為る", &["v5"], &["vs"], RuleType::Suffix),
                    inflection("きちゃ", "くる", &["v5"], &["vk"], RuleType::Suffix),
                    inflection("来ちゃ", "来る", &["v5"], &["vk"], RuleType::Suffix),
                    inflection("來ちゃ", "來る", &["v5"], &["vk"], RuleType::Suffix),
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
                    inflection("ちゃう", "る", &["v5"], &["v1"], RuleType::Suffix),
                    inflection("いじゃう", "ぐ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("いちゃう", "く", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("しちゃう", "す", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちゃう", "う", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちゃう", "く", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちゃう", "つ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちゃう", "る", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("んじゃう", "ぬ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("んじゃう", "ぶ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("んじゃう", "む", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("じちゃう", "ずる", &["v5"], &["vz"], RuleType::Suffix),
                    inflection("しちゃう", "する", &["v5"], &["vs"], RuleType::Suffix),
                    inflection("為ちゃう", "為る", &["v5"], &["vs"], RuleType::Suffix),
                    inflection("きちゃう", "くる", &["v5"], &["vk"], RuleType::Suffix),
                    inflection("来ちゃう", "来る", &["v5"], &["vk"], RuleType::Suffix),
                    inflection("來ちゃう", "來る", &["v5"], &["vk"], RuleType::Suffix),
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
                    inflection("ちまう", "る", &["v5"], &["v1"], RuleType::Suffix),
                    inflection("いじまう", "ぐ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("いちまう", "く", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("しちまう", "す", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちまう", "う", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちまう", "く", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちまう", "つ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("っちまう", "る", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("んじまう", "ぬ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("んじまう", "ぶ", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("んじまう", "む", &["v5"], &["v5"], RuleType::Suffix),
                    inflection("じちまう", "ずる", &["v5"], &["vz"], RuleType::Suffix),
                    inflection("しちまう", "する", &["v5"], &["vs"], RuleType::Suffix),
                    inflection("為ちまう", "為る", &["v5"], &["vs"], RuleType::Suffix),
                    inflection("きちまう", "くる", &["v5"], &["vk"], RuleType::Suffix),
                    inflection("来ちまう", "来る", &["v5"], &["vk"], RuleType::Suffix),
                    inflection("來ちまう", "來る", &["v5"], &["vk"], RuleType::Suffix),
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
                    inflection("てしまう", "て", &["v5"], &["-て"], RuleType::Suffix),
                    inflection("でしまう", "で", &["v5"], &["-て"], RuleType::Suffix),
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
                    inflection("なさい", "る", &["-なさい"], &["v1"], RuleType::Suffix),
                    inflection("いなさい", "う", &["-なさい"], &["v5"], RuleType::Suffix),
                    inflection("きなさい", "く", &["-なさい"], &["v5"], RuleType::Suffix),
                    inflection("ぎなさい", "ぐ", &["-なさい"], &["v5"], RuleType::Suffix),
                    inflection("しなさい", "す", &["-なさい"], &["v5"], RuleType::Suffix),
                    inflection("ちなさい", "つ", &["-なさい"], &["v5"], RuleType::Suffix),
                    inflection("になさい", "ぬ", &["-なさい"], &["v5"], RuleType::Suffix),
                    inflection("びなさい", "ぶ", &["-なさい"], &["v5"], RuleType::Suffix),
                    inflection("みなさい", "む", &["-なさい"], &["v5"], RuleType::Suffix),
                    inflection("りなさい", "る", &["-なさい"], &["v5"], RuleType::Suffix),
                    inflection("じなさい", "ずる", &["-なさい"], &["vz"], RuleType::Suffix),
                    inflection("しなさい", "する", &["-なさい"], &["vs"], RuleType::Suffix),
                    inflection("為なさい", "為る", &["-なさい"], &["vs"], RuleType::Suffix),
                    inflection("きなさい", "くる", &["-なさい"], &["vk"], RuleType::Suffix),
                    inflection("来なさい", "来る", &["-なさい"], &["vk"], RuleType::Suffix),
                    inflection("來なさい", "來る", &["-なさい"], &["vk"], RuleType::Suffix),
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
                    inflection("そう", "い", &[], &["adj-i"], RuleType::Suffix),
                    inflection("そう", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("いそう", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("きそう", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("ぎそう", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("しそう", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("ちそう", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("にそう", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("びそう", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("みそう", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("りそう", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("じそう", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("しそう", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為そう", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("きそう", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来そう", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來そう", "來る", &[], &["vk"], RuleType::Suffix),
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
                    inflection("すぎる", "い", &["v1"], &["adj-i"], RuleType::Suffix),
                    inflection("すぎる", "る", &["v1"], &["v1"], RuleType::Suffix),
                    inflection("いすぎる", "う", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("きすぎる", "く", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("ぎすぎる", "ぐ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("しすぎる", "す", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("ちすぎる", "つ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("にすぎる", "ぬ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("びすぎる", "ぶ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("みすぎる", "む", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("りすぎる", "る", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("じすぎる", "ずる", &["v1"], &["vz"], RuleType::Suffix),
                    inflection("しすぎる", "する", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("為すぎる", "為る", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("きすぎる", "くる", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("来すぎる", "来る", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("來すぎる", "來る", &["v1"], &["vk"], RuleType::Suffix),
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
                    inflection("過ぎる", "い", &["v1"], &["adj-i"], RuleType::Suffix),
                    inflection("過ぎる", "る", &["v1"], &["v1"], RuleType::Suffix),
                    inflection("い過ぎる", "う", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("き過ぎる", "く", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("ぎ過ぎる", "ぐ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("し過ぎる", "す", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("ち過ぎる", "つ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("に過ぎる", "ぬ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("び過ぎる", "ぶ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("み過ぎる", "む", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("り過ぎる", "る", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("じ過ぎる", "ずる", &["v1"], &["vz"], RuleType::Suffix),
                    inflection("し過ぎる", "する", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("為過ぎる", "為る", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("き過ぎる", "くる", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("来過ぎる", "来る", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("來過ぎる", "來る", &["v1"], &["vk"], RuleType::Suffix),
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
                    inflection("たい", "る", &["adj-i"], &["v1"], RuleType::Suffix),
                    inflection("いたい", "う", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("きたい", "く", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("ぎたい", "ぐ", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("したい", "す", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("ちたい", "つ", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("にたい", "ぬ", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("びたい", "ぶ", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("みたい", "む", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("りたい", "る", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("じたい", "ずる", &["adj-i"], &["vz"], RuleType::Suffix),
                    inflection("したい", "する", &["adj-i"], &["vs"], RuleType::Suffix),
                    inflection("為たい", "為る", &["adj-i"], &["vs"], RuleType::Suffix),
                    inflection("きたい", "くる", &["adj-i"], &["vk"], RuleType::Suffix),
                    inflection("来たい", "来る", &["adj-i"], &["vk"], RuleType::Suffix),
                    inflection("來たい", "來る", &["adj-i"], &["vk"], RuleType::Suffix),
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
                    inflection("かったら", "い", &[], &["adj-i"], RuleType::Suffix),
                    inflection("たら",  "る", &[], &["v1"], RuleType::Suffix),
                    inflection("いたら", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("いだら", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("したら", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("ったら", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("ったら", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("ったら", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("んだら", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("んだら", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("んだら", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("じたら", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("したら", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為たら", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("きたら", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来たら", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來たら", "來る", &[], &["vk"], RuleType::Suffix),
                ].into_iter().chain(irregular_verb_inflections(
                    IrregularVerbSuffix::たら,
                    &[],
                    &["v5"]
                ).into_iter().map(Into::into)).chain(std::iter::once(inflection("ましたら", "ます", &[], &["-ます"], RuleType::Suffix))).collect(),
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
                    inflection("かったり", "い", &[], &["adj-i"], RuleType::Suffix),
                    inflection("たり", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("いたり", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("いだり", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("したり", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("ったり", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("ったり", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("ったり", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("んだり", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("んだり", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("んだり", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("じたり", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("したり", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為たり", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("きたり", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来たり", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來たり", "來る", &[], &["vk"], RuleType::Suffix),
                ].into_iter().chain(irregular_verb_inflections(IrregularVerbSuffix::たり, &[], &["v5"]).into_iter().map(Into::into)).collect(),
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
                    inflection("くて", "い", &["-て"], &["adj-i"], RuleType::Suffix),
                    inflection("て", "る", &["-て"], &["v1"], RuleType::Suffix),
                    inflection("いて", "く", &["-て"], &["v5"], RuleType::Suffix),
                    inflection("いで", "ぐ", &["-て"], &["v5"], RuleType::Suffix),
                    inflection("して", "す", &["-て"], &["v5"], RuleType::Suffix),
                    inflection("って", "う", &["-て"], &["v5"], RuleType::Suffix),
                    inflection("って", "つ", &["-て"], &["v5"], RuleType::Suffix),
                    inflection("って", "る", &["-て"], &["v5"], RuleType::Suffix),
                    inflection("んで", "ぬ", &["-て"], &["v5"], RuleType::Suffix),
                    inflection("んで", "ぶ", &["-て"], &["v5"], RuleType::Suffix),
                    inflection("んで", "む", &["-て"], &["v5"], RuleType::Suffix),
                    inflection("じて", "ずる", &["-て"], &["vz"], RuleType::Suffix),
                    inflection("して", "する", &["-て"], &["vs"], RuleType::Suffix),
                    inflection("為て", "為る", &["-て"], &["vs"], RuleType::Suffix),
                    inflection("きて", "くる", &["-て"], &["vk"], RuleType::Suffix),
                    inflection("来て", "来る", &["-て"], &["vk"], RuleType::Suffix),
                    inflection("來て", "來る", &["-て"], &["vk"], RuleType::Suffix),
                ].into_iter()
                    .chain(irregular_verb_inflections(
                        IrregularVerbSuffix::て,
                        &["-て"],
                        &["v5"]
                    ).into_iter().map(Into::into))
                    .chain(Vec::from_iter([inflection("まして", "ます", &[], &["-ます"], RuleType::Suffix)]))
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
                    inflection("ず", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("かず", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("がず", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("さず", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("たず", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("なず", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("ばず", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("まず", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("らず", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("わず", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("ぜず", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("せず", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為ず", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("こず", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来ず", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來ず", "來る", &[], &["vk"], RuleType::Suffix),
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
                    inflection("ぬ", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("かぬ", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("がぬ", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("さぬ", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("たぬ", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("なぬ", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("ばぬ", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("まぬ", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("らぬ", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("わぬ", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("ぜぬ", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("せぬ", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為ぬ", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("こぬ", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来ぬ", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來ぬ", "來る", &[], &["vk"], RuleType::Suffix),
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
                    inflection("ん", "る", &["-ん"], &["v1"], RuleType::Suffix),
                    inflection("かん", "く", &["-ん"], &["v5"], RuleType::Suffix),
                    inflection("がん", "ぐ", &["-ん"], &["v5"], RuleType::Suffix),
                    inflection("さん", "す", &["-ん"], &["v5"], RuleType::Suffix),
                    inflection("たん", "つ", &["-ん"], &["v5"], RuleType::Suffix),
                    inflection("なん", "ぬ", &["-ん"], &["v5"], RuleType::Suffix),
                    inflection("ばん", "ぶ", &["-ん"], &["v5"], RuleType::Suffix),
                    inflection("まん", "む", &["-ん"], &["v5"], RuleType::Suffix),
                    inflection("らん", "る", &["-ん"], &["v5"], RuleType::Suffix),
                    inflection("わん", "う", &["-ん"], &["v5"], RuleType::Suffix),
                    inflection("ぜん", "ずる", &["-ん"], &["vz"], RuleType::Suffix),
                    inflection("せん", "する", &["-ん"], &["vs"], RuleType::Suffix),
                    inflection("為ん", "為る", &["-ん"], &["vs"], RuleType::Suffix),
                    inflection("こん", "くる", &["-ん"], &["vk"], RuleType::Suffix),
                    inflection("来ん", "来る", &["-ん"], &["vk"], RuleType::Suffix),
                    inflection("來ん", "來る", &["-ん"], &["vk"], RuleType::Suffix),
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
                    inflection("んばかり", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("かんばかり", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("がんばかり", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("さんばかり", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("たんばかり", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("なんばかり", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("ばんばかり", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("まんばかり", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("らんばかり", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("わんばかり", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("ぜんばかり", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("せんばかり", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為んばかり", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("こんばかり", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来んばかり", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來んばかり", "來る", &[], &["vk"], RuleType::Suffix),
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
                    inflection("んとする", "る", &["vs"], &["v1"], RuleType::Suffix),
                    inflection("かんとする", "く", &["vs"], &["v5"], RuleType::Suffix),
                    inflection("がんとする", "ぐ", &["vs"], &["v5"], RuleType::Suffix),
                    inflection("さんとする", "す", &["vs"], &["v5"], RuleType::Suffix),
                    inflection("たんとする", "つ", &["vs"], &["v5"], RuleType::Suffix),
                    inflection("なんとする", "ぬ", &["vs"], &["v5"], RuleType::Suffix),
                    inflection("ばんとする", "ぶ", &["vs"], &["v5"], RuleType::Suffix),
                    inflection("まんとする", "む", &["vs"], &["v5"], RuleType::Suffix),
                    inflection("らんとする", "る", &["vs"], &["v5"], RuleType::Suffix),
                    inflection("わんとする", "う", &["vs"], &["v5"], RuleType::Suffix),
                    inflection("ぜんとする", "ずる", &["vs"], &["vz"], RuleType::Suffix),
                    inflection("せんとする", "する", &["vs"], &["vs"], RuleType::Suffix),
                    inflection("為んとする", "為る", &["vs"], &["vs"], RuleType::Suffix),
                    inflection("こんとする", "くる", &["vs"], &["vk"], RuleType::Suffix),
                    inflection("来んとする", "来る", &["vs"], &["vk"], RuleType::Suffix),
                    inflection("來んとする", "來る", &["vs"], &["vk"], RuleType::Suffix),
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
                    inflection("む", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("かむ", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("がむ", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("さむ", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("たむ", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("なむ", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("ばむ", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("まむ", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("らむ", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("わむ", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("ぜむ", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("せむ", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為む", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("こむ", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来む", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來む", "來る", &[], &["vk"], RuleType::Suffix),
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
                    inflection("ざる", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("かざる", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("がざる", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("さざる", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("たざる", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("なざる", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("ばざる", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("まざる", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("らざる", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("わざる", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("ぜざる", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("せざる", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為ざる", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("こざる", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来ざる", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來ざる", "來る", &[], &["vk"], RuleType::Suffix),
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
                    inflection("ねば", "る", &["-ば"], &["v1"], RuleType::Suffix),
                    inflection("かねば", "く", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("がねば", "ぐ", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("さねば", "す", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("たねば", "つ", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("なねば", "ぬ", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("ばねば", "ぶ", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("まねば", "む", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("らねば", "る", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("わねば", "う", &["-ば"], &["v5"], RuleType::Suffix),
                    inflection("ぜねば", "ずる", &["-ば"], &["vz"], RuleType::Suffix),
                    inflection("せねば", "する", &["-ば"], &["vs"], RuleType::Suffix),
                    inflection("為ねば", "為る", &["-ば"], &["vs"], RuleType::Suffix),
                    inflection("こねば", "くる", &["-ば"], &["vk"], RuleType::Suffix),
                    inflection("来ねば", "来る", &["-ば"], &["vk"], RuleType::Suffix),
                    inflection("來ねば", "來る", &["-ば"], &["vk"], RuleType::Suffix),
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
                    inflection("く", "い", &["-く"], &["adj-i"], RuleType::Suffix),
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
                    inflection("させる", "る", &["v1"], &["v1"], RuleType::Suffix),
                    inflection("かせる", "く", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("がせる", "ぐ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("させる", "す", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("たせる", "つ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("なせる", "ぬ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("ばせる", "ぶ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("ませる", "む", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("らせる", "る", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("わせる", "う", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("じさせる", "ずる", &["v1"], &["vz"], RuleType::Suffix),
                    inflection("ぜさせる", "ずる", &["v1"], &["vz"], RuleType::Suffix),
                    inflection("させる", "する", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("為せる", "為る", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("せさせる", "する", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("為させる", "為る", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("こさせる", "くる", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("来させる", "来る", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("來させる", "來る", &["v1"], &["vk"], RuleType::Suffix),
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
                    inflection("さす", "る", &["v5ss"], &["v1"], RuleType::Suffix),
                    inflection("かす", "く", &["v5sp"], &["v5"], RuleType::Suffix),
                    inflection("がす", "ぐ", &["v5sp"], &["v5"], RuleType::Suffix),
                    inflection("さす", "す", &["v5ss"], &["v5"], RuleType::Suffix),
                    inflection("たす", "つ", &["v5sp"], &["v5"], RuleType::Suffix),
                    inflection("なす", "ぬ", &["v5sp"], &["v5"], RuleType::Suffix),
                    inflection("ばす", "ぶ", &["v5sp"], &["v5"], RuleType::Suffix),
                    inflection("ます", "む", &["v5sp"], &["v5"], RuleType::Suffix),
                    inflection("らす", "る", &["v5sp"], &["v5"], RuleType::Suffix),
                    inflection("わす", "う", &["v5sp"], &["v5"], RuleType::Suffix),
                    inflection("じさす", "ずる", &["v5ss"], &["vz"], RuleType::Suffix),
                    inflection("ぜさす", "ずる", &["v5ss"], &["vz"], RuleType::Suffix),
                    inflection("さす", "する", &["v5ss"], &["vs"], RuleType::Suffix),
                    inflection("為す", "為る", &["v5ss"], &["vs"], RuleType::Suffix),
                    inflection("こさす", "くる", &["v5ss"], &["vk"], RuleType::Suffix),
                    inflection("来さす", "来る", &["v5ss"], &["vk"], RuleType::Suffix),
                    inflection("來さす", "來る", &["v5ss"], &["vk"], RuleType::Suffix),
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
                    inflection("ろ", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("よ", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("え", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("け", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("げ", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("せ", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("て", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("ね", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("べ", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("め", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("れ", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("じろ", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("ぜよ", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("しろ", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("せよ", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為ろ", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("為よ", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("こい", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来い", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來い", "來る", &[], &["vk"], RuleType::Suffix),
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
                    inflection("い", "いる", &[], &["v1d"], RuleType::Suffix),
                    inflection("え", "える", &[], &["v1d"], RuleType::Suffix),
                    inflection("き", "きる", &[], &["v1d"], RuleType::Suffix),
                    inflection("ぎ", "ぎる", &[], &["v1d"], RuleType::Suffix),
                    inflection("け", "ける", &[], &["v1d"], RuleType::Suffix),
                    inflection("げ", "げる", &[], &["v1d"], RuleType::Suffix),
                    inflection("じ", "じる", &[], &["v1d"], RuleType::Suffix),
                    inflection("せ", "せる", &[], &["v1d"], RuleType::Suffix),
                    inflection("ぜ", "ぜる", &[], &["v1d"], RuleType::Suffix),
                    inflection("ち", "ちる", &[], &["v1d"], RuleType::Suffix),
                    inflection("て", "てる", &[], &["v1d"], RuleType::Suffix),
                    inflection("で", "でる", &[], &["v1d"], RuleType::Suffix),
                    inflection("に", "にる", &[], &["v1d"], RuleType::Suffix),
                    inflection("ね", "ねる", &[], &["v1d"], RuleType::Suffix),
                    inflection("ひ", "ひる", &[], &["v1d"], RuleType::Suffix),
                    inflection("び", "びる", &[], &["v1d"], RuleType::Suffix),
                    inflection("へ", "へる", &[], &["v1d"], RuleType::Suffix),
                    inflection("べ", "べる", &[], &["v1d"], RuleType::Suffix),
                    inflection("み", "みる", &[], &["v1d"], RuleType::Suffix),
                    inflection("め", "める", &[], &["v1d"], RuleType::Suffix),
                    inflection("り", "りる", &[], &["v1d"], RuleType::Suffix),
                    inflection("れ", "れる", &[], &["v1d"], RuleType::Suffix),
                    inflection("い", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("き", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("ぎ", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("し", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("ち", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("に", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("び", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("み", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("り", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("き", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("し", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("来", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來", "來る", &[], &["vk"], RuleType::Suffix),
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
                    inflection("くない", "い", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("ない", "る", &["adj-i"], &["v1"], RuleType::Suffix),
                    inflection("かない", "く", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("がない", "ぐ", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("さない", "す", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("たない", "つ", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("なない", "ぬ", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("ばない", "ぶ", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("まない", "む", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("らない", "る", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("わない", "う", &["adj-i"], &["v5"], RuleType::Suffix),
                    inflection("じない", "ずる", &["adj-i"], &["vz"], RuleType::Suffix),
                    inflection("しない", "する", &["adj-i"], &["vs"], RuleType::Suffix),
                    inflection("為ない", "為る", &["adj-i"], &["vs"], RuleType::Suffix),
                    inflection("こない", "くる", &["adj-i"], &["vk"], RuleType::Suffix),
                    inflection("来ない", "来る", &["adj-i"], &["vk"], RuleType::Suffix),
                    inflection("來ない", "來る", &["adj-i"], &["vk"], RuleType::Suffix),
                    inflection("ません", "ます", &["-ません"], &["-ます"], RuleType::Suffix),
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
                    inflection("さ", "い", &[], &["adj-i"], RuleType::Suffix),
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
                    inflection("かれる", "く", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("がれる", "ぐ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("される", "す", &["v1"], &["v5d", "v5sp"], RuleType::Suffix),
                    inflection("たれる", "つ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("なれる", "ぬ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("ばれる", "ぶ", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("まれる", "む", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("われる", "う", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("られる", "る", &["v1"], &["v5"], RuleType::Suffix),
                    inflection("じされる", "ずる", &["v1"], &["vz"], RuleType::Suffix),
                    inflection("ぜされる", "ずる", &["v1"], &["vz"], RuleType::Suffix),
                    inflection("される", "する", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("為れる", "為る", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("こられる", "くる", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("来られる", "来る", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("來られる", "來る", &["v1"], &["vk"], RuleType::Suffix),
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
                    inflection("かった", "い", &["-た"], &["adj-i"], RuleType::Suffix),
                    inflection("た", "る", &["-た"], &["v1"], RuleType::Suffix),
                    inflection("いた", "く", &["-た"], &["v5"], RuleType::Suffix),
                    inflection("いだ", "ぐ", &["-た"], &["v5"], RuleType::Suffix),
                    inflection("した", "す", &["-た"], &["v5"], RuleType::Suffix),
                    inflection("った", "う", &["-た"], &["v5"], RuleType::Suffix),
                    inflection("った", "つ", &["-た"], &["v5"], RuleType::Suffix),
                    inflection("った", "る", &["-た"], &["v5"], RuleType::Suffix),
                    inflection("んだ", "ぬ", &["-た"], &["v5"], RuleType::Suffix),
                    inflection("んだ", "ぶ", &["-た"], &["v5"], RuleType::Suffix),
                    inflection("んだ", "む", &["-た"], &["v5"], RuleType::Suffix),
                    inflection("じた", "ずる", &["-た"], &["vz"], RuleType::Suffix),
                    inflection("した", "する", &["-た"], &["vs"], RuleType::Suffix),
                    inflection("為た", "為る", &["-た"], &["vs"], RuleType::Suffix),
                    inflection("きた", "くる", &["-た"], &["vk"], RuleType::Suffix),
                    inflection("来た", "来る", &["-た"], &["vk"], RuleType::Suffix),
                    inflection("來た", "來る", &["-た"], &["vk"], RuleType::Suffix),
                ]
                .into_iter()
                .chain(irregular_verb_inflections(IrregularVerbSuffix::た, &["-た"], &["v5"]).into_iter().map(Into::into))
                .chain([
                    inflection("ました", "ます", &["-た"], &["-ます"], RuleType::Suffix),
                    inflection("でした", "", &["-た"], &["-ません"], RuleType::Suffix),
                    inflection("かった", "", &["-た"], &["-ません", "-ん"], RuleType::Suffix)
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
                    inflection("ます", "る", &["-ます"], &["v1"], RuleType::Suffix),
                    inflection("います", "う", &["-ます"], &["v5d"], RuleType::Suffix),
                    inflection("きます", "く", &["-ます"], &["v5d"], RuleType::Suffix),
                    inflection("ぎます", "ぐ", &["-ます"], &["v5d"], RuleType::Suffix),
                    inflection("します", "す", &["-ます"], &["v5d", "v5s"], RuleType::Suffix),
                    inflection("ちます", "つ", &["-ます"], &["v5d"], RuleType::Suffix),
                    inflection("にます", "ぬ", &["-ます"], &["v5d"], RuleType::Suffix),
                    inflection("びます", "ぶ", &["-ます"], &["v5d"], RuleType::Suffix),
                    inflection("みます", "む", &["-ます"], &["v5d"], RuleType::Suffix),
                    inflection("ります", "る", &["-ます"], &["v5d"], RuleType::Suffix),
                    inflection("じます", "ずる", &["-ます"], &["vz"], RuleType::Suffix),
                    inflection("します", "する", &["-ます"], &["vs"], RuleType::Suffix),
                    inflection("為ます", "為る", &["-ます"], &["vs"], RuleType::Suffix),
                    inflection("きます", "くる", &["-ます"], &["vk"], RuleType::Suffix),
                    inflection("来ます", "来る", &["-ます"], &["vk"], RuleType::Suffix),
                    inflection("來ます", "來る", &["-ます"], &["vk"], RuleType::Suffix),
                    inflection("くあります", "い", &["-ます"], &["adj-i"], RuleType::Suffix),
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
                    inflection("れる", "る", &["v1"], &["v1", "v5d"], RuleType::Suffix),
                    inflection("える", "う", &["v1"], &["v5d"], RuleType::Suffix),
                    inflection("ける", "く", &["v1"], &["v5d"], RuleType::Suffix),
                    inflection("げる", "ぐ", &["v1"], &["v5d"], RuleType::Suffix),
                    inflection("せる", "す", &["v1"], &["v5d"], RuleType::Suffix),
                    inflection("てる", "つ", &["v1"], &["v5d"], RuleType::Suffix),
                    inflection("ねる", "ぬ", &["v1"], &["v5d"], RuleType::Suffix),
                    inflection("べる", "ぶ", &["v1"], &["v5d"], RuleType::Suffix),
                    inflection("める", "む", &["v1"], &["v5d"], RuleType::Suffix),
                    inflection("できる", "する", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("出来る", "する", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("これる", "くる", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("来れる", "来る", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("來れる", "來る", &["v1"], &["vk"], RuleType::Suffix),
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
                    inflection("られる", "る", &["v1"], &["v1"], RuleType::Suffix),
                    inflection("ざれる", "ずる", &["v1"], &["vz"], RuleType::Suffix),
                    inflection("ぜられる", "ずる", &["v1"], &["vz"], RuleType::Suffix),
                    inflection("せられる", "する", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("為られる", "為る", &["v1"], &["vs"], RuleType::Suffix),
                    inflection("こられる", "くる", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("来られる", "来る", &["v1"], &["vk"], RuleType::Suffix),
                    inflection("來られる", "來る", &["v1"], &["vk"], RuleType::Suffix),
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
                    inflection("よう", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("おう", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("こう", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("ごう", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("そう", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("とう", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("のう", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("ぼう", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("もう", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("ろう", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("じよう", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("しよう", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為よう", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("こよう", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来よう", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來よう", "來る", &[], &["vk"], RuleType::Suffix),
                    inflection("ましょう", "ます", &[], &["-ます"], RuleType::Suffix),
                    inflection("かろう", "い", &[], &["adj-i"], RuleType::Suffix),
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
                    inflection("よっか", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("おっか", "う", &[], &["v5"], RuleType::Suffix),
                    inflection("こっか", "く", &[], &["v5"], RuleType::Suffix),
                    inflection("ごっか", "ぐ", &[], &["v5"], RuleType::Suffix),
                    inflection("そっか", "す", &[], &["v5"], RuleType::Suffix),
                    inflection("とっか", "つ", &[], &["v5"], RuleType::Suffix),
                    inflection("のっか", "ぬ", &[], &["v5"], RuleType::Suffix),
                    inflection("ぼっか", "ぶ", &[], &["v5"], RuleType::Suffix),
                    inflection("もっか", "む", &[], &["v5"], RuleType::Suffix),
                    inflection("ろっか", "る", &[], &["v5"], RuleType::Suffix),
                    inflection("じよっか", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("しよっか", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為よっか", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("こよっか", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来よっか", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來よっか", "來る", &[], &["vk"], RuleType::Suffix),
                    inflection("ましょっか", "ます", &[], &["-ます"], RuleType::Suffix),
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
                    inflection("まい", "", &[], &["v"], RuleType::Suffix),
                    inflection("まい", "る", &[], &["v1"], RuleType::Suffix),
                    inflection("じまい", "ずる", &[], &["vz"], RuleType::Suffix),
                    inflection("しまい", "する", &[], &["vs"], RuleType::Suffix),
                    inflection("為まい", "為る", &[], &["vs"], RuleType::Suffix),
                    inflection("こまい", "くる", &[], &["vk"], RuleType::Suffix),
                    inflection("来まい", "来る", &[], &["vk"], RuleType::Suffix),
                    inflection("來まい", "來る", &[], &["vk"], RuleType::Suffix),
                    inflection("まい", "", &[], &["-ます"], RuleType::Suffix),
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
                    inflection("ておく", "て", &["v5"], &["-て"], RuleType::Suffix),
                    inflection("でおく", "で", &["v5"], &["-て"], RuleType::Suffix),
                    inflection("とく", "て", &["v5"], &["-て"], RuleType::Suffix),
                    inflection("どく", "で", &["v5"], &["-て"], RuleType::Suffix),
                    inflection("ないでおく", "ない", &["v5"], &["adj-i"], RuleType::Suffix),
                    inflection("ないどく", "ない", &["v5"], &["adj-i"], RuleType::Suffix),
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
                    inflection("ている", "て", &["v1"], &["-て"], RuleType::Suffix),
                    inflection("ておる", "て", &["v5"], &["-て"], RuleType::Suffix),
                    inflection("てる", "て", &["v1p"], &["-て"], RuleType::Suffix),
                    inflection("でいる", "で", &["v1"], &["-て"], RuleType::Suffix),
                    inflection("でおる", "で", &["v5"], &["-て"], RuleType::Suffix),
                    inflection("でる", "で", &["v1p"], &["-て"], RuleType::Suffix),
                    inflection("とる", "て", &["v5"], &["-て"], RuleType::Suffix),
                    inflection("ないでいる", "ない", &["v1"], &["adj-i"], RuleType::Suffix),
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
                    inflection("き", "い", &[], &["adj-i"], RuleType::Suffix),
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
                    inflection("げ", "い", &[], &["adj-i"], RuleType::Suffix),
                    inflection("気", "い", &[], &["adj-i"], RuleType::Suffix),
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
                    inflection("がる", "い", &["v5"], &["adj-i"], RuleType::Suffix),
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
                    inflection("ねえ", "ない", &[], &["adj-i"], RuleType::Suffix),
                    inflection("めえ", "むい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("みい", "むい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ちぇえ", "つい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ちい", "つい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("せえ", "すい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ええ", "いい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ええ", "わい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ええ", "よい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("いぇえ", "よい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("うぇえ", "わい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("けえ", "かい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("げえ", "がい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("げえ", "ごい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("せえ", "さい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("めえ", "まい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ぜえ", "ずい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("っぜえ", "ずい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("れえ", "らい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("れえ", "らい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ちぇえ", "ちゃい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("でえ", "どい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("れえ", "れい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("べえ", "ばい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("てえ", "たい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ねぇ", "ない", &[], &["adj-i"], RuleType::Suffix),
                    inflection("めぇ", "むい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("みぃ", "むい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ちぃ", "つい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("せぇ", "すい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("けぇ", "かい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("げぇ", "がい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("げぇ", "ごい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("せぇ", "さい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("めぇ", "まい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ぜぇ", "ずい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("っぜぇ", "ずい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("れぇ", "らい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("でぇ", "どい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("れぇ", "れい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("べぇ", "ばい", &[], &["adj-i"], RuleType::Suffix),
                    inflection("てぇ", "たい", &[], &["adj-i"], RuleType::Suffix),
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
                    inflection("んなさい", "りなさい", &[], &["-なさい"], RuleType::Suffix),
                    inflection("らんない", "られない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("んない", "らない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("んなきゃ", "らなきゃ", &[], &["-ゃ"], RuleType::Suffix),
                    inflection("んなきゃ", "れなきゃ", &[], &["-ゃ"], RuleType::Suffix),
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
                    inflection("んな", "る", &[], &["v"], RuleType::Suffix),
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
                    inflection("へん", "ない", &[], &["adj-i"], RuleType::Suffix),
                    inflection("ひん", "ない", &[], &["adj-i"], RuleType::Suffix),
                    inflection("せえへん", "しない", &[], &["adj-i"], RuleType::Suffix),
                    inflection("へんかった", "なかった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("ひんかった", "なかった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("うてへん", "ってない", &[], &["adj-i"], RuleType::Suffix),
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
                    inflection("うて", "って", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("おうて", "あって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("こうて", "かって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ごうて", "がって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("そうて", "さって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ぞうて", "ざって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("とうて", "たって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("どうて", "だって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("のうて", "なって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ほうて", "はって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ぼうて", "ばって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("もうて", "まって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ろうて", "らって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ようて", "やって", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ゆうて", "いって", &["-て"], &["-て"], RuleType::Suffix),
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
                    inflection("うた", "った", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("おうた", "あった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("こうた", "かった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("ごうた", "がった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("そうた", "さった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("ぞうた", "ざった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("とうた", "たった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("どうた", "だった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("のうた", "なった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("ほうた", "はった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("ぼうた", "ばった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("もうた", "まった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("ろうた", "らった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("ようた", "やった", &["-た"], &["-た"], RuleType::Suffix),
                    inflection("ゆうた", "いった", &["-た"], &["-た"], RuleType::Suffix),
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
                    inflection("うたら", "ったら", &[], &[], RuleType::Suffix),
                    inflection("おうたら", "あったら", &[], &[], RuleType::Suffix),
                    inflection("こうたら", "かったら", &[], &[], RuleType::Suffix),
                    inflection("ごうたら", "がったら", &[], &[], RuleType::Suffix),
                    inflection("そうたら", "さったら", &[], &[], RuleType::Suffix),
                    inflection("ぞうたら", "ざったら", &[], &[], RuleType::Suffix),
                    inflection("とうたら", "たったら", &[], &[], RuleType::Suffix),
                    inflection("どうたら", "だったら", &[], &[], RuleType::Suffix),
                    inflection("のうたら", "なったら", &[], &[], RuleType::Suffix),
                    inflection("ほうたら", "はったら", &[], &[], RuleType::Suffix),
                    inflection("ぼうたら", "ばったら", &[], &[], RuleType::Suffix),
                    inflection("もうたら", "まったら", &[], &[], RuleType::Suffix),
                    inflection("ろうたら", "らったら", &[], &[], RuleType::Suffix),
                    inflection("ようたら", "やったら", &[], &[], RuleType::Suffix),
                    inflection("ゆうたら", "いったら", &[], &[], RuleType::Suffix),
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
                    inflection("うたり", "ったり", &[], &[], RuleType::Suffix),
                    inflection("おうたり", "あったり", &[], &[], RuleType::Suffix),
                    inflection("こうたり", "かったり", &[], &[], RuleType::Suffix),
                    inflection("ごうたり", "がったり", &[], &[], RuleType::Suffix),
                    inflection("そうたり", "さったり", &[], &[], RuleType::Suffix),
                    inflection("ぞうたり", "ざったり", &[], &[], RuleType::Suffix),
                    inflection("とうたり", "たったり", &[], &[], RuleType::Suffix),
                    inflection("どうたり", "だったり", &[], &[], RuleType::Suffix),
                    inflection("のうたり", "なったり", &[], &[], RuleType::Suffix),
                    inflection("ほうたり", "はったり", &[], &[], RuleType::Suffix),
                    inflection("ぼうたり", "ばったり", &[], &[], RuleType::Suffix),
                    inflection("もうたり", "まったり", &[], &[], RuleType::Suffix),
                    inflection("ろうたり", "らったり", &[], &[], RuleType::Suffix),
                    inflection("ようたり", "やったり", &[], &[], RuleType::Suffix),
                    inflection("ゆうたり", "いったり", &[], &[], RuleType::Suffix),
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
                    inflection("う", "く", &[], &["-く"], RuleType::Suffix),
                    inflection("こう", "かく", &[], &["-く"], RuleType::Suffix),
                    inflection("ごう", "がく", &[], &["-く"], RuleType::Suffix),
                    inflection("そう", "さく", &[], &["-く"], RuleType::Suffix),
                    inflection("とう", "たく", &[], &["-く"], RuleType::Suffix),
                    inflection("のう", "なく", &[], &["-く"], RuleType::Suffix),
                    inflection("ぼう", "ばく", &[], &["-く"], RuleType::Suffix),
                    inflection("もう", "まく", &[], &["-く"], RuleType::Suffix),
                    inflection("ろう", "らく", &[], &["-く"], RuleType::Suffix),
                    inflection("よう", "よく", &[], &["-く"], RuleType::Suffix),
                    inflection("しゅう", "しく", &[], &["-く"], RuleType::Suffix),
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
                    inflection("うて", "くて", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("こうて", "かくて", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ごうて", "がくて", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("そうて", "さくて", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("とうて", "たくて", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("のうて", "なくて", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ぼうて", "ばくて", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("もうて", "まくて", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ろうて", "らくて", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("ようて", "よくて", &["-て"], &["-て"], RuleType::Suffix),
                    inflection("しゅうて", "しくて", &["-て"], &["-て"], RuleType::Suffix),
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
                    inflection("うない", "くない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("こうない", "かくない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("ごうない", "がくない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("そうない", "さくない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("とうない", "たくない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("のうない", "なくない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("ぼうない", "ばくない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("もうない", "まくない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("ろうない", "らくない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("ようない", "よくない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                    inflection("しゅうない", "しくない", &["adj-i"], &["adj-i"], RuleType::Suffix),
                ],
            },
        ),
    ]))
});

#[rustfmt::skip]
pub(crate) static JP_CONDITIONS_MAP: LazyLock<ConditionMap> = LazyLock::new(|| {    ConditionMap(IndexMap::from([            (                "v",                Condition {                    name: "Verb",                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "動詞",                    }]),                    sub_conditions: Some(&[                        "v1",                        "v5",                        "vk",                        "vs",                        "vz",                    ], ),                },            ),            (                "v1",                Condition {                    name: "Ichidan verb",                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "一段動詞",                    }]),                    sub_conditions: Some(&["v1d", "v1p"]),                    },                ),            (                "v1d",                Condition {                    name: "Ichidan verb, dictionary form",                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "一段動詞、辞書形",                    }]),                    sub_conditions: None,                },            ),            (                "v1p",                Condition {                    name: "Ichidan verb, progressive or perfect form",                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "一段動詞、～てる・でる",                    }], ),                    sub_conditions: None,                },            ),            (                "v5",                Condition {                    name: "Godan verb",                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "五段動詞",                    }], ),                    sub_conditions: Some(&["v5d", "v5s"], ),                },            ),            (                "v5d",                Condition {                    name: "Godan verb, dictionary form",                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "五段動詞、終止形",                    }], ),                    sub_conditions: None,                },            ),            (                "v5s",                Condition {                    name: "Godan verb, short causative form",                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "五段動詞、～す・さす",                    }], ),                    sub_conditions: Some(&["v5ss", "v5sp"], ),                },            ),            (                "v5ss",                Condition {                    name: "Godan verb, short causative form having さす ending (cannot conjugate with passive form)",                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "五段動詞、～さす",                    }], ),                    sub_conditions: None,                },            ),            (                "v5sp",                Condition {                    name: "Godan verb, short causative form not having さす ending (can conjugate with passive form)",                    is_dictionary_form: false,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "五段動詞、～す",                    }], ),                    sub_conditions: None,                },            ),            (                "vk",                Condition {                    name: "Kuru verb",                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "来る動詞",                    }], ),                    sub_conditions: None,                },            ),            (                "vs",                Condition {                    name: "Suru verb",                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "する動詞",                    }], ),                    sub_conditions: None,                },            ),            (                "vz",                Condition {                    name: "Zuru verb",                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "ずる動詞",                    }], ),                    sub_conditions: None,                },            ),            (                "adj-i",                Condition {                    name: "Adjective with i ending",                    is_dictionary_form: true,                    i18n: Some(vec![RuleI18n {                        language: "ja",                        name: "形容詞",                    }], ),                    sub_conditions: None,                },            ),            (                "-ます",                Condition {                    name: "Polite -ます ending",                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-ません",                Condition {                    name: "Polite negative -ません ending",                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-て",                Condition {                    name: "Intermediate -て endings for progressive or perfect tense",                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-ば",                Condition {                    name: "Intermediate -ば endings for conditional contraction",                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-く",                Condition {                    name: "Intermediate -く endings for adverbs",                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-た",                Condition {                    name: "-た form ending",                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-ん",                Condition {                    name: "-ん negative ending",                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-なさい",                Condition {                    name: "Intermediate -なさい ending (polite imperative)",                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),            (                "-ゃ",                Condition {                    name: "Intermediate -や ending (conditional contraction)",                    is_dictionary_form: false,                    i18n: None,                    sub_conditions: None,                },            ),        ], ))});

use crate::transformer::{DeinflectFnType, Rule, RuleType, SuffixRule};
use fancy_regex::Regex;
use std::sync::Arc;

pub fn inflection(
    inflected: &str,
    deinflected: &'static str,
    conditions_in: &'static [&'static str],
    conditions_out: &'static [&'static str],
    rule_type: RuleType,
) -> Rule {
    let regx = match rule_type {
        RuleType::Prefix => format!("^{inflected}"),
        RuleType::Suffix => format!("{inflected}$"),
        RuleType::WholeWord => format!("^{inflected}$"),
        _ => panic!(
            "{rule_type:?} is invalid, only RuleType Suffix, Prefix && WholeWord work with this fn"
        ),
    };
    let deinflect_fn = match rule_type {
        RuleType::Suffix => DeinflectFnType::GenericSuffix,
        RuleType::Prefix => DeinflectFnType::GenericPrefix,
        RuleType::WholeWord => DeinflectFnType::GenericWholeWord,
        _ => panic!(
            "{rule_type:?} is invalid, only RuleType Suffix, Prefix && WholeWord work with this fn"
        ),
    };
    let is_inflected = Regex::new(&regx).unwrap();
    let deinflected = if deinflected.is_empty() {
        None
    } else {
        Some(deinflected)
    };
    Rule {
        rule_type,
        is_inflected,
        deinflected,
        deinflect_fn,
        inflected_str: Some(inflected.to_string()),
        conditions_in,
        conditions_out,
    }
}

/// Creates a Rule for stem-changing verbs (e.g., e -> ie).
/// These rules are always `RuleType::Other`.
pub fn generic_stem_change_rule(
    stem_from: &'static str,
    stem_to: &'static str,
    // The regex pattern for the ending, WITHOUT the anchor. e.g., "(o|as|a|an)"
    ending_pattern: &'static str,
    ending_to: &'static str,
    conditions_in: &'static [&'static str],
    conditions_out: &'static [&'static str],
) -> Rule {
    // 1. Create the full regex pattern to check if a word is inflected.
    // e.g., "ie([a-z]*)(o|as|a|an)$"
    let is_inflected_re = format!("{stem_from}([a-z]*){ending_pattern}$");
    let is_inflected = Regex::new(&is_inflected_re).unwrap();

    // 2. Define the deinflection logic by creating the parameterized enum variant.
    // The `deinflect` function itself will need the anchor, so we add it here.
    let deinflect_fn = DeinflectFnType::GenericStemChange {
        stem_from,
        stem_to,
        ending_re: format!("{ending_pattern}$").leak(),
        ending_to,
    };

    // 3. Assemble the final Rule struct.
    Rule {
        rule_type: RuleType::Other,
        is_inflected,
        // This type of rule de-inflects programmatically, not to a single word.
        deinflected: None,
        deinflect_fn,
        // The "source" of the regex, without the final anchor.
        inflected_str: Some(is_inflected_re.strip_suffix('$').unwrap().to_string()),
        conditions_in,
        conditions_out,
    }
}

/// Creates a Rule for stem-changing verbs that have a special case (e.g., "jugar", "oler").
/// These rules are always `RuleType::Other`.
pub fn special_cased_stem_change_rule(
    // The stem used for the initial `is_inflected` check, e.g., "ue"
    inflected_stem: &'static str,
    prefix: &'static str,
    special_stem_from: &'static str,
    special_stem_to: &'static str,
    default_stem_from: &'static str,
    default_stem_to: &'static str,
    ending_pattern: &'static str,
    ending_to: &'static str,
    conditions_in: &'static [&'static str],
    conditions_out: &'static [&'static str],
) -> Rule {
    // 1. Create the `is_inflected` regex. e.g., "ue([a-z]*)(o|as|a|an)$"
    let is_inflected_re = format!("{inflected_stem}([a-z]*){ending_pattern}$");
    let is_inflected = Regex::new(&is_inflected_re).unwrap();

    // 2. Define the deinflection logic with the special case parameters.
    let deinflect_fn = DeinflectFnType::SpecialCasedStemChange {
        prefix,
        special_stem_from,
        special_stem_to,
        default_stem_from,
        default_stem_to,
        ending_re: format!("{ending_pattern}$").leak(),
        ending_to,
    };

    // 3. Assemble the final Rule.
    Rule {
        rule_type: RuleType::Other,
        is_inflected,
        deinflected: None,
        deinflect_fn,
        inflected_str: Some(is_inflected_re.strip_suffix('$').unwrap().to_string()),
        conditions_in,
        conditions_out,
    }
}

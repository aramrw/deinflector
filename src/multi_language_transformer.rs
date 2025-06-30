use std::collections::HashMap;

use crate::{
    languages::get_all_language_transform_descriptors,
    transformer::{InflectionRule, InflectionRuleChain, LanguageTransformer, TransformedText},
};

// key: language (ie: "en", "ja")
// valueL LanguageTransformer
#[derive(Clone, Debug)]
pub struct MultiLanguageTransformer {
    inner: HashMap<&'static str, LanguageTransformer>,
}

impl Default for MultiLanguageTransformer {
    fn default() -> Self {
        let mut mlt = Self {
            inner: HashMap::default(),
        };
        mlt.prepare();
        mlt
    }
}

impl MultiLanguageTransformer {
    fn prepare(&mut self) {
        let langs = get_all_language_transform_descriptors();
        for transforms in langs {
            let mut lt = LanguageTransformer::new();
            let descriptor = transforms.language_transforms;
            lt.add_descriptor(&descriptor).unwrap();
            let language = descriptor.language;
            self.inner.insert(language, lt);
        }
    }

    pub fn get_condition_flags_from_parts_of_speech(
        &self,
        language: &str,
        parts_of_speech: &[String],
    ) -> usize {
        self.inner
            .get(language)
            .map(|lt| lt.get_condition_flags_from_parts_of_speech(parts_of_speech))
            .unwrap_or(0)
    }

    pub fn get_condition_flags_from_condition_types(
        &self,
        language: &str,
        condition_types: &[String],
    ) -> usize {
        self.inner
            .get(language)
            .map(|lt| lt.get_condition_flags_from_condition_types(condition_types))
            .unwrap_or(0)
    }

    pub fn get_condition_flags_from_condition_type(
        &self,
        language: &str,
        condition_type: &str,
    ) -> usize {
        self.inner
            .get(language)
            .map(|lt| lt.get_condition_flags_from_single_condition_type(condition_type))
            .unwrap_or(0)
    }

    pub fn transform(&self, language: &str, source_text: &str) -> Vec<TransformedText> {
        match self.inner.get(language) {
            Some(lt) => lt.transform(source_text),
            None => vec![TransformedText::create_transformed_text(
                source_text.to_owned(),
                0,
                Vec::new(),
            )],
        }
    }

    pub fn get_user_facing_inflection_rules(
        &self,
        language: &str,
        inflection_rules: &[String],
    ) -> InflectionRuleChain {
        match self.inner.get(language) {
            Some(lt) => lt.get_user_facing_inflection_rules(inflection_rules),
            None => inflection_rules
                .iter()
                .map(|rule| InflectionRule {
                    name: rule.clone(),
                    description: None,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod mlt {
    use crate::transformer::{Trace, TraceFrame, TransformedText};

    use super::MultiLanguageTransformer;
    use pretty_assertions::assert_eq as passert_eq;

    #[test]
    fn transform_jp() {
        let json: &str = include_str!("../tests/multi_language_transformer/transform.json");
        let expected: Vec<TransformedText> = serde_json::from_str(json).unwrap();
        let mlt = MultiLanguageTransformer::default();
        let res = mlt.transform("ja", "流れて");
        passert_eq!(res, expected);
        dbg!(res);
    }

    #[test]
    fn transform_es() {
        let mlt = MultiLanguageTransformer::default();
        let res = mlt.transform("es", "bueno");
        dbg!(res);
    }
}

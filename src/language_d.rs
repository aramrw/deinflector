use std::hash::Hash;

use fancy_regex::Regex;

use crate::transformer::LanguageTransformDescriptor;

/// This is the following function type in yomitan:
/// export type TextProcessorFunction<T = unknown> = (str: string, setting: T) => string;
trait TextProcessable<T> {
    fn process(str: &str, options: Vec<T>) -> String;
}

/// Information about how text should be replaced when looking up terms.
#[derive(Debug, Clone)]
pub struct FindTermsTextReplacement {
    pub pattern: Regex,
    /// The replacement string. This can contain special sequences, such as `$&`.
    pub replacement: String,
    pub is_global: bool,
}
impl PartialEq for FindTermsTextReplacement {
    fn eq(&self, other: &Self) -> bool {
        if self.pattern.as_str() == other.pattern.as_str() && self.replacement == other.replacement
        {
            return true;
        }
        false
    }
}
impl Eq for FindTermsTextReplacement {}
impl Hash for FindTermsTextReplacement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pattern.as_str().hash(state);
        self.replacement.as_str().hash(state);
    }
}

/// Multiple text replacements.
/// This was (FindTermsTextReplacement[] | null)[]
/// Which means an array of (array of replacements OR null).
/// In Rust, this translates to Vec<Option<Vec<FindTermsTextReplacement>>>
pub type FindTermsTextReplacements = Vec<Option<Vec<FindTermsTextReplacement>>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextDeinflectionOptions {
    pub text_replacements: Option<Vec<FindTermsTextReplacement>>,
    pub half_width: bool,
    pub numeric: bool,
    pub alphabetic: bool,
    pub katakana: bool,
    pub hiragana: bool,
    /// [collapse_emphatic, collapse_emphatic_full]
    pub emphatic: (bool, bool),
}

#[derive(Debug, Clone)]
pub struct TextDeinflectionOptionsArrays {
    pub text_replacements: Vec<Option<Vec<FindTermsTextReplacement>>>,
    pub half_width: Vec<bool>,
    pub numeric: Vec<bool>,
    pub alphabetic: Vec<bool>,
    pub katakana: Vec<bool>,
    pub hiragana: Vec<bool>,
    pub emphatic: Vec<(bool, bool)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextProcessorSetting {
    Bool(bool),
    Int(i64),
    String(String),
    Emphatic(bool, bool),
    Deinflection(TextDeinflectionOptions),
    BiDirectional(BidirectionalPreProcessorOptions),
}

/// Text `pre-` & `post-`processors are used during the translation process to
/// create alternate versions of the input text to search for.
///
/// This can be helpful when the input text don't exactly
/// match the term or expression found in the database.
///
/// When a language has multiple processors, the translator will generate
/// variants of the text by applying all combinations of the processors.
#[derive(Debug, Clone)]
pub struct TextProcessor {
    pub name: &'static str,
    pub description: &'static str,
    pub options: &'static [TextProcessorSetting],
    pub process: fn(&str, TextProcessorSetting) -> String,
}

pub type TextProcessorFn<T> = fn(&str, T) -> String;

/// Helper function to normalize .
pub type ReadingNormalizer = fn(&str) -> String;

#[derive(Debug, Clone)]
pub enum AnyTextProcessor {
    // Japanese Processors
    ConvertHalfWidth(TextProcessor),
    AlphabeticToHiragana(TextProcessor),
    NormalizeCombiningCharacters(TextProcessor),
    NormalizeCjkCompatibilityCharacters(TextProcessor),
    NormalizeRadicalCharacters(TextProcessor),
    StandardizeKanji(TextProcessor),
    AlphanumericWidth(BidirectionalConversionPreProcessor),
    HiraganaToKatakana(BidirectionalConversionPreProcessor),
    CollapseEmphatic(TextProcessor),

    // English Processors
    Decapitalize(TextProcessor),
    CapitalizeFirst(TextProcessor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BidirectionalPreProcessorOptions {
    Off,
    Direct,
    Inverse,
}

/// TextProcessor<BidirectionalPreProcessorOptions, BidirectionalPreProcessorOptions>;
pub type BidirectionalConversionPreProcessor = TextProcessor;

pub enum AllTextProcessorsEnum {}

pub struct LanguageAndProcessors {
    pub iso: &'static str,
    pub pre: Vec<TextProcessorWithId>,
    pub post: Vec<TextProcessorWithId>,
}

pub struct LanguageAndReadingNormalizer {
    pub iso: &'static str,
    pub reading_normalizer: ReadingNormalizer,
}

#[derive(Debug, Clone)]
pub struct TextProcessorWithId {
    pub id: &'static str,
    pub processor: TextProcessor,
}

pub struct LanguageAndTransforms {
    pub iso: &'static str,
    pub language_transforms: LanguageTransformDescriptor,
}

#[derive(Debug, Clone)]
pub struct LanguageSummary {
    pub name: &'static str,
    pub iso: &'static str,
    pub iso639_3: &'static str,
    pub example_text: &'static str,
}

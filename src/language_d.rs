use crate::transformer::LanguageTransformDescriptor;

/// This is the following function type in yomitan:
/// export type TextProcessorFunction<T = unknown> = (str: string, setting: T) => string;
trait TextProcessable<T> {
    fn process(str: &str, options: Vec<T>) -> String;
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
pub struct TextProcessor<Opts: 'static, Fn> {
    pub name: &'static str,
    pub description: &'static str,
    pub options: &'static [Opts],
    pub process: TextProcessorFn<Fn>,
}

pub type TextProcessorFn<T> = fn(&str, T) -> String;

/// Helper function to normalize .
pub type ReadingNormalizer = fn(&str) -> String;

#[derive(Debug, Clone)]
pub enum AnyTextProcessor {
    // Japanese Processors
    ConvertHalfWidth(TextProcessor<bool, bool>),
    AlphabeticToHiragana(TextProcessor<bool, bool>),
    NormalizeCombiningCharacters(TextProcessor<bool, bool>),
    NormalizeCjkCompatibilityCharacters(TextProcessor<bool, bool>),
    NormalizeRadicalCharacters(TextProcessor<bool, bool>),
    StandardizeKanji(TextProcessor<bool, bool>),
    AlphanumericWidth(BidirectionalConversionPreProcessor),
    HiraganaToKatakana(BidirectionalConversionPreProcessor),
    CollapseEmphatic(TextProcessor<[bool; 2], [bool; 2]>),

    // English Processors
    Decapitalize(TextProcessor<bool, bool>),
    CapitalizeFirst(TextProcessor<bool, bool>),
}

#[derive(Debug, Clone)]
pub enum BidirectionalPreProcessorOptions {
    Off,
    Direct,
    Inverse,
}

pub type BidirectionalConversionPreProcessor =
    TextProcessor<BidirectionalPreProcessorOptions, BidirectionalPreProcessorOptions>;

pub enum AllTextProcessorsEnum {}

pub struct LanguageAndProcessors {
    pub iso: String,
    pub preprocessors: Vec<TextProcessorWithId>,
    pub postprocessors: Vec<TextProcessorWithId>,
}

pub struct LanguageAndReadingNormalizer {
    pub iso: &'static str,
    pub reading_normalizer: ReadingNormalizer,
}

#[derive(Debug, Clone)]
pub struct TextProcessorWithId {
    pub id: &'static str,
    pub processor: AnyTextProcessor,
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

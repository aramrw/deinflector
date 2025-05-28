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
pub struct TextProcessor<O: 'static, S> {
    pub name: &'static str,
    pub description: &'static str,
    pub options: &'static [O],
    pub process: TextProcessorFP<S>,
}

pub type TextProcessorFP<T> = fn(&str, T) -> String;

/// Helper function to normalize .
pub type ReadingNormalizer = fn(&str) -> String;

#[derive(Debug, Clone)]
pub enum BidirectionalPreProcessorOptions {
    Off,
    Direct,
    Inverse,
}

pub type BidirectionalConversionPreProcessor =
    TextProcessor<BidirectionalPreProcessorOptions, BidirectionalPreProcessorOptions>;

pub struct LanguageAndProcessors<O: 'static, S> {
    pub iso: String,
    pub text_preprocessors: Option<Vec<TextProcessorWithId<O, S>>>,
    pub text_postprocessors: Option<Vec<TextProcessorWithId<O, S>>>,
}

pub struct LanguageAndReadingNormalizer {
    pub iso: &'static str,
    pub reading_normalizer: ReadingNormalizer,
}

pub struct LanguageAndTransforms {
    pub iso: &'static str,
    pub language_transforms: LanguageTransformDescriptor,
}

pub struct TextProcessorWithId<O: 'static, S> {
    pub id: String,
    pub text_processor: TextProcessor<O, S>,
}

#[derive(Debug, Clone)]
pub struct LanguageSummary {
    pub name: &'static str,
    pub iso: &'static str,
    pub iso639_3: &'static str,
    pub example_text: &'static str,
}

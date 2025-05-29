// In a real project, you would add these to your Cargo.toml:
// unicode-normalization = "0.1"
// regex = "1"
// once_cell = "1" // For lazy static Regex compilation

use std::sync::LazyLock;

use fancy_regex::Regex;
use unicode_normalization::{is_nfc, UnicodeNormalization}; // For NFC normalization

/// Represents a range of Unicode code points [start, end] inclusive.
pub type CodepointRange = (u32, u32);

// These would typically come from your CJK_util module/crate
pub const CJK_IDEOGRAPH_RANGES: &[CodepointRange] = &[
    (0x4E00, 0x9FFF), // CJK Unified Ideographs
    (0x3400, 0x4DBF), // CJK Unified Ideographs Extension A
                      // ... other CJK ideograph ranges
];
pub const CJK_PUNCTUATION_RANGE: CodepointRange = (0x3000, 0x303F);
pub const FULLWIDTH_CHARACTER_RANGES: &[CodepointRange] = &[
    (0xFF00, 0xFFEF), // Halfwidth and Fullwidth Forms
                      // ... other fullwidth ranges
];

pub const BOPOMOFO_RANGE: CodepointRange = (0x3100, 0x312f);
pub const BOPOMOFO_EXTENDED_RANGE: CodepointRange = (0x31a0, 0x31bf);
pub const IDEOGRAPHIC_SYMBOLS_AND_PUNCTUATION_RANGE: CodepointRange = (0x16fe0, 0x16fff);
pub const SMALL_FORM_RANGE: CodepointRange = (0xfe50, 0xfe6f);
pub const VERTICAL_FORM_RANGE: CodepointRange = (0xfe10, 0xfe1f);

/// Chinese character ranges, roughly ordered in order of expected frequency.
/// In Rust, const array initialization doesn't have a direct spread operator for other const arrays.
/// We list them explicitly or use a macro if combining many such arrays.
/// For simplicity, we'll construct it by listing.
pub const CHINESE_RANGES: &[CodepointRange] = &[
    // ...CJK_IDEOGRAPH_RANGES (manually list them or use a build script/macro to generate)
    (0x4E00, 0x9FFF), // CJK Unified Ideographs (example from CJK_IDEOGRAPH_RANGES)
    (0x3400, 0x4DBF), // CJK Unified Ideographs Extension A (example from CJK_IDEOGRAPH_RANGES)
    CJK_PUNCTUATION_RANGE,
    // ...FULLWIDTH_CHARACTER_RANGES (manually list them)
    (0xFF00, 0xFFEF), // Halfwidth and Fullwidth Forms (example from FULLWIDTH_CHARACTER_RANGES)
    BOPOMOFO_RANGE,
    BOPOMOFO_EXTENDED_RANGE,
    IDEOGRAPHIC_SYMBOLS_AND_PUNCTUATION_RANGE,
    SMALL_FORM_RANGE,
    VERTICAL_FORM_RANGE,
];

/// Checks if a given code point falls within any of the specified ranges.
///
/// # Arguments
/// * `code_point` - The Unicode code point (as u32) to check.
/// * `ranges` - A slice of `CodepointRange` tuples.
///
/// # Returns
/// `true` if the code point is in any of the ranges, `false` otherwise.
pub fn is_code_point_in_ranges(code_point: u32, ranges: &[CodepointRange]) -> bool {
    for &(start, end) in ranges {
        if code_point >= start && code_point <= end {
            return true;
        }
    }
    false
}

/// Checks if a string contains at least one Chinese character.
///
/// # Arguments
/// * `s` - The string slice to check.
///
/// # Returns
/// `true` if any character in the string is considered Chinese, `false` otherwise.
pub fn is_string_partially_chinese(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    for c in s.chars() {
        if is_code_point_chinese(c as u32) {
            return true;
        }
    }
    false
}

/// Checks if a given Unicode code point is considered a Chinese character.
///
/// # Arguments
/// * `code_point` - The Unicode code point (as u32).
///
/// # Returns
/// `true` if the code point is within the defined Chinese character ranges, `false` otherwise.
pub fn is_code_point_chinese(code_point: u32) -> bool {
    is_code_point_in_ranges(code_point, CHINESE_RANGES)
}

// For normalize_pinyin, we need a regex. It's good practice to compile it once.
// `once_cell::sync::Lazy` is great for this.
static PINYIN_CLEANUP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\s・:'’-]|\/\/").unwrap() // In JS: /[\s・:'’-]|\/\//g
});

/// Normalizes a Pinyin string.
/// This involves NFC normalization, converting to lowercase, and removing specific characters.
///
/// # Arguments
/// * `s` - The Pinyin string slice to normalize.
///
/// # Returns
/// A `String` containing the normalized Pinyin.
pub fn normalize_pinyin(s: &str) -> String {
    // 1. Normalize to NFC
    // The .nfc() iterator produces characters, so we collect them into a String.
    let normalized_s: String = s.nfc().collect();

    // 2. Convert to lowercase
    let lowercased_s = normalized_s.to_lowercase();

    // 3. Replace specified characters with an empty string
    // The regex crate's replace_all returns a Cow<str> (Clone-on-Write string).
    // We convert it to an owned String.
    PINYIN_CLEANUP_REGEX
        .replace_all(&lowercased_s, "")
        .into_owned()
}

mod zh_tests {
    use unicode_normalization::{is_nfc, UnicodeNormalization};

    use crate::zh::chinese::{
        is_code_point_chinese, is_string_partially_chinese, normalize_pinyin,
    };

    fn zhtest() {
        println!(
            "Is '你好世界' partially Chinese? {}",
            is_string_partially_chinese("你好世界")
        ); // true
        println!(
            "Is 'Hello' partially Chinese? {}",
            is_string_partially_chinese("Hello")
        ); // false
        println!("Is '世' Chinese? {}", is_code_point_chinese('世' as u32)); // true
        println!("Is 'A' Chinese? {}", is_code_point_chinese('A' as u32)); // false

        let pinyin1 = "Nǐ hǎo";
        let pinyin2 = "ni³ hao³ // comment";
        let pinyin3 = "Pīn・yīn: 'test' - example";
        println!("'{}' -> '{}'", pinyin1, normalize_pinyin(pinyin1)); // "nǐhǎo"
        println!("'{}' -> '{}'", pinyin2, normalize_pinyin(pinyin2)); // "ni³hao³"
        println!("'{}' -> '{}'", pinyin3, normalize_pinyin(pinyin3)); // "pīnyīntestexample"

        // Check if a specific character is NFC (just an example of using the unicode-normalization crate)
        let precomposed = "é"; // U+00E9
        let decomposed = "é"; // U+0065 U+0301
        println!("'{}' is NFC: {}", precomposed, is_nfc(precomposed)); // true
        println!("'{}' is NFC: {}", decomposed, is_nfc(decomposed)); // false
        let nfc_decomposed: String = decomposed.nfc().collect();
        println!("Decomposed '{decomposed}' to NFC: '{nfc_decomposed}'"); // "é"
    }
}

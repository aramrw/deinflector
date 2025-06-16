use std::{
    cmp,
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

// NOTE: You will need to add `unicode_normalization = "0.1.22"` (or latest) to your Cargo.toml
use unicode_normalization::UnicodeNormalization;

// Import from the (now fixed) cjk_utils
use crate::cjk_utils::{
    is_code_point_in_range, is_code_point_in_ranges, CodepointRange,
    CJK_COMPATIBILITY_IDEOGRAPHS_RANGE, CJK_IDEOGRAPH_RANGES,
};

pub const HIRAGANA_SMALL_TSU_CODE_POINT: u32 = 0x3063;
pub const KATAKANA_SMALL_TSU_CODE_POINT: u32 = 0x30c3;
pub const KATAKANA_SMALL_KA_CODE_POINT: u32 = 0x30f5;
pub const KATAKANA_SMALL_KE_CODE_POINT: u32 = 0x30f6;
pub const KANA_PROLONGED_SOUND_MARK_CODE_POINT: u32 = 0x30fc;

pub const HIRAGANA_CONVERSION_RANGE: CodepointRange = (0x3041, 0x3096);
pub const KATAKANA_CONVERSION_RANGE: CodepointRange = (0x30a1, 0x30f6);

pub const HIRAGANA_RANGE: CodepointRange = (0x3040, 0x309f);
pub const KATAKANA_RANGE: CodepointRange = (0x30a0, 0x30ff);

pub const KANA_RANGES: &[CodepointRange] = &[HIRAGANA_RANGE, KATAKANA_RANGE];

// Matches JS order and content
pub static JAPANESE_RANGES: LazyLock<Vec<CodepointRange>> = LazyLock::new(|| {
    vec![
        HIRAGANA_RANGE,
        KATAKANA_RANGE,
        CJK_IDEOGRAPH_RANGES[0],  // CJK Unified
        CJK_IDEOGRAPH_RANGES[1],  // CJK Unified Ext A
        CJK_IDEOGRAPH_RANGES[2],  // CJK Unified Ext B
        CJK_IDEOGRAPH_RANGES[3],  // CJK Unified Ext C
        CJK_IDEOGRAPH_RANGES[4],  // CJK Unified Ext D
        CJK_IDEOGRAPH_RANGES[5],  // CJK Unified Ext E
        CJK_IDEOGRAPH_RANGES[6],  // CJK Unified Ext F
        CJK_IDEOGRAPH_RANGES[7],  // CJK Unified Ext G
        CJK_IDEOGRAPH_RANGES[8],  // CJK Unified Ext H
        CJK_IDEOGRAPH_RANGES[9],  // CJK Unified Ext I
        (0xff66, 0xff9f),         // Halfwidth katakana
        (0x30fb, 0x30fc),         // Katakana punctuation
        (0xff61, 0xff65),         // Kana punctuation
        (0x3000, 0x303f),         // CJK punctuation
        (0xff10, 0xff19),         // Fullwidth numbers
        (0xff21, 0xff3a),         // Fullwidth upper case Latin letters
        (0xff41, 0xff5a),         // Fullwidth lower case Latin letters
        (0xff01, 0xff0f),         // Fullwidth punctuation 1
        (0xff1a, 0xff1f),         // Fullwidth punctuation 2
        (0xff3b, 0xff3f),         // Fullwidth punctuation 3
        (0xff5b, 0xff60),         // Fullwidth punctuation 4
        (0xffe0, 0xffee),         // Currency markers
        CJK_IDEOGRAPH_RANGES[10], // CJK Compatibility
        CJK_IDEOGRAPH_RANGES[11], // CJK Compatibility Sup
    ]
});

pub static SMALL_KANA_SET: LazyLock<HashSet<char>> = LazyLock::new(|| {
    HashSet::from([
        'ぁ', 'ぃ', 'ぅ', 'ぇ', 'ぉ', 'ゃ', 'ゅ', 'ょ', 'ゎ', 'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ャ',
        'ュ', 'ョ', 'ヮ',
    ])
});

// Fixed to match JS structure with '---' placeholders
#[rustfmt::skip]
pub static HALFWIDTH_KATAKANA_MAP: LazyLock<HashMap<char, &str>> = LazyLock::new(|| {
    HashMap::from([
        ('･', "・--"), ('ｦ', "ヲヺ-"), ('ｧ', "ァ--"), ('ｨ', "ィ--"), ('ｩ', "ゥ--"),
        ('ｪ', "ェ--"), ('ｫ', "ォ--"), ('ｬ', "ャ--"), ('ｭ', "ュ--"), ('ｮ', "ョ--"),
        ('ｯ', "ッ--"), ('ｰ', "ー--"), ('ｱ', "ア--"), ('ｲ', "イ--"), ('ｳ', "ウヴ-"),
        ('ｴ', "エ--"), ('ｵ', "オ--"), ('ｶ', "カガ-"), ('ｷ', "キギ-"), ('ｸ', "クグ-"),
        ('ｹ', "ケゲ-"), ('ｺ', "コゴ-"), ('ｻ', "サザ-"), ('ｼ', "シジ-"), ('ｽ', "スズ-"),
        ('ｾ', "セゼ-"), ('ｿ', "ソゾ-"), ('ﾀ', "タダ-"), ('ﾁ', "チヂ-"), ('ﾂ', "ツヅ-"),
        ('ﾃ', "テデ-"), ('ﾄ', "トド-"), ('ﾅ', "ナ--"), ('ﾆ', "ニ--"), ('ﾇ', "ヌ--"),
        ('ﾈ', "ネ--"), ('ﾉ', "ノ--"), ('ﾊ', "ハバパ"), ('ﾋ', "ヒビピ"), ('ﾌ', "フブプ"),
        ('ﾍ', "ヘベペ"), ('ﾎ', "ホボポ"), ('ﾏ', "マ--"), ('ﾐ', "ミ--"), ('ﾑ', "ム--"),
        ('ﾒ', "メ--"), ('ﾓ', "モ--"), ('ﾔ', "ヤ--"), ('ﾕ', "ユ--"), ('ﾖ', "ヨ--"),
        ('ﾗ', "ラ--"), ('ﾘ', "リ--"), ('ﾙ', "ル--"), ('ﾚ', "レ--"), ('ﾛ', "ロ--"),
        ('ﾜ', "ワ--"), ('ﾝ', "ン--"),
    ])
});

#[rustfmt::skip]
static VOWEL_TO_KANA_MAPPING: LazyLock<HashMap<char, &str>> = LazyLock::new(|| {
    HashMap::from([
        ('a', "ぁあかがさざただなはばぱまゃやらゎわヵァアカガサザタダナハバパマャヤラヮワヵヷ"),
        ('i', "ぃいきぎしじちぢにひびぴみりゐィイキギシジチヂニヒビピミリヰヸ"),
        ('u', "ぅうくぐすずっつづぬふぶぷむゅゆるゥウクグスズッツヅヌフブプムュユルヴ"),
        ('e', "ぇえけげせぜてでねへべぺめれゑヶェエケゲセゼテデネヘベペメレヱヶヹ"),
        ('o', "ぉおこごそぞとどのほぼぽもょよろをォオコゴソゾトドノホボポモョヨロヲヺ"),
        ('_', "のノ"),
    ])
});

pub static KANA_TO_VOWEL_MAPPING: LazyLock<HashMap<char, char>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (&vowel, characters) in VOWEL_TO_KANA_MAPPING.iter() {
        for char in characters.chars() {
            map.insert(char, vowel);
        }
    }
    map
});

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuriganaGroup {
    pub is_kana: bool,
    pub text: String,
    pub text_normalized: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuriganaSegment {
    pub text: String,
    pub reading: Option<String>,
}

impl FuriganaSegment {
    pub fn create_furigana_segment(text: String, reading: Option<String>) -> Self {
        let final_reading = reading.and_then(|r| if r.is_empty() { None } else { Some(r) });
        Self {
            text,
            reading: final_reading,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PitchCategory {
    Heiban,
    Kifuku,
    Atamadaka,
    Odaka,
    Nakadaka,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiacriticType {
    Dakuten,
    Handakuten,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiacriticInfo {
    pub character: char,
    pub diacritic_type: DiacriticType,
}

pub static DIACRITIC_MAPPING: LazyLock<HashMap<char, DiacriticInfo>> = LazyLock::new(|| {
    const KANA: &str = "うゔ-かが-きぎ-くぐ-けげ-こご-さざ-しじ-すず-せぜ-そぞ-ただ-ちぢ-つづ-てで-とど-はばぱひびぴふぶぷへべぺほぼぽワヷ-ヰヸ-ウヴ-ヱヹ-ヲヺ-カガ-キギ-クグ-ケゲ-コゴ-サザ-シジ-スズ-セゼ-ソゾ-タダ-チヂ-ツヅ-テデ-トド-ハバパヒビピフブプヘベペホボポ";
    let mut map = HashMap::new();
    let chars: Vec<char> = KANA.chars().collect();
    for chunk in chars.chunks(3) {
        if let [character, dakuten, handakuten] = *chunk {
            map.insert(
                dakuten,
                DiacriticInfo {
                    character,
                    diacritic_type: DiacriticType::Dakuten,
                },
            );
            if handakuten != '-' {
                map.insert(
                    handakuten,
                    DiacriticInfo {
                        character,
                        diacritic_type: DiacriticType::Handakuten,
                    },
                );
            }
        }
    }
    map
});

// Fixed to match JS logic and return Option<char>
fn get_prolonged_hiragana(prev_char: char) -> Option<char> {
    match KANA_TO_VOWEL_MAPPING.get(&prev_char) {
        Some('a') => Some('あ'),
        Some('i') => Some('い'),
        Some('u') => Some('う'),
        Some('e') => Some('え'),
        Some('o') => Some('う'), // 'o' prolongs to 'う'
        _ => None,
    }
}

// O(n) implementation returning byte length of common char prefix
pub fn get_stem_length<T: AsRef<str>>(text1: T, text2: T) -> usize {
    let text1 = text1.as_ref();
    let text2 = text2.as_ref();
    text1
        .chars()
        .zip(text2.chars())
        .take_while(|(c1, c2)| c1 == c2)
        .map(|(c1, _)| c1.len_utf8())
        .sum()
}

// Character code testing functions

pub fn is_code_point_kanji(code_point: u32) -> bool {
    is_code_point_in_ranges(code_point, &CJK_IDEOGRAPH_RANGES)
}

pub fn is_code_point_kana(code_point: u32) -> bool {
    is_code_point_in_ranges(code_point, KANA_RANGES)
}

pub fn is_code_point_japanese(code_point: u32) -> bool {
    is_code_point_in_ranges(code_point, &JAPANESE_RANGES)
}

// String testing functions

pub fn is_string_entirely_kana<T: AsRef<str>>(str: T) -> bool {
    let str = str.as_ref();
    !str.is_empty() && str.chars().all(|c| is_code_point_kana(c as u32))
}

pub fn is_string_partially_japanese(str: &str) -> bool {
    !str.is_empty() && str.chars().any(|c| is_code_point_japanese(c as u32))
}

// Mora functions

pub fn is_mora_pitch_high(mora_index: usize, pitch_accent_downstep_position: usize) -> bool {
    match pitch_accent_downstep_position {
        0 => mora_index > 0,
        1 => mora_index < 1,
        _ => mora_index > 0 && mora_index < pitch_accent_downstep_position,
    }
}

pub fn get_pitch_category(
    text: &str,
    pitch_accent_downstep_position: usize,
    is_verb_or_adjective: bool,
) -> Option<PitchCategory> {
    if pitch_accent_downstep_position == 0 {
        return Some(PitchCategory::Heiban);
    }
    if is_verb_or_adjective {
        return if pitch_accent_downstep_position > 0 {
            Some(PitchCategory::Kifuku)
        } else {
            None
        };
    }
    if pitch_accent_downstep_position == 1 {
        return Some(PitchCategory::Atamadaka);
    }
    if pitch_accent_downstep_position > 1 {
        return if pitch_accent_downstep_position >= get_kana_mora_count(text) as usize {
            Some(PitchCategory::Odaka)
        } else {
            Some(PitchCategory::Nakadaka)
        };
    }
    None
}

pub fn get_kana_morae<T: AsRef<str>>(text: T) -> Vec<String> {
    let text = text.as_ref();
    let mut morae: Vec<String> = Vec::new();
    for char in text.chars() {
        if SMALL_KANA_SET.contains(&char) && !morae.is_empty() {
            morae.last_mut().unwrap().push(char);
        } else {
            morae.push(char.to_string());
        }
    }
    morae
}

pub fn get_kana_mora_count<T: AsRef<str>>(text: T) -> u16 {
    let mut mora_count: u16 = 0;
    for c in text.as_ref().chars() {
        if !(SMALL_KANA_SET.contains(&c) && mora_count > 0) {
            mora_count += 1;
        }
    }
    mora_count
}

// Conversion functions

pub fn convert_katakana_to_hiragana<T: AsRef<str>>(
    text: T,
    keep_prolonged_sound_marks: bool,
) -> String {
    let mut result = String::new();
    let text = text.as_ref();
    let offset = HIRAGANA_CONVERSION_RANGE.0 as i32 - KATAKANA_CONVERSION_RANGE.0 as i32;

    for char in text.chars() {
        let code_point = char as u32;
        let mut processed_char = char;

        match code_point {
            KATAKANA_SMALL_KA_CODE_POINT | KATAKANA_SMALL_KE_CODE_POINT => {}
            KANA_PROLONGED_SOUND_MARK_CODE_POINT => {
                if !keep_prolonged_sound_marks && !result.is_empty() {
                    if let Some(last_char) = result.chars().last() {
                        if let Some(prolonged) = get_prolonged_hiragana(last_char) {
                            processed_char = prolonged;
                        }
                    }
                }
            }
            _ => {
                if is_code_point_in_range(code_point, KATAKANA_CONVERSION_RANGE) {
                    if let Some(new_char) = std::char::from_u32((code_point as i32 + offset) as u32)
                    {
                        processed_char = new_char;
                    }
                }
            }
        }
        result.push(processed_char);
    }
    result
}

pub fn convert_hiragana_to_katakana<T: AsRef<str>>(text: T) -> String {
    let mut result = String::new();
    let text = text.as_ref();
    let offset = KATAKANA_CONVERSION_RANGE.0 as i32 - HIRAGANA_CONVERSION_RANGE.0 as i32;

    for char in text.chars() {
        let code_point = char as u32;
        let mut processed_char = char;
        if is_code_point_in_range(code_point, HIRAGANA_CONVERSION_RANGE) {
            if let Some(new_char) = std::char::from_u32((code_point as i32 + offset) as u32) {
                processed_char = new_char;
            }
        }
        result.push(processed_char);
    }
    result
}

pub fn convert_alphanumeric_to_fullwidth<T: AsRef<str>>(text: T) -> String {
    text.as_ref()
        .chars()
        .map(|c| {
            let cp = c as u32;
            match cp {
                0x30..=0x39 => std::char::from_u32(cp + 0xff10 - 0x30).unwrap_or(c),
                0x41..=0x5a => std::char::from_u32(cp + 0xff21 - 0x41).unwrap_or(c),
                0x61..=0x7a => std::char::from_u32(cp + 0xff41 - 0x61).unwrap_or(c),
                _ => c,
            }
        })
        .collect()
}

pub fn convert_fullwidth_alphanumeric_to_normal<T: AsRef<str>>(text: T) -> String {
    text.as_ref()
        .chars()
        .map(|c| {
            let cp = c as u32;
            match cp {
                0xff10..=0xff19 => std::char::from_u32(cp - (0xff10 - 0x30)).unwrap_or(c),
                0xff21..=0xff3a => std::char::from_u32(cp - (0xff21 - 0x41)).unwrap_or(c),
                0xff41..=0xff5a => std::char::from_u32(cp - (0xff41 - 0x61)).unwrap_or(c),
                _ => c,
            }
        })
        .collect()
}

pub fn convert_halfwidth_kana_to_fullwidth(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if let Some(mapping_str) = HALFWIDTH_KATAKANA_MAP.get(&c) {
            let mut index = 0;
            if let Some(&next_char) = chars.peek() {
                match next_char as u32 {
                    0xff9e => index = 1, // Dakuten
                    0xff9f => index = 2, // Handakuten
                    _ => {}
                }
            }

            let mut mapped_char = mapping_str.chars().nth(index).unwrap_or('-');

            if index > 0 {
                if mapped_char == '-' {
                    mapped_char = mapping_str.chars().next().unwrap_or(c);
                } else {
                    chars.next(); // Consume diacritic
                }
            } else {
                mapped_char = mapping_str.chars().next().unwrap_or(c);
            }

            result.push(if mapped_char == '-' { c } else { mapped_char });
        } else {
            result.push(c);
        }
    }
    result
}

pub fn get_kana_diacritic_info(character: char) -> Option<DiacriticInfo> {
    DIACRITIC_MAPPING.get(&character).copied()
}

pub fn dakuten_allowed(code_point: u32) -> bool {
    (0x304B..=0x3068).contains(&code_point)
        || (0x306F..=0x307B).contains(&code_point)
        || (0x30AB..=0x30C8).contains(&code_point)
        || (0x30CF..=0x30DB).contains(&code_point)
}

pub fn handakuten_allowed(code_point: u32) -> bool {
    (0x306F..=0x307B).contains(&code_point) || (0x30CF..=0x30DB).contains(&code_point)
}

pub fn normalize_combining_characters(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(current_char) = chars.next() {
        let current_cp = current_char as u32;
        let mut combined = false;

        if let Some(&next_char) = chars.peek() {
            if next_char == '\u{3099}' && dakuten_allowed(current_cp) {
                if let Some(combined_char) = std::char::from_u32(current_cp + 1) {
                    result.push(combined_char);
                    chars.next();
                    combined = true;
                }
            } else if next_char == '\u{309A}' && handakuten_allowed(current_cp) {
                if let Some(combined_char) = std::char::from_u32(current_cp + 2) {
                    result.push(combined_char);
                    chars.next();
                    combined = true;
                }
            }
        }
        if !combined {
            result.push(current_char);
        }
    }
    result
}

// Fixed to use `nfkd()` and correct import
pub fn normalize_cjk_compatibility_characters(text: &str) -> String {
    text.chars()
        .map(|c| {
            let code_point = c as u32;
            if is_code_point_in_range(code_point, CJK_COMPATIBILITY_IDEOGRAPHS_RANGE) {
                c.nfkd().collect::<String>()
            } else {
                c.to_string()
            }
        })
        .collect()
}

// Furigana distribution

fn get_furigana_kana_segments(text: &str, reading: &str) -> Vec<FuriganaSegment> {
    let mut new_segments: Vec<FuriganaSegment> = Vec::new();
    let mut start_idx = 0;
    let mut reading_start_idx = 0;

    if text.is_empty() {
        return new_segments;
    }

    let mut text_iter = text.char_indices();
    let mut reading_iter = reading.char_indices();

    let mut state = text.chars().next() == reading.chars().next();

    loop {
        match (text_iter.next(), reading_iter.next()) {
            (Some((i_t, tc)), Some((i_r, rc))) => {
                let new_state = tc == rc;
                if state != new_state {
                    new_segments.push(FuriganaSegment::create_furigana_segment(
                        text[start_idx..i_t].to_string(),
                        if state {
                            None
                        } else {
                            Some(reading[reading_start_idx..i_r].to_string())
                        },
                    ));
                    state = new_state;
                    start_idx = i_t;
                    reading_start_idx = i_r;
                }
            }
            _ => break,
        }
    }

    new_segments.push(FuriganaSegment::create_furigana_segment(
        text[start_idx..].to_string(),
        if state {
            None
        } else {
            Some(reading[reading_start_idx..].to_string())
        },
    ));
    new_segments
}

// Fixed to handle char vs byte indices more carefully
fn segmentize_furigana(
    reading: &str,
    reading_normalized: &str,
    groups: &[FuriganaGroup],
    groups_start: usize,
) -> Option<Vec<FuriganaSegment>> {
    let group_count = groups.len().saturating_sub(groups_start);
    if group_count == 0 {
        return if reading.is_empty() {
            Some(vec![])
        } else {
            None
        };
    }

    let group = &groups[groups_start];
    let text = &group.text;

    if group.is_kana {
        if let Some(text_normalized_val) = &group.text_normalized {
            if reading_normalized.starts_with(text_normalized_val) {
                let norm_char_count = text_normalized_val.chars().count();
                let reading_byte_idx = reading
                    .char_indices()
                    .nth(norm_char_count)
                    .map_or(reading.len(), |(i, _)| i);
                let norm_byte_idx = reading_normalized
                    .char_indices()
                    .nth(norm_char_count)
                    .map_or(reading_normalized.len(), |(i, _)| i);

                if let Some(mut segments) = segmentize_furigana(
                    &reading[reading_byte_idx..],
                    &reading_normalized[norm_byte_idx..],
                    groups,
                    groups_start + 1,
                ) {
                    let reading_prefix = &reading[..reading_byte_idx];
                    if reading_prefix == text {
                        segments.insert(
                            0,
                            FuriganaSegment::create_furigana_segment(text.clone(), None),
                        );
                    } else {
                        let mut kana_segments = get_furigana_kana_segments(text, reading_prefix);
                        kana_segments.extend(segments);
                        segments = kana_segments;
                    }
                    return Some(segments);
                }
            }
        }
        None
    } else {
        let mut result = None;
        let reading_char_count = reading.chars().count();
        let text_char_count = text.chars().count();

        for char_len in (text_char_count..=reading_char_count).rev() {
            let reading_byte_idx = reading
                .char_indices()
                .nth(char_len)
                .map_or(reading.len(), |(i, _)| i);
            let norm_byte_idx = reading_normalized
                .char_indices()
                .nth(char_len)
                .map_or(reading_normalized.len(), |(i, _)| i);

            if let Some(mut segments) = segmentize_furigana(
                &reading[reading_byte_idx..],
                &reading_normalized[norm_byte_idx..],
                groups,
                groups_start + 1,
            ) {
                if result.is_some() {
                    return None;
                }

                segments.insert(
                    0,
                    FuriganaSegment::create_furigana_segment(
                        text.clone(),
                        Some(reading[..reading_byte_idx].to_string()),
                    ),
                );
                result = Some(segments);

                if group_count == 1 {
                    break;
                }
            }
        }
        result
    }
}

pub fn distribute_furigana(term: String, reading: String) -> Vec<FuriganaSegment> {
    if reading == term {
        return vec![FuriganaSegment::create_furigana_segment(term, None)];
    }

    let mut groups: Vec<FuriganaGroup> = vec![];
    if !term.is_empty() {
        let mut term_chars = term.chars();
        let first_char = term_chars.next().unwrap();
        let mut current_is_kana = is_code_point_kana(first_char as u32);
        let mut current_text = String::from(first_char);

        for c in term_chars {
            let is_kana = is_code_point_kana(c as u32);
            if is_kana == current_is_kana {
                current_text.push(c);
            } else {
                groups.push(FuriganaGroup {
                    is_kana: current_is_kana,
                    text: current_text,
                    text_normalized: None,
                });
                current_text = String::from(c);
                current_is_kana = is_kana;
            }
        }
        groups.push(FuriganaGroup {
            is_kana: current_is_kana,
            text: current_text,
            text_normalized: None,
        });
    }

    for group in &mut groups {
        if group.is_kana {
            group.text_normalized = Some(convert_katakana_to_hiragana(&group.text, false));
        }
    }

    let reading_normalized = convert_katakana_to_hiragana(&reading, false);
    if let Some(segments) = segmentize_furigana(&reading, &reading_normalized, &groups, 0) {
        return segments;
    }

    vec![FuriganaSegment::create_furigana_segment(
        term,
        Some(reading),
    )]
}

// Fixed to use byte length stem
pub fn distribute_furigana_inflected(
    term: String,
    mut reading: String,
    source: String,
) -> Vec<FuriganaSegment> {
    let term_normalized = convert_katakana_to_hiragana(&term, false);
    let reading_normalized = convert_katakana_to_hiragana(&reading, false);
    let source_normalized = convert_katakana_to_hiragana(&source, false);

    let mut main_text = term.clone();
    let mut stem_byte_length = get_stem_length(&term_normalized, &source_normalized);

    let reading_stem_byte_length = get_stem_length(&reading_normalized, &source_normalized);
    if reading_stem_byte_length > 0 && reading_stem_byte_length >= stem_byte_length {
        main_text = reading.clone();
        stem_byte_length = reading_stem_byte_length;
        reading = format!(
            "{}{}",
            &source[..stem_byte_length],
            &reading[stem_byte_length..]
        );
    }

    let source_byte_len = source.len();

    let mut segments: Vec<FuriganaSegment> = vec![];
    if stem_byte_length > 0 {
        main_text = format!(
            "{}{}",
            &source[..stem_byte_length],
            &main_text[stem_byte_length..]
        );

        let segments2 = distribute_furigana(main_text.clone(), reading);
        let mut consumed_bytes = 0;
        for segment in segments2 {
            let text_len = segment.text.len();
            let start = consumed_bytes;
            consumed_bytes += text_len;
            if consumed_bytes < stem_byte_length {
                segments.push(segment);
            } else if consumed_bytes == stem_byte_length {
                segments.push(segment);
                break;
            } else {
                if start < stem_byte_length {
                    segments.push(FuriganaSegment::create_furigana_segment(
                        main_text[start..stem_byte_length].to_string(),
                        None,
                    ));
                }
                break;
            }
        }
    }

    if stem_byte_length < source_byte_len {
        let remainder = &source[stem_byte_length..];
        if let Some(last_segment) = segments.last_mut() {
            if last_segment.reading.is_none() {
                last_segment.text.push_str(remainder);
                return segments;
            }
        }
        segments.push(FuriganaSegment::create_furigana_segment(
            remainder.to_string(),
            None,
        ));
    }
    segments
}

// Miscellaneous

pub fn is_emphatic_code_point(code_point: u32) -> bool {
    matches!(
        code_point,
        HIRAGANA_SMALL_TSU_CODE_POINT
            | KATAKANA_SMALL_TSU_CODE_POINT
            | KANA_PROLONGED_SOUND_MARK_CODE_POINT
    )
}

pub fn collapse_emphatic_sequences(text: &str, full_collapse: bool) -> String {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    if len == 0 {
        return String::new();
    }

    let mut left = 0;
    while left < len && is_emphatic_code_point(chars[left] as u32) {
        left += 1;
    }

    let mut right = len;
    while right > left && is_emphatic_code_point(chars[right - 1] as u32) {
        right -= 1;
    }

    if left >= right {
        return text.to_string();
    }

    let leading_emphatics: String = chars[0..left].iter().collect();
    let trailing_emphatics: String = chars[right..len].iter().collect();
    let mut middle = String::new();
    let mut current_collapsed_code_point: Option<u32> = None;

    for &char in &chars[left..right] {
        let code_point = char as u32;
        if is_emphatic_code_point(code_point) {
            if current_collapsed_code_point != Some(code_point) {
                current_collapsed_code_point = Some(code_point);
                middle.push(char);
            } else if !full_collapse {
                middle.push(char);
            }
        } else {
            current_collapsed_code_point = None;
            middle.push(char);
        }
    }

    // Second pass for full collapse
    if full_collapse {
        let original_middle = middle;
        middle = String::new();
        current_collapsed_code_point = None;
        for char in original_middle.chars() {
            let code_point = char as u32;
            if is_emphatic_code_point(code_point) {
                if current_collapsed_code_point != Some(code_point) {
                    current_collapsed_code_point = Some(code_point);
                    middle.push(char);
                }
            } else {
                current_collapsed_code_point = None;
                middle.push(char);
            }
        }
    }

    format!("{leading_emphatics}{middle}{trailing_emphatics}")
}

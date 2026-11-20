use std::collections::{HashSet, VecDeque};
use std::io::{self, BufRead, Write};
use std::process;

use deinflector::de::de_transforms::SEPARABLE_PREFIXES;
use deinflector::multi_language_transformer::MultiLanguageTransformer;
use deinflector::transformer::TransformedText;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum ServeRequest {
    Ping,
    Deinflect {
        #[serde(default = "default_lang")]
        lang: String,
        word: String,
    },
    Match {
        #[serde(default = "default_lang")]
        lang: String,
        target: String,
        tokens: Vec<String>,
        #[serde(default)]
        separators: Option<Vec<String>>,
    },
}

fn default_lang() -> String {
    "es".to_string()
}

#[derive(Serialize)]
struct DeinflectEntry {
    source: String,
    candidates: Vec<TransformedText>,
}

#[derive(Serialize)]
struct MatchResponse {
    ok: bool,
    matched_indices: Vec<usize>,
    matched_tokens: Vec<String>,
}

#[derive(Serialize)]
struct ServeDeinflectResponse {
    ok: bool,
    word: String,
    candidates: Vec<TransformedText>,
}

#[derive(Serialize)]
struct ServeOkResponse {
    ok: bool,
}

#[derive(Serialize)]
struct ServeErrorResponse {
    ok: bool,
    error: String,
}

fn print_usage() {
    eprintln!(
        "Usage:
  deinflector [--lang <iso>] [--json] <word>...
  deinflector deinflect [--lang <iso>] [--json] <word>...
  deinflector match [--lang <iso>] --target <dictionary_form> [--json] <token>...
  deinflector serve

Options:
  -l, --lang <iso>       Language ISO code (default: es)
  -t, --target <word>    Dictionary form to match against (for `match`)
      --json             Output JSON
  -h, --help             Show this help message"
    );
}

/// Returns German `ß` <-> `ss` orthographic variants for a lowercased word.
fn eszett_variants(lower: &str) -> Vec<String> {
    let mut variants = vec![lower.to_string()];
    if lower.contains('ß') {
        variants.push(lower.replace('ß', "ss"));
    }
    if lower.contains("ss") {
        variants.push(lower.replace("ss", "ß"));
    }
    variants
}

/// Expands a dictionary form into acceptable target lemmas.
///
/// For Spanish reflexive verbs (`-arse`, `-erse`, `-irse`), also allows matching
/// the base non-reflexive infinitive (e.g. `atorarse` -> `atorar`) so inflected
/// participles/conjugations in sentences still match reflexive headwords.
/// For German (`de`), includes `ß` <-> `ss` variants.
fn expand_target_forms(lang: &str, target: &str) -> HashSet<String> {
    let lower = target.trim().to_lowercase();
    let mut forms = HashSet::new();
    if lower.is_empty() {
        return forms;
    }
    forms.insert(lower.clone());

    if lang == "es"
        && lower.len() > 4
        && (lower.ends_with("arse") || lower.ends_with("erse") || lower.ends_with("irse"))
    {
        forms.insert(lower[..lower.len() - 2].to_string());
    } else if lang == "de" {
        for variant in eszett_variants(&lower) {
            forms.insert(variant);
        }
        // Informal `du` imperative drops `-en` (e.g. `machen` -> `mach`, `kommen` -> `komm`).
        if let Some(stem) = german_bare_imperative_stem(&lower) {
            for variant in eszett_variants(stem) {
                forms.insert(variant);
            }
        }
        // Determiner/pronoun dictionary headwords ending in `-er` (e.g. `keiner` -> `kein`).
        if let Some(base) = match lower.as_str() {
            "keiner" => Some("kein"),
            "einer" => Some("ein"),
            "meiner" => Some("mein"),
            "deiner" => Some("dein"),
            "seiner" => Some("sein"),
            "ihrer" => Some("ihr"),
            "unserer" => Some("unser"),
            "eurer" => Some("euer"),
            "dieser" => Some("dies"),
            "jeder" => Some("jed"),
            "welcher" => Some("welch"),
            "mancher" => Some("manch"),
            _ => None,
        } {
            forms.insert(base.to_string());
        }
    }
    forms
}

/// Checks whether `token` matches any form in `target_forms` directly or via deinflection.
fn token_matches_target_forms(
    mlt: &MultiLanguageTransformer,
    lang: &str,
    token: &str,
    target_forms: &HashSet<String>,
) -> bool {
    let lower = token.trim().to_lowercase();
    if lower.is_empty() {
        return false;
    }
    let token_variants = if lang == "de" {
        eszett_variants(&lower)
    } else {
        vec![lower]
    };

    for variant in &token_variants {
        if target_forms.contains(variant) {
            return true;
        }
    }

    token_variants.into_iter().any(|variant| {
        mlt.transform(lang, &variant)
            .into_iter()
            .any(|candidate| target_forms.contains(&candidate.text))
    })
}

/// Splits a German separable verb (e.g. `aufhören`) into `(prefix, base_verb)` using the longest
/// matching prefix in `SEPARABLE_PREFIXES`.
fn split_german_separable_verb(lower_target: &str) -> Option<(&'static str, &str)> {
    let mut best: Option<&'static str> = None;
    for &prefix in SEPARABLE_PREFIXES {
        if lower_target.starts_with(prefix) && lower_target.len() >= prefix.len() + 3 {
            if best.map_or(true, |b| prefix.len() > b.len()) {
                best = Some(prefix);
            }
        }
    }
    best.map(|p| (p, &lower_target[p.len()..]))
}

fn german_bare_imperative_stem(base_verb: &str) -> Option<&str> {
    if base_verb.ends_with("en") && base_verb.len() > 4 {
        Some(&base_verb[..base_verb.len() - 2])
    } else if base_verb.ends_with('n') && base_verb.len() > 3 {
        Some(&base_verb[..base_verb.len() - 1])
    } else {
        None
    }
}

fn is_german_clause_boundary(
    idx: usize,
    tokens: &[String],
    separators: Option<&[String]>,
) -> bool {
    if idx + 1 >= tokens.len() {
        return true;
    }
    if let Some(seps) = separators {
        if let Some(sep) = seps.get(idx) {
            if sep.chars().any(|c| {
                matches!(
                    c,
                    ',' | '.' | '!' | '?' | ';' | ':' | '—' | '–' | '(' | ')' | '[' | ']' | '\n' | '\r'
                )
            }) {
                return true;
            }
        }
    }
    let next_lower = tokens[idx + 1].to_lowercase();
    matches!(
        next_lower.as_str(),
        "und" | "oder" | "aber" | "denn" | "sondern" | "weil" | "dass" | "obwohl" | "wenn" | "als" | "zu"
    )
}

/// Matches German split separable verbs (`aufhören` -> `Hör ... auf`, `Er hört ... auf.`).
fn find_german_separable_matches(
    mlt: &MultiLanguageTransformer,
    target_word: &str,
    tokens: &[String],
    separators: Option<&[String]>,
) -> Vec<usize> {
    let lower_target = target_word.trim().to_lowercase();
    let Some((prefix, base_verb)) = split_german_separable_verb(&lower_target) else {
        return Vec::new();
    };
    let base_forms = expand_target_forms("de", base_verb);
    let bare_stem = german_bare_imperative_stem(base_verb);

    let mut matched = Vec::new();
    let mut clause_start = 0;

    for i in 0..tokens.len() {
        if !is_german_clause_boundary(i, tokens, separators) {
            continue;
        }
        let clause_end = i + 1;
        if clause_end - clause_start >= 2 {
            let prefix_idx = clause_end - 1;
            let prefix_tok_lower = tokens[prefix_idx].trim().to_lowercase();
            if prefix_tok_lower == prefix {
                let mut found_verb = false;
                for v_idx in clause_start..prefix_idx {
                    let v_lower = tokens[v_idx].trim().to_lowercase();
                    let is_bare_imp = bare_stem.is_some_and(|stem| v_lower == stem);
                    if is_bare_imp
                        || token_matches_target_forms(mlt, "de", &tokens[v_idx], &base_forms)
                    {
                        matched.push(v_idx);
                        found_verb = true;
                    }
                }
                if found_verb {
                    matched.push(prefix_idx);
                }
            }

            // Also match immediately adjacent `verb + prefix` (e.g. "Hör auf damit, Mitsu.")
            for v_idx in clause_start..prefix_idx {
                let next_lower = tokens[v_idx + 1].trim().to_lowercase();
                if next_lower != prefix {
                    continue;
                }
                let v_lower = tokens[v_idx].trim().to_lowercase();
                let is_bare_imp = bare_stem.is_some_and(|stem| v_lower == stem);
                if is_bare_imp
                    || token_matches_target_forms(mlt, "de", &tokens[v_idx], &base_forms)
                {
                    matched.push(v_idx);
                    matched.push(v_idx + 1);
                }
            }
        }
        clause_start = clause_end;
    }

    matched
}

/// Returns indices of `tokens` that deinflect to `target` (supports single- and multi-word targets).
fn find_matched_token_indices(
    mlt: &MultiLanguageTransformer,
    lang: &str,
    target: &str,
    tokens: &[String],
) -> Vec<usize> {
    find_matched_token_indices_with_separators(mlt, lang, target, tokens, None)
}

fn find_matched_token_indices_with_separators(
    mlt: &MultiLanguageTransformer,
    lang: &str,
    target: &str,
    tokens: &[String],
    separators: Option<&[String]>,
) -> Vec<usize> {
    let target_words: Vec<&str> = target.split_whitespace().collect();
    if target_words.is_empty() || tokens.is_empty() {
        return Vec::new();
    }

    if target_words.len() == 1 {
        let target_forms = expand_target_forms(lang, target_words[0]);
        let mut matched: HashSet<usize> = tokens
            .iter()
            .enumerate()
            .filter_map(|(idx, tok)| {
                token_matches_target_forms(mlt, lang, tok, &target_forms).then_some(idx)
            })
            .collect();

        if lang == "de" {
            for idx in find_german_separable_matches(mlt, target_words[0], tokens, separators) {
                matched.insert(idx);
            }
        }

        let mut indices: Vec<usize> = matched.into_iter().collect();
        indices.sort_unstable();
        return indices;
    }

    let expanded_words: Vec<HashSet<String>> = target_words
        .iter()
        .map(|w| expand_target_forms(lang, w))
        .collect();
    let window_len = expanded_words.len();
    if tokens.len() < window_len {
        return Vec::new();
    }

    let mut matched = HashSet::new();
    for start in 0..=tokens.len() - window_len {
        let all_match = (0..window_len).all(|offset| {
            token_matches_target_forms(mlt, lang, &tokens[start + offset], &expanded_words[offset])
        });
        if all_match {
            for offset in 0..window_len {
                matched.insert(start + offset);
            }
        }
    }

    let mut indices: Vec<usize> = matched.into_iter().collect();
    indices.sort_unstable();
    indices
}

fn run_deinflect(mlt: &MultiLanguageTransformer, lang: &str, words: &[String], json_out: bool) {
    if json_out {
        let entries: Vec<DeinflectEntry> = words
            .iter()
            .map(|w| DeinflectEntry {
                source: w.clone(),
                candidates: mlt.transform(lang, &w.to_lowercase()),
            })
            .collect();
        println!("{}", serde_json::to_string(&entries).unwrap());
        return;
    }

    for word in words {
        let results = mlt.transform(lang, &word.to_lowercase());
        if words.len() > 1 {
            println!("{word}:");
        }
        let mut seen = HashSet::new();
        for res in results {
            if seen.insert(res.text.clone()) {
                let chain: Vec<&str> = res.trace.iter().map(|f| f.transform.as_str()).collect();
                if chain.is_empty() {
                    println!("  {}", res.text);
                } else {
                    println!("  {} ({})", res.text, chain.join(" -> "));
                }
            }
        }
    }
}

fn run_match(
    mlt: &MultiLanguageTransformer,
    lang: &str,
    target: &str,
    tokens: &[String],
    json_out: bool,
) {
    let matched_indices = find_matched_token_indices(mlt, lang, target, tokens);
    if json_out {
        let matched_tokens: Vec<String> = matched_indices
            .iter()
            .map(|&i| tokens[i].clone())
            .collect();
        let resp = MatchResponse {
            ok: true,
            matched_indices,
            matched_tokens,
        };
        println!("{}", serde_json::to_string(&resp).unwrap());
    } else {
        for idx in matched_indices {
            println!("{idx}\t{}", tokens[idx]);
        }
    }
}

fn run_serve(mlt: &MultiLanguageTransformer) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line_res in stdin.lock().lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(err) => {
                let err_resp = ServeErrorResponse {
                    ok: false,
                    error: format!("stdin read error: {err}"),
                };
                let _ = writeln!(out, "{}", serde_json::to_string(&err_resp).unwrap());
                let _ = out.flush();
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response_json = match serde_json::from_str::<ServeRequest>(trimmed) {
            Ok(ServeRequest::Ping) => serde_json::to_string(&ServeOkResponse { ok: true }).unwrap(),
            Ok(ServeRequest::Deinflect { lang, word }) => {
                let candidates = mlt.transform(&lang, &word.to_lowercase());
                serde_json::to_string(&ServeDeinflectResponse {
                    ok: true,
                    word,
                    candidates,
                })
                .unwrap()
            }
            Ok(ServeRequest::Match {
                lang,
                target,
                tokens,
                separators,
            }) => {
                let matched_indices = find_matched_token_indices_with_separators(
                    mlt,
                    &lang,
                    &target,
                    &tokens,
                    separators.as_deref(),
                );
                let matched_tokens: Vec<String> = matched_indices
                    .iter()
                    .map(|&i| tokens[i].clone())
                    .collect();
                serde_json::to_string(&MatchResponse {
                    ok: true,
                    matched_indices,
                    matched_tokens,
                })
                .unwrap()
            }
            Err(err) => serde_json::to_string(&ServeErrorResponse {
                ok: false,
                error: format!("invalid request: {err}"),
            })
            .unwrap(),
        };

        if writeln!(out, "{response_json}").is_err() || out.flush().is_err() {
            break;
        }
    }
}

fn take_value(args: &mut VecDeque<String>, flag: &str) -> String {
    match args.pop_front() {
        Some(v) => v,
        None => {
            eprintln!("Missing value for {flag}");
            process::exit(2);
        }
    }
}

fn main() {
    let mut args: VecDeque<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        process::exit(2);
    }

    let subcommand = match args.front().map(String::as_str) {
        Some("deinflect") | Some("match") | Some("serve") => args.pop_front().unwrap(),
        _ => "deinflect".to_string(),
    };

    let mut lang = default_lang();
    let mut target: Option<String> = None;
    let mut json_out = false;
    let mut positionals: Vec<String> = Vec::new();

    while let Some(arg) = args.pop_front() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                return;
            }
            "-l" | "--lang" => {
                lang = take_value(&mut args, &arg);
            }
            "-t" | "--target" => {
                target = Some(take_value(&mut args, &arg));
            }
            "--json" => {
                json_out = true;
            }
            "--" => {
                positionals.extend(args.drain(..));
            }
            other if other.starts_with('-') => {
                eprintln!("Unknown option: {other}");
                print_usage();
                process::exit(2);
            }
            _ => positionals.push(arg),
        }
    }

    let mlt = MultiLanguageTransformer::default();

    match subcommand.as_str() {
        "serve" => run_serve(&mlt),
        "match" => {
            let Some(target_word) = target else {
                eprintln!("`match` requires --target <dictionary_form>");
                process::exit(2);
            };
            run_match(&mlt, &lang, &target_word, &positionals, json_out);
        }
        _ => {
            if positionals.is_empty() {
                print_usage();
                process::exit(2);
            }
            run_deinflect(&mlt, &lang, &positionals, json_out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn spanish_atorar_sentence_matches_atorada() {
        let mlt = MultiLanguageTransformer::default();
        let tokens = to_strings(&[
            "Pero",
            "es",
            "igual",
            "de",
            "desesperante",
            "que",
            "tener",
            "un",
            "pedazo",
            "de",
            "comida",
            "atorada",
            "en",
            "los",
            "dientes",
            "de",
            "atrás",
        ]);
        let matched = find_matched_token_indices(&mlt, "es", "atorar", &tokens);
        assert_eq!(matched, vec![11]);
    }

    #[test]
    fn spanish_capitalized_and_reflexive_and_nouns() {
        let mlt = MultiLanguageTransformer::default();
        assert_eq!(
            find_matched_token_indices(&mlt, "es", "atorar", &to_strings(&["Atorada"])),
            vec![0]
        );
        assert_eq!(
            find_matched_token_indices(&mlt, "es", "atorarse", &to_strings(&["atorada"])),
            vec![0]
        );
        assert_eq!(
            find_matched_token_indices(&mlt, "es", "cerilla", &to_strings(&["cerilla", "cerillas"])),
            vec![0, 1]
        );
        assert_eq!(
            find_matched_token_indices(
                &mlt,
                "es",
                "despertarse",
                &to_strings(&["Se", "despertó", "temprano"])
            ),
            vec![1]
        );
        assert_eq!(
            find_matched_token_indices(
                &mlt,
                "es",
                "drenar",
                &to_strings(&["No", "debió", "drenarlo", "todo"])
            ),
            vec![2]
        );
    }

    #[test]
    fn spanish_multi_word_target() {
        let mlt = MultiLanguageTransformer::default();
        let tokens = to_strings(&["No", "quiero", "darse", "cuenta", "tarde"]);
        assert_eq!(
            find_matched_token_indices(&mlt, "es", "darse cuenta", &tokens),
            vec![2, 3]
        );
    }

    #[test]
    fn german_transforms_and_eszett() {
        let mlt = MultiLanguageTransformer::default();
        assert_eq!(
            find_matched_token_indices(
                &mlt,
                "de",
                "aufmachen",
                &to_strings(&["Ich", "habe", "die", "Tür", "aufgemacht"])
            ),
            vec![4]
        );
        assert_eq!(
            find_matched_token_indices(
                &mlt,
                "de",
                "aufmachen",
                &to_strings(&["Es", "ist", "schwer", "aufzumachen"])
            ),
            vec![3]
        );
        assert_eq!(
            find_matched_token_indices(
                &mlt,
                "de",
                "lesen",
                &to_strings(&["Das", "ist", "unlesbar"])
            ),
            vec![2]
        );
        assert_eq!(
            find_matched_token_indices(
                &mlt,
                "de",
                "Straße",
                &to_strings(&["Auf", "der", "Strasse"])
            ),
            vec![2]
        );
    }

    #[test]
    fn german_split_separable_verbs() {
        let mlt = MultiLanguageTransformer::default();

        // "Hör sofort auf!" -> aufhören
        assert_eq!(
            find_matched_token_indices_with_separators(
                &mlt,
                "de",
                "aufhören",
                &to_strings(&["Hör", "sofort", "auf"]),
                Some(&to_strings(&[" ", " ", "!"])),
            ),
            vec![0, 2]
        );

        // "Er hört sofort auf." -> aufhören
        assert_eq!(
            find_matched_token_indices_with_separators(
                &mlt,
                "de",
                "aufhören",
                &to_strings(&["Er", "hört", "sofort", "auf"]),
                Some(&to_strings(&[" ", " ", " ", "."])),
            ),
            vec![1, 3]
        );

        // "Sie hörte gestern damit auf, zu rauchen." -> aufhören
        assert_eq!(
            find_matched_token_indices_with_separators(
                &mlt,
                "de",
                "aufhören",
                &to_strings(&["Sie", "hörte", "gestern", "damit", "auf", "zu", "rauchen"]),
                Some(&to_strings(&[" ", " ", " ", " ", ", ", " ", "."])),
            ),
            vec![1, 4]
        );

        // "Hör auf zu reden!" -> aufhören
        assert_eq!(
            find_matched_token_indices_with_separators(
                &mlt,
                "de",
                "aufhören",
                &to_strings(&["Hör", "auf", "zu", "reden"]),
                Some(&to_strings(&[" ", " ", " ", "!"])),
            ),
            vec![0, 1]
        );

        // "Hör auf damit, Mitsu." -> aufhören
        assert_eq!(
            find_matched_token_indices_with_separators(
                &mlt,
                "de",
                "aufhören",
                &to_strings(&["Hör", "auf", "damit", "Mitsu"]),
                Some(&to_strings(&[" ", " ", ", ", "."])),
            ),
            vec![0, 1]
        );

        // "Mach es bald wieder." -> machen (bare `du` imperative)
        assert_eq!(
            find_matched_token_indices_with_separators(
                &mlt,
                "de",
                "machen",
                &to_strings(&["Mach", "es", "bald", "wieder"]),
                Some(&to_strings(&[" ", " ", " ", "."])),
            ),
            vec![0]
        );

        // "du bist kein Mensch." -> keiner (pronoun headword -> determiner base)
        assert_eq!(
            find_matched_token_indices_with_separators(
                &mlt,
                "de",
                "keiner",
                &to_strings(&["du", "bist", "kein", "Mensch"]),
                Some(&to_strings(&[" ", " ", " ", "."])),
            ),
            vec![2]
        );

        // "Er hört Musik auf dem Bett." -> should NOT match preposition `auf`
        assert_eq!(
            find_matched_token_indices_with_separators(
                &mlt,
                "de",
                "aufhören",
                &to_strings(&["Er", "hört", "Musik", "auf", "dem", "Bett"]),
                Some(&to_strings(&[" ", " ", " ", " ", " ", "."])),
            ),
            Vec::<usize>::new()
        );
    }
}

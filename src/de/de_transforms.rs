use std::sync::LazyLock;

use fancy_regex::Regex;
use indexmap::IndexMap;

use crate::{
    transformer::{
        Condition, ConditionMap, DeinflectFnType, LanguageTransformDescriptor, Rule, RuleType,
        Transform, TransformMap,
    },
    transforms::{generic_stem_change_rule, inflection},
};

pub const SEPARABLE_PREFIXES: &[&str] = &[
    "ab", "an", "auf", "aus", "auseinander", "bei", "da", "dabei", "dar", "daran", "dazwischen",
    "durch", "ein", "empor", "entgegen", "entlang", "entzwei", "fehl", "fern", "fest", "fort",
    "frei", "gegenüber", "gleich", "heim", "her", "herab", "heran", "herauf", "heraus",
    "herbei", "herein", "herüber", "herum", "herunter", "hervor", "hin", "hinab", "hinauf",
    "hinaus", "hinein", "hinterher", "hinunter", "hinweg", "hinzu", "hoch", "los", "mit", "nach",
    "nebenher", "nieder", "statt", "um", "vor", "voran", "voraus", "vorbei", "vorüber",
    "vorweg", "weg", "weiter", "wieder", "zu", "zurecht", "zurück", "zusammen",
];

const GERMAN_LETTERS: &str = "a-zA-ZäöüßÄÖÜẞ";

pub static GERMAN_TRANSFORMS_DESCRIPTOR: LazyLock<LanguageTransformDescriptor> =
    LazyLock::new(|| LanguageTransformDescriptor {
        language: "de",
        conditions: &DE_CONDITIONS_MAP,
        transforms: &DE_TRANSFORMS_MAP,
    });

pub static DE_CONDITIONS_MAP: LazyLock<ConditionMap> = LazyLock::new(|| {
    ConditionMap(IndexMap::from([
        (
            "v",
            Condition {
                name: "Verb",
                is_dictionary_form: true,
                sub_conditions: Some(&["vw", "vst"]),
                i18n: None,
            },
        ),
        (
            "vw",
            Condition {
                name: "Weak verb",
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "vst",
            Condition {
                name: "Strong verb",
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "n",
            Condition {
                name: "Noun",
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "adj",
            Condition {
                name: "Adjective",
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
    ]))
});

fn separated_prefix_rule(
    prefix: &'static str,
    conditions_in: &'static [&'static str],
    conditions_out: &'static [&'static str],
) -> Rule {
    let pattern = format!("^([{GERMAN_LETTERS}]+) .+ {prefix}$");
    Rule {
        rule_type: RuleType::Other,
        is_inflected: Regex::new(&pattern).unwrap(),
        deinflected: Some(prefix),
        deinflect_fn: DeinflectFnType::DeSeparatedPrefix,
        inflected_str: None,
        conditions_in,
        conditions_out,
    }
}

fn get_basic_past_participles() -> Vec<Rule> {
    let weak_re = Regex::new(&format!("^ge([{GERMAN_LETTERS}]+)t$")).unwrap();
    let strong_re = Regex::new(&format!("^ge([{GERMAN_LETTERS}]+)en$")).unwrap();

    ["n", "en"]
        .into_iter()
        .map(|suffix| Rule {
            rule_type: RuleType::Other,
            is_inflected: weak_re.clone(),
            deinflected: Some(suffix),
            deinflect_fn: DeinflectFnType::DeBasicPastParticiple,
            inflected_str: None,
            conditions_in: &["adj"],
            conditions_out: &["vw"],
        })
        .chain(std::iter::once(Rule {
            rule_type: RuleType::Other,
            is_inflected: strong_re,
            deinflected: Some("en"),
            deinflect_fn: DeinflectFnType::DeBasicPastParticiple,
            inflected_str: None,
            conditions_in: &["adj"],
            conditions_out: &["vst"],
        }))
        .collect()
}

fn get_separable_past_participles() -> Vec<Rule> {
    let prefix_disjunction = SEPARABLE_PREFIXES.join("|");
    let weak_re =
        Regex::new(&format!("^({prefix_disjunction})ge([{GERMAN_LETTERS}]+)t$")).unwrap();
    let strong_re =
        Regex::new(&format!("^({prefix_disjunction})ge([{GERMAN_LETTERS}]+)en$")).unwrap();

    ["n", "en"]
        .into_iter()
        .map(|suffix| Rule {
            rule_type: RuleType::Other,
            is_inflected: weak_re.clone(),
            deinflected: Some(suffix),
            deinflect_fn: DeinflectFnType::DeSeparablePastParticiple,
            inflected_str: None,
            conditions_in: &["adj"],
            conditions_out: &["vw"],
        })
        .chain(std::iter::once(Rule {
            rule_type: RuleType::Other,
            is_inflected: strong_re,
            deinflected: Some("en"),
            deinflect_fn: DeinflectFnType::DeSeparablePastParticiple,
            inflected_str: None,
            conditions_in: &["adj"],
            conditions_out: &["vst"],
        }))
        .collect()
}

/// Expands a strong/irregular preterite stem (e.g. `("kam", "kommen")`) into all person endings.
fn push_preterite_stem_rules(rules: &mut Vec<Rule>, stem: &str, infinitive: &'static str) {
    let (second_sg, second_pl) = if stem.ends_with('d') || stem.ends_with('t') {
        ("est", "et")
    } else if stem.ends_with('s') || stem.ends_with('ß') || stem.ends_with('z') {
        ("est", "t")
    } else {
        ("st", "t")
    };
    rules.push(inflection(stem, infinitive, &["v"], &["vst"], RuleType::Suffix));
    rules.push(inflection(
        &format!("{stem}{second_sg}"),
        infinitive,
        &["v"],
        &["vst"],
        RuleType::Suffix,
    ));
    rules.push(inflection(
        &format!("{stem}en"),
        infinitive,
        &["v"],
        &["vst"],
        RuleType::Suffix,
    ));
    rules.push(inflection(
        &format!("{stem}{second_pl}"),
        infinitive,
        &["v"],
        &["vst"],
        RuleType::Suffix,
    ));
}

static DE_TRANSFORMS_MAP: LazyLock<TransformMap> = LazyLock::new(|| {
    TransformMap(IndexMap::from([
        (
            "nominalization",
            Transform {
                name: "nominalization",
                description: Some("Noun formed from a verb"),
                rules: vec![
                    inflection("ung", "en", &["n"], &["v"], RuleType::Suffix),
                    inflection("lung", "eln", &["n"], &["v"], RuleType::Suffix),
                    inflection("rung", "rn", &["n"], &["v"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
        (
            "-bar",
            Transform {
                name: "-bar",
                description: Some("-able adjective from a verb"),
                rules: vec![
                    inflection("bar", "en", &["adj"], &["v"], RuleType::Suffix),
                    inflection("bar", "n", &["adj"], &["v"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
        (
            "negative",
            Transform {
                name: "negative",
                description: Some("Negation"),
                rules: vec![inflection("un", "", &["adj"], &["adj"], RuleType::Prefix)],
                i18n: None,
            },
        ),
        (
            "present indicative",
            Transform {
                name: "present indicative",
                description: Some("Present tense verb conjugation"),
                rules: {
                    let mut rules = vec![
                        // Strong verb stem vowel changes (2nd/3rd person singular)
                        generic_stem_change_rule("äu", "au", "(st|t)", "en", &["v"], &["vst"]),
                        generic_stem_change_rule(
                            "ä",
                            "a",
                            "(st|est|t|et)",
                            "en",
                            &["v"],
                            &["vst"],
                        ),
                        generic_stem_change_rule("ö", "o", "(st|t)", "en", &["v"], &["vst"]),
                        generic_stem_change_rule("ie", "e", "(st|t)", "en", &["v"], &["vst"]),
                        generic_stem_change_rule("i", "e", "(st|t)", "en", &["v"], &["vst"]),
                        // Regular -en verbs (and Konjunktiv I)
                        inflection("e", "en", &["v"], &["v"], RuleType::Suffix),
                        inflection("st", "en", &["v"], &["v"], RuleType::Suffix),
                        inflection("est", "en", &["v"], &["v"], RuleType::Suffix),
                        inflection("t", "en", &["v"], &["v"], RuleType::Suffix),
                        inflection("et", "en", &["v"], &["v"], RuleType::Suffix),
                        // -eln / -ern verbs
                        inflection("le", "eln", &["v"], &["v"], RuleType::Suffix),
                        inflection("re", "ern", &["v"], &["v"], RuleType::Suffix),
                        inflection("st", "n", &["v"], &["v"], RuleType::Suffix),
                        inflection("t", "n", &["v"], &["v"], RuleType::Suffix),
                    ];

                    // High-frequency irregular & modal present forms
                    for (form, lemma) in [
                        ("bin", "sein"),
                        ("bist", "sein"),
                        ("ist", "sein"),
                        ("sind", "sein"),
                        ("seid", "sein"),
                        ("sei", "sein"),
                        ("seien", "sein"),
                        ("habe", "haben"),
                        ("hast", "haben"),
                        ("hat", "haben"),
                        ("habt", "haben"),
                        ("werde", "werden"),
                        ("wirst", "werden"),
                        ("wird", "werden"),
                        ("werdet", "werden"),
                        ("kann", "können"),
                        ("kannst", "können"),
                        ("könnt", "können"),
                        ("muss", "müssen"),
                        ("musst", "müssen"),
                        ("müsst", "müssen"),
                        ("darf", "dürfen"),
                        ("darfst", "dürfen"),
                        ("dürft", "dürfen"),
                        ("soll", "sollen"),
                        ("sollst", "sollen"),
                        ("sollt", "sollen"),
                        ("will", "wollen"),
                        ("willst", "wollen"),
                        ("wollt", "wollen"),
                        ("mag", "mögen"),
                        ("magst", "mögen"),
                        ("mögt", "mögen"),
                        ("weiß", "wissen"),
                        ("weißt", "wissen"),
                        ("wisst", "wissen"),
                        ("tue", "tun"),
                        ("tust", "tun"),
                        ("tut", "tun"),
                        ("nimmst", "nehmen"),
                        ("nimmt", "nehmen"),
                        ("isst", "essen"),
                        ("trittst", "treten"),
                        ("tritt", "treten"),
                        ("hältst", "halten"),
                        ("hält", "halten"),
                        ("lässt", "lassen"),
                        ("fällst", "fallen"),
                        ("fällt", "fallen"),
                        ("stößt", "stoßen"),
                        ("liest", "lesen"),
                        ("vergisst", "vergessen"),
                    ] {
                        rules.push(inflection(form, lemma, &["v"], &["vst"], RuleType::Suffix));
                    }
                    rules
                },
                i18n: None,
            },
        ),
        (
            "preterite",
            Transform {
                name: "preterite",
                description: Some("Simple past (Präteritum) and Konjunktiv II"),
                rules: {
                    let mut rules = vec![
                        // Weak Präteritum (-en verbs)
                        inflection("te", "en", &["v"], &["vw"], RuleType::Suffix),
                        inflection("test", "en", &["v"], &["vw"], RuleType::Suffix),
                        inflection("ten", "en", &["v"], &["vw"], RuleType::Suffix),
                        inflection("tet", "en", &["v"], &["vw"], RuleType::Suffix),
                        inflection("ete", "en", &["v"], &["vw"], RuleType::Suffix),
                        inflection("etest", "en", &["v"], &["vw"], RuleType::Suffix),
                        inflection("eten", "en", &["v"], &["vw"], RuleType::Suffix),
                        inflection("etet", "en", &["v"], &["vw"], RuleType::Suffix),
                        // Weak Präteritum (-eln / -ern verbs)
                        inflection("te", "n", &["v"], &["vw"], RuleType::Suffix),
                        inflection("test", "n", &["v"], &["vw"], RuleType::Suffix),
                        inflection("ten", "n", &["v"], &["vw"], RuleType::Suffix),
                        inflection("tet", "n", &["v"], &["vw"], RuleType::Suffix),
                    ];

                    // Strong & irregular Präteritum / Konjunktiv II stems (Suffix handles prefixed verbs too)
                    for (stem, lemma) in [
                        ("war", "sein"),
                        ("wär", "sein"),
                        ("hatte", "haben"),
                        ("hätte", "haben"),
                        ("wurde", "werden"),
                        ("würde", "werden"),
                        ("konnte", "können"),
                        ("könnte", "können"),
                        ("musste", "müssen"),
                        ("müsste", "müssen"),
                        ("durfte", "dürfen"),
                        ("dürfte", "dürfen"),
                        ("sollte", "sollen"),
                        ("wollte", "wollen"),
                        ("mochte", "mögen"),
                        ("möchte", "mögen"),
                        ("wusste", "wissen"),
                        ("wüsste", "wissen"),
                        ("ging", "gehen"),
                        ("stand", "stehen"),
                        ("ständ", "stehen"),
                        ("kam", "kommen"),
                        ("käm", "kommen"),
                        ("nahm", "nehmen"),
                        ("nähm", "nehmen"),
                        ("gab", "geben"),
                        ("gäb", "geben"),
                        ("sah", "sehen"),
                        ("säh", "sehen"),
                        ("aß", "essen"),
                        ("äß", "essen"),
                        ("trank", "trinken"),
                        ("tränk", "trinken"),
                        ("fand", "finden"),
                        ("fänd", "finden"),
                        ("sprach", "sprechen"),
                        ("spräch", "sprechen"),
                        ("schrieb", "schreiben"),
                        ("blieb", "bleiben"),
                        ("fuhr", "fahren"),
                        ("führ", "fahren"),
                        ("fiel", "fallen"),
                        ("hielt", "halten"),
                        ("ließ", "lassen"),
                        ("lief", "laufen"),
                        ("las", "lesen"),
                        ("läs", "lesen"),
                        ("lag", "liegen"),
                        ("läg", "liegen"),
                        ("saß", "sitzen"),
                        ("säß", "sitzen"),
                        ("schlief", "schlafen"),
                        ("schlug", "schlagen"),
                        ("trug", "tragen"),
                        ("traf", "treffen"),
                        ("zog", "ziehen"),
                        ("zög", "ziehen"),
                        ("flog", "fliegen"),
                        ("flög", "fliegen"),
                        ("schloss", "schließen"),
                        ("verlor", "verlieren"),
                        ("gewann", "gewinnen"),
                        ("begann", "beginnen"),
                        ("half", "helfen"),
                        ("warf", "werfen"),
                        ("brach", "brechen"),
                        ("starb", "sterben"),
                        ("bat", "bitten"),
                        ("rief", "rufen"),
                        ("schien", "scheinen"),
                        ("stieg", "steigen"),
                        ("trieb", "treiben"),
                        ("wies", "weisen"),
                        ("hieß", "heißen"),
                        ("tat", "tun"),
                        ("tät", "tun"),
                        ("brachte", "bringen"),
                        ("brächte", "bringen"),
                        ("dachte", "denken"),
                        ("dächte", "denken"),
                        ("kannte", "kennen"),
                        ("nannte", "nennen"),
                        ("rannte", "rennen"),
                        ("brannte", "brennen"),
                        ("sandte", "senden"),
                        ("wandte", "wenden"),
                    ] {
                        if stem.ends_with('e') {
                            let bare = &stem[..stem.len() - 1];
                            rules.push(inflection(stem, lemma, &["v"], &["vst"], RuleType::Suffix));
                            rules.push(inflection(
                                &format!("{bare}est"),
                                lemma,
                                &["v"],
                                &["vst"],
                                RuleType::Suffix,
                            ));
                            rules.push(inflection(
                                &format!("{bare}en"),
                                lemma,
                                &["v"],
                                &["vst"],
                                RuleType::Suffix,
                            ));
                            rules.push(inflection(
                                &format!("{bare}et"),
                                lemma,
                                &["v"],
                                &["vst"],
                                RuleType::Suffix,
                            ));
                        } else {
                            push_preterite_stem_rules(&mut rules, stem, lemma);
                        }
                    }

                    // Konjunktiv II with -e on umlauted strong stems (gäbe, käme, wäre, etc.)
                    for (form, lemma) in [
                        ("wäre", "sein"),
                        ("gäbe", "geben"),
                        ("käme", "kommen"),
                        ("nähme", "nehmen"),
                        ("sähe", "sehen"),
                        ("fände", "finden"),
                        ("spräche", "sprechen"),
                        ("stünde", "stehen"),
                        ("stände", "stehen"),
                        ("ginge", "gehen"),
                        ("ließe", "lassen"),
                        ("hielte", "halten"),
                        ("zöge", "ziehen"),
                        ("flöge", "fliegen"),
                        ("täte", "tun"),
                    ] {
                        rules.push(inflection(form, lemma, &["v"], &["vst"], RuleType::Suffix));
                    }

                    rules
                },
                i18n: None,
            },
        ),
        (
            "imperative",
            Transform {
                name: "imperative",
                description: Some("Irregular stem-changing imperative forms"),
                rules: [
                    ("sprich", "sprechen"),
                    ("gib", "geben"),
                    ("nimm", "nehmen"),
                    ("sieh", "sehen"),
                    ("lies", "lesen"),
                    ("hilf", "helfen"),
                    ("wirf", "werfen"),
                    ("iss", "essen"),
                    ("vergiss", "vergessen"),
                    ("triff", "treffen"),
                    ("brich", "brechen"),
                    ("stirb", "sterben"),
                    ("tritt", "treten"),
                    ("schlaf", "schlafen"),
                    ("fahr", "fahren"),
                    ("lauf", "laufen"),
                    ("lass", "lassen"),
                    ("halt", "halten"),
                    ("hab", "haben"),
                    ("werde", "werden"),
                ]
                .into_iter()
                .map(|(form, lemma)| inflection(form, lemma, &["v"], &["vst"], RuleType::Suffix))
                .collect(),
                i18n: None,
            },
        ),
        (
            "present participle",
            Transform {
                name: "present participle",
                description: Some("Partizip I (-end)"),
                rules: vec![
                    inflection("end", "en", &["adj"], &["v"], RuleType::Suffix),
                    inflection("nd", "n", &["adj"], &["v"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
        (
            "past participle",
            Transform {
                name: "past participle",
                description: Some("Partizip II"),
                rules: get_basic_past_participles()
                    .into_iter()
                    .chain(get_separable_past_participles())
                    .chain([inflection("iert", "ieren", &["adj"], &["vw"], RuleType::Suffix)])
                    .chain(
                        [
                            ("gewesen", "sein"),
                            ("gehabt", "haben"),
                            ("geworden", "werden"),
                            ("worden", "werden"),
                            ("gekonnt", "können"),
                            ("gemusst", "müssen"),
                            ("gedurft", "dürfen"),
                            ("gesollt", "sollen"),
                            ("gewollt", "wollen"),
                            ("gemocht", "mögen"),
                            ("gewusst", "wissen"),
                            ("gegangen", "gehen"),
                            ("gestanden", "stehen"),
                            ("genommen", "nehmen"),
                            ("gegessen", "essen"),
                            ("getrunken", "trinken"),
                            ("gefunden", "finden"),
                            ("gesprochen", "sprechen"),
                            ("geschrieben", "schreiben"),
                            ("geblieben", "bleiben"),
                            ("gezogen", "ziehen"),
                            ("geflogen", "fliegen"),
                            ("geschlossen", "schließen"),
                            ("verloren", "verlieren"),
                            ("gewonnen", "gewinnen"),
                            ("begonnen", "beginnen"),
                            ("geholfen", "helfen"),
                            ("geworfen", "werfen"),
                            ("gebrochen", "brechen"),
                            ("gestorben", "sterben"),
                            ("gebeten", "bitten"),
                            ("geschienen", "scheinen"),
                            ("gestiegen", "steigen"),
                            ("getrieben", "treiben"),
                            ("gewiesen", "weisen"),
                            ("gesessen", "sitzen"),
                            ("gelegen", "liegen"),
                            ("getan", "tun"),
                            ("gebracht", "bringen"),
                            ("gedacht", "denken"),
                            ("gekannt", "kennen"),
                            ("genannt", "nennen"),
                            ("gerannt", "rennen"),
                            ("gebrannt", "brennen"),
                            ("gesandt", "senden"),
                            ("gewandt", "wenden"),
                            ("geboten", "bieten"),
                            ("geflossen", "fließen"),
                            ("genossen", "genießen"),
                            ("gegossen", "gießen"),
                            ("geschossen", "schießen"),
                            ("gebogen", "biegen"),
                            ("gelogen", "lügen"),
                            ("gewogen", "wiegen"),
                            ("gehoben", "heben"),
                            ("gesungen", "singen"),
                            ("gesprungen", "springen"),
                            ("geklungen", "klingen"),
                            ("gezwungen", "zwingen"),
                            ("gelungen", "gelingen"),
                            ("geschnitten", "schneiden"),
                            ("gelitten", "leiden"),
                            ("geritten", "reiten"),
                            ("gestritten", "streiten"),
                            ("gegriffen", "greifen"),
                            ("gebissen", "beißen"),
                            ("gerissen", "reißen"),
                            ("geschmissen", "schmeißen"),
                            ("geschwiegen", "schweigen"),
                        ]
                        .into_iter()
                        .map(|(part, lemma)| {
                            inflection(part, lemma, &["adj"], &["vst"], RuleType::Suffix)
                        }),
                    )
                    .collect(),
                i18n: None,
            },
        ),
        (
            "adjective declension",
            Transform {
                name: "adjective declension",
                description: Some("Inflected adjective, comparative, and superlative endings"),
                rules: vec![
                    inflection("e", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("en", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("n", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("er", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("es", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("em", "", &["adj"], &["adj"], RuleType::Suffix),
                    // Comparative + declension
                    inflection("ere", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("eren", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("erer", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("eres", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("erem", "", &["adj"], &["adj"], RuleType::Suffix),
                    // Superlative + declension
                    inflection("ste", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("sten", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("ster", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("stes", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("stem", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("st", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("este", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("esten", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("ester", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("estes", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("estem", "", &["adj"], &["adj"], RuleType::Suffix),
                    inflection("est", "", &["adj"], &["adj"], RuleType::Suffix),
                    // Umlaut restoration in comparatives/superlatives (größer -> groß, ältesten -> alt)
                    generic_stem_change_rule("ä", "a", "", "", &["adj"], &["adj"]),
                    generic_stem_change_rule("ö", "o", "", "", &["adj"], &["adj"]),
                    generic_stem_change_rule("ü", "u", "", "", &["adj"], &["adj"]),
                ],
                i18n: None,
            },
        ),
        (
            "noun inflection",
            Transform {
                name: "noun inflection",
                description: Some("Noun plural and case endings"),
                rules: vec![
                    inflection("nen", "", &["n"], &["n"], RuleType::Suffix),
                    inflection("en", "", &["n"], &["n"], RuleType::Suffix),
                    inflection("n", "", &["n"], &["n"], RuleType::Suffix),
                    inflection("e", "", &["n"], &["n"], RuleType::Suffix),
                    inflection("ern", "", &["n"], &["n"], RuleType::Suffix),
                    inflection("er", "", &["n"], &["n"], RuleType::Suffix),
                    inflection("es", "", &["n"], &["n"], RuleType::Suffix),
                    inflection("s", "", &["n"], &["n"], RuleType::Suffix),
                    inflection("ses", "s", &["n"], &["n"], RuleType::Suffix),
                    inflection("se", "s", &["n"], &["n"], RuleType::Suffix),
                    // Umlauted plurals (Häuser -> Haus, Bücher -> Buch, Hände -> Hand, Väter -> Vater)
                    generic_stem_change_rule("ä", "a", "", "", &["n"], &["n"]),
                    generic_stem_change_rule("ö", "o", "", "", &["n"], &["n"]),
                    generic_stem_change_rule("ü", "u", "", "", &["n"], &["n"]),
                ],
                i18n: None,
            },
        ),
        (
            "separated prefix",
            Transform {
                name: "separated prefix",
                description: None,
                rules: SEPARABLE_PREFIXES
                    .iter()
                    .map(|&prefix| separated_prefix_rule(prefix, &[], &[]))
                    .collect(),
                i18n: None,
            },
        ),
        (
            "zu-infinitive",
            Transform {
                name: "zu-infinitive",
                description: None,
                rules: SEPARABLE_PREFIXES
                    .iter()
                    .map(|&prefix| {
                        inflection(
                            &format!("{prefix}zu"),
                            prefix,
                            &["v"],
                            &["v"],
                            RuleType::Prefix,
                        )
                    })
                    .collect(),
                i18n: None,
            },
        ),
        (
            "-heit",
            Transform {
                name: "-heit",
                description: Some(
                    "1. Converts an adjective into a noun and usually denotes an abstract quality of the adjectival root. \
                     It is often equivalent to the English suffixes -ness, -th, -ty, -dom:\n\
                     \t schön (“beautiful”) + -heit → Schönheit (“beauty”)\n\
                     \t neu (“new”) + -heit → Neuheit (“novelty”)\n\
                     2. Converts concrete nouns into abstract nouns:\n\
                     \t Kind (“child”) + -heit → Kindheit (“childhood”)\n\
                     \t Christ (“Christian”) + -heit → Christenheit (“Christendom”)\n",
                ),
                rules: vec![
                    inflection("heit", "", &["n"], &["adj", "n"], RuleType::Suffix),
                    inflection("keit", "", &["n"], &["adj", "n"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
    ]))
});

#[cfg(test)]
mod detransforms {
    use crate::{
        de::de_transforms::GERMAN_TRANSFORMS_DESCRIPTOR,
        ja::ja_transforms::{has_term_reasons, LanguageTransformerTestCase, TransformTest},
        transformer::LanguageTransformer,
    };

    #[test]
    fn transforms() {
        let mut lt = LanguageTransformer::new();
        lt.add_descriptor(&GERMAN_TRANSFORMS_DESCRIPTOR).unwrap();

        let tests = [
            TransformTest {
                term: "machen",
                sources: vec![
                    LanguageTransformerTestCase {
                        inner: "mache",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "machst",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "macht",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "machte",
                        rule: "v",
                        reasons: vec!["preterite"],
                    },
                    LanguageTransformerTestCase {
                        inner: "machten",
                        rule: "v",
                        reasons: vec!["preterite"],
                    },
                    LanguageTransformerTestCase {
                        inner: "machend",
                        rule: "v",
                        reasons: vec!["present participle"],
                    },
                    LanguageTransformerTestCase {
                        inner: "gemacht",
                        rule: "v",
                        reasons: vec!["past participle"],
                    },
                    LanguageTransformerTestCase {
                        inner: "gemachten",
                        rule: "v",
                        reasons: vec!["past participle", "adjective declension"],
                    },
                    LanguageTransformerTestCase {
                        inner: "machbar",
                        rule: "v",
                        reasons: vec!["-bar"],
                    },
                ],
            },
            TransformTest {
                term: "fahren",
                sources: vec![
                    LanguageTransformerTestCase {
                        inner: "fährst",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "fährt",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "fuhr",
                        rule: "v",
                        reasons: vec!["preterite"],
                    },
                    LanguageTransformerTestCase {
                        inner: "gefahren",
                        rule: "v",
                        reasons: vec!["past participle"],
                    },
                ],
            },
            TransformTest {
                term: "sprechen",
                sources: vec![
                    LanguageTransformerTestCase {
                        inner: "spricht",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "sprach",
                        rule: "v",
                        reasons: vec!["preterite"],
                    },
                    LanguageTransformerTestCase {
                        inner: "gesprochen",
                        rule: "v",
                        reasons: vec!["past participle"],
                    },
                    LanguageTransformerTestCase {
                        inner: "sprich",
                        rule: "v",
                        reasons: vec!["imperative"],
                    },
                ],
            },
            TransformTest {
                term: "aufmachen",
                sources: vec![
                    LanguageTransformerTestCase {
                        inner: "aufgemacht",
                        rule: "v",
                        reasons: vec!["past participle"],
                    },
                    LanguageTransformerTestCase {
                        inner: "aufzumachen",
                        rule: "v",
                        reasons: vec!["zu-infinitive"],
                    },
                ],
            },
            TransformTest {
                term: "aufstehen",
                sources: vec![
                    LanguageTransformerTestCase {
                        inner: "aufgestanden",
                        rule: "v",
                        reasons: vec!["past participle"],
                    },
                    LanguageTransformerTestCase {
                        inner: "aufstand",
                        rule: "v",
                        reasons: vec!["preterite"],
                    },
                ],
            },
            TransformTest {
                term: "studieren",
                sources: vec![LanguageTransformerTestCase {
                    inner: "studiert",
                    rule: "v",
                    reasons: vec!["past participle"],
                }],
            },
            TransformTest {
                term: "lesen",
                sources: vec![
                    LanguageTransformerTestCase {
                        inner: "lesbar",
                        rule: "v",
                        reasons: vec!["-bar"],
                    },
                    LanguageTransformerTestCase {
                        inner: "unlesbar",
                        rule: "v",
                        reasons: vec!["-bar", "negative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "unlesbare",
                        rule: "v",
                        reasons: vec!["-bar", "negative", "adjective declension"],
                    },
                ],
            },
            TransformTest {
                term: "gut",
                sources: vec![LanguageTransformerTestCase {
                    inner: "gutes",
                    rule: "adj",
                    reasons: vec!["adjective declension"],
                }],
            },
            TransformTest {
                term: "groß",
                sources: vec![LanguageTransformerTestCase {
                    inner: "größer",
                    rule: "adj",
                    reasons: vec!["adjective declension", "adjective declension"],
                }],
            },
            TransformTest {
                term: "alt",
                sources: vec![LanguageTransformerTestCase {
                    inner: "ältesten",
                    rule: "adj",
                    reasons: vec!["adjective declension", "adjective declension"],
                }],
            },
            TransformTest {
                term: "schön",
                sources: vec![
                    LanguageTransformerTestCase {
                        inner: "schönheit",
                        rule: "adj",
                        reasons: vec!["-heit"],
                    },
                    LanguageTransformerTestCase {
                        inner: "schönheiten",
                        rule: "adj",
                        reasons: vec!["-heit", "noun inflection"],
                    },
                ],
            },
            TransformTest {
                term: "haus",
                sources: vec![LanguageTransformerTestCase {
                    inner: "häuser",
                    rule: "n",
                    reasons: vec!["noun inflection", "noun inflection"],
                }],
            },
            TransformTest {
                term: "lernen",
                sources: vec![LanguageTransformerTestCase {
                    inner: "lernung",
                    rule: "v",
                    reasons: vec!["nominalization"],
                }],
            },
        ];

        for test in &tests {
            for case in &test.sources {
                if let Err(e) =
                    has_term_reasons(&lt, case.inner, test.term, Some(case.rule), Some(&case.reasons))
                {
                    panic!("Failed: {e}");
                }
            }
        }

        if let Err(e) = has_term_reasons(
            &lt,
            "mache es auf",
            "mache auf",
            None,
            Some(&vec!["separated prefix"]),
        ) {
            panic!("Failed: {e}");
        }
    }
}

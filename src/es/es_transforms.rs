use fancy_regex::Regex;
use indexmap::IndexMap;
use phf::{phf_map, Map};
use std::sync::LazyLock;

use crate::{
    ja::ja_transforms::{LanguageTransformerTestCase, TransformTest},
    transformer::{
        Condition, ConditionMap, DeinflectFnType, LanguageTransformDescriptor, Rule, RuleType,
        Transform, TransformMap,
    },
    transforms::{generic_stem_change_rule, inflection, special_cased_stem_change_rule},
};

static REFLEXIVE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(me|te|se|nos|os)\s+(\w+)(ar|er|ir)\b").unwrap());

static ACCENTS: Map<&'static str, &'static str> = phf_map! {
    "a" => "á",
    "e" => "é",
    "i" => "í",
    "o" => "ó",
    "u" => "ú",
};

fn add_accent(char: &'static str) -> &'static str {
    if let Some(char) = ACCENTS.get(char) {
        return char;
    }
    char
}

pub static SPANISH_TRANSFORMS_DESCRIPTOR: LazyLock<LanguageTransformDescriptor> =
    LazyLock::new(|| LanguageTransformDescriptor {
        language: "es",
        conditions: &ES_CONDITIONS_MAP,
        transforms: &ES_TRANSFORMS_MAP,
    });

pub static ES_CONDITIONS_MAP: LazyLock<ConditionMap> = LazyLock::new(|| {
    ConditionMap(IndexMap::from([
        (
            "n",
            Condition {
                name: "Noun", // Noun
                is_dictionary_form: true,
                sub_conditions: Some(&["ns", "np"]),
                i18n: None,
            },
        ),
        (
            "np",
            Condition {
                name: "Noun plural", // Noun plural
                is_dictionary_form: false,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "ns",
            Condition {
                name: "Noun singular", // Noun singular
                is_dictionary_form: false,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "v",
            Condition {
                name: "Verb", // Verb
                is_dictionary_form: true,
                sub_conditions: Some(&["v_ar", "v_er", "v_ir"]),
                i18n: None,
            },
        ),
        (
            "v_ar",
            Condition {
                name: "-ar verb", // -ar verb
                is_dictionary_form: false,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "v_er",
            Condition {
                name: "-er verb", // -er verb
                is_dictionary_form: false,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "v_ir",
            Condition {
                name: "-ir verb", // -ir verb
                is_dictionary_form: false,
                sub_conditions: None,
                i18n: None,
            },
        ),
        (
            "adj",
            Condition {
                name: "Adjective", // Adjective
                is_dictionary_form: true,
                sub_conditions: None,
                i18n: None,
            },
        ),
    ]))
});

static ES_TRANSFORMS_MAP: LazyLock<TransformMap> = LazyLock::new(|| {
    TransformMap(IndexMap::from([
        (
            "plural",
            Transform {
                name: "plural",
                description: Some("Plural form of a noun"),
                rules: vec![
                    inflection("s", "", &["np"], &["ns"], RuleType::Suffix),
                    inflection("es", "", &["np"], &["ns"], RuleType::Suffix),
                    inflection("ces", "z", &["np"], &["ns"], RuleType::Suffix),
                ]
                .into_iter()
                .chain(["a", "e", "i", "o", "u"].into_iter().map(|v| {
                    inflection(
                        format!("{v}ses").as_str(),
                        format!("{}s", add_accent(v)).leak(),
                        &["np"],
                        &["ns"],
                        RuleType::Suffix,
                    )
                }))
                .chain(["a", "e", "i", "o", "u"].into_iter().map(|v| {
                    inflection(
                        format!("{v}nes").as_str(),
                        format!("{}n", add_accent(v)).leak(),
                        &["np"],
                        &["ns"],
                        RuleType::Suffix,
                    )
                }))
                .collect(),
                i18n: None,
            },
        ),
        (
            "feminine adjective",
            Transform {
                name: "feminine adjective",
                description: Some("feminine form of an adjective"),
                rules: vec![
                    inflection("a", "o", &["adj"], &["adj"], RuleType::Suffix),
                    // Handles cases like: encantadora -> encantador, española -> español
                    inflection("a", "", &["adj"], &["adj"], RuleType::Suffix),
                ]
                .into_iter()
                .chain(["a", "e", "i", "o"].into_iter().map(|v| {
                    // Handles cases like: dormilona -> dormilón
                    inflection(
                        &format!("{v}na"),
                        format!("{}n", add_accent(v)).leak(),
                        &["adj"],
                        &["adj"],
                        RuleType::Suffix,
                    )
                }))
                .chain(["a", "e", "i", "o"].into_iter().map(|v| {
                    // Handles cases like: francesa -> francés
                    inflection(
                        &format!("{v}sa"),
                        format!("{}s", add_accent(v)).leak(),
                        &["adj"],
                        &["adj"],
                        RuleType::Suffix,
                    )
                }))
                .collect(),
                i18n: None,
            },
        ),
        (
            "present indicative",
            Transform {
                name: "present indicative",
                description: Some("Present indicative form of a verb"),
                rules: vec![
                    // e->ie for -ar verbs
                    generic_stem_change_rule("ie", "e", "(o|as|a|an)", "ar", &["v_ar"], &["v_ar"]),
                    // e->ie for -er verbs
                    generic_stem_change_rule("ie", "e", "(o|es|e|en)", "er", &["v_er"], &["v_er"]),
                    // e->ie for -ir verbs
                    generic_stem_change_rule("ie", "e", "(o|es|e|en)", "ir", &["v_ir"], &["v_ir"]),
                    // o->ue for -ar (with "jugar" special case)
                    special_cased_stem_change_rule(
                        "ue",
                        "jue",
                        "ue",
                        "u",
                        "ue",
                        "o",
                        "(o|as|a|an)",
                        "ar",
                        &["v_ar"],
                        &["v_ar"],
                    ),
                    // o->ue for -er (with "oler" special case)
                    special_cased_stem_change_rule(
                        "ue",
                        "hue",
                        "hue",
                        "o",
                        "ue",
                        "o",
                        "(o|es|e|en)",
                        "er",
                        &["v_er"],
                        &["v_er"],
                    ),
                    // o->ue for -ir (this is a generic rule)
                    generic_stem_change_rule("ue", "o", "(o|es|e|en)", "ir", &["v_ir"], &["v_ir"]),
                    // e->i for -ir (also a generic rule)
                    generic_stem_change_rule("i", "e", "(o|es|e|en)", "ir", &["v_ir"], &["v_ir"]),
                    inflection("o", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("as", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("a", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("amos", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("áis", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("an", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    // -er verbs
                    inflection("o", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("es", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("e", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("emos", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("éis", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("en", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    // -ir verbs
                    inflection("o", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("es", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("e", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("imos", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ís", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("en", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    // i -> y verbs (e.g., incluir, huir, construir...)
                    inflection("uyo", "uir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("uyes", "uir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("uye", "uir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("uyen", "uir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    // -tener verbs
                    inflection("tengo", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tienes", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tiene", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tenemos", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tenéis", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tienen", "tener", &["v"], &["v"], RuleType::Suffix),
                    // -oír verbs
                    inflection("oigo", "oír", &["v"], &["v"], RuleType::Suffix),
                    inflection("oyes", "oír", &["v"], &["v"], RuleType::Suffix),
                    inflection("oye", "oír", &["v"], &["v"], RuleType::Suffix),
                    inflection("oímos", "oír", &["v"], &["v"], RuleType::Suffix),
                    inflection("oís", "oír", &["v"], &["v"], RuleType::Suffix),
                    inflection("oyen", "oír", &["v"], &["v"], RuleType::Suffix),
                    // -venir verbs
                    inflection("vengo", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vienes", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("viene", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("venimos", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("venís", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vienen", "venir", &["v"], &["v"], RuleType::Suffix),
                    // Verbs with Irregular Yo Forms
                    // -guir, -ger, or -gir verbs
                    inflection("go", "guir", &["v"], &["v"], RuleType::Suffix),
                    inflection("jo", "ger", &["v"], &["v"], RuleType::Suffix),
                    inflection("jo", "gir", &["v"], &["v"], RuleType::Suffix),
                    inflection("aigo", "aer", &["v"], &["v"], RuleType::Suffix),
                    inflection("zco", "cer", &["v"], &["v"], RuleType::Suffix),
                    inflection("zco", "cir", &["v"], &["v"], RuleType::Suffix),
                    inflection("hago", "hacer", &["v"], &["v"], RuleType::Suffix),
                    inflection("pongo", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("lgo", "lir", &["v"], &["v"], RuleType::Suffix),
                    inflection("lgo", "ler", &["v"], &["v"], RuleType::Suffix),
                    inflection("doy", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sé", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("veo", "ver", &["v"], &["v"], RuleType::WholeWord),
                    // Ser, estar, ir, haber
                    // ser
                    inflection("soy", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("eres", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("es", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("somos", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sois", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("son", "ser", &["v"], &["v"], RuleType::WholeWord),
                    // estar
                    inflection("estoy", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estás", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("está", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estamos", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estáis", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("están", "estar", &["v"], &["v"], RuleType::WholeWord),
                    // ir
                    inflection("voy", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vas", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("va", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vamos", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vais", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("van", "ir", &["v"], &["v"], RuleType::WholeWord),
                    // haber
                    inflection("he", "haber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("has", "haber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("ha", "haber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("hemos", "haber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("habéis", "haber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("han", "haber", &["v"], &["v"], RuleType::WholeWord),
                ],
                i18n: None,
            },
        ),
        (
            "preterite",
            Transform {
                name: "preterite",
                description: Some("Preterite (past) form of a verb"),
                rules: vec![
                    // e->i for -ir (3rd person)
                    generic_stem_change_rule("i", "e", "(ió|ieron)", "ir", &["v_ir"], &["v_ir"]),
                    // o->u for -ir
                    generic_stem_change_rule("u", "o", "(ió|ieron)", "ir", &["v_ir"], &["v_ir"]),
                    // -ar verbs
                    inflection("é", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("aste", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("ó", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("amos", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("asteis", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("aron", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    // -er verbs
                    inflection("í", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("iste", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ió", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("imos", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("isteis", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ieron", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    // -ir verbs
                    inflection("í", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("iste", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ió", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("imos", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("isteis", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ieron", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    // -car, -gar, -zar verbs
                    inflection("qué", "car", &["v"], &["v"], RuleType::Suffix),
                    inflection("gué", "gar", &["v"], &["v"], RuleType::Suffix),
                    inflection("cé", "zar", &["v"], &["v"], RuleType::Suffix),
                    // -uir verbs
                    inflection("í", "uir", &["v"], &["v"], RuleType::Suffix),
                    // Verbs with irregular forms
                    // ser
                    inflection("fui", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuiste", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fue", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuimos", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuisteis", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fueron", "ser", &["v"], &["v"], RuleType::WholeWord),
                    // ir
                    inflection("fui", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuiste", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fue", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuimos", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuisteis", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fueron", "ir", &["v"], &["v"], RuleType::WholeWord),
                    // dar
                    inflection("di", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("diste", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("dio", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("dimos", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("disteis", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("dieron", "dar", &["v"], &["v"], RuleType::WholeWord),
                    // hacer
                    inflection("hice", "hacer", &["v"], &["v"], RuleType::Suffix),
                    inflection("hiciste", "hacer", &["v"], &["v"], RuleType::Suffix),
                    inflection("hizo", "hacer", &["v"], &["v"], RuleType::Suffix),
                    inflection("hicimos", "hacer", &["v"], &["v"], RuleType::Suffix),
                    inflection("hicisteis", "hacer", &["v"], &["v"], RuleType::Suffix),
                    inflection("hicieron", "hacer", &["v"], &["v"], RuleType::Suffix),
                    // poner
                    inflection("puse", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("pusiste", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("puso", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("pusimos", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("pusisteis", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("pusieron", "poner", &["v"], &["v"], RuleType::Suffix),
                    // decir
                    inflection("dije", "decir", &["v"], &["v"], RuleType::Suffix),
                    inflection("dijiste", "decir", &["v"], &["v"], RuleType::Suffix),
                    inflection("dijo", "decir", &["v"], &["v"], RuleType::Suffix),
                    inflection("dijimos", "decir", &["v"], &["v"], RuleType::Suffix),
                    inflection("dijisteis", "decir", &["v"], &["v"], RuleType::Suffix),
                    inflection("dijeron", "decir", &["v"], &["v"], RuleType::Suffix),
                    // venir
                    inflection("vine", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("viniste", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vino", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vinimos", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vinisteis", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vinieron", "venir", &["v"], &["v"], RuleType::Suffix),
                    // querer
                    inflection("quise", "querer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("quisiste", "querer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("quiso", "querer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("quisimos", "querer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("quisisteis", "querer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("quisieron", "querer", &["v"], &["v"], RuleType::WholeWord),
                    // tener
                    inflection("tuve", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tuviste", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tuvo", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tuvimos", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tuvisteis", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tuvieron", "tener", &["v"], &["v"], RuleType::Suffix),
                    // poder
                    inflection("pude", "poder", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pudiste", "poder", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pudo", "poder", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pudimos", "poder", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pudisteis", "poder", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pudieron", "poder", &["v"], &["v"], RuleType::WholeWord),
                    // saber
                    inflection("supe", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("supiste", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("supo", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("supimos", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("supisteis", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("supieron", "saber", &["v"], &["v"], RuleType::WholeWord),
                    // estar
                    inflection("estuve", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estuviste", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estuvo", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estuvimos", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estuvisteis", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estuvieron", "estar", &["v"], &["v"], RuleType::WholeWord),
                    // andar
                    inflection("anduve", "andar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("anduviste", "andar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("anduvo", "andar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("anduvimos", "andar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("anduvisteis", "andar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("anduvieron", "andar", &["v"], &["v"], RuleType::WholeWord),
                ],
                i18n: None,
            },
        ),
        (
            "imperfect",
            Transform {
                name: "imperfect",
                description: Some("Imperfect form of a verb"),
                rules: vec![
                    // -ar verbs
                    inflection("aba", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("abas", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("aba", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("ábamos", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("abais", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("aban", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    // -er verbs
                    inflection("ía", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ías", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ía", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("íamos", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("íais", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ían", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    // -ir verbs
                    inflection("ía", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ías", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ía", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("íamos", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("íais", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ían", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    // -ir verbs with stem changes (e.g. reír -> reía)
                    inflection("eía", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("eías", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("eía", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("eíamos", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("eíais", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("eían", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    // irregular verbs ir, ser, ver
                    // ser
                    inflection("era", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("eras", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("era", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("éramos", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("erais", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("eran", "ser", &["v"], &["v"], RuleType::WholeWord),
                    // ir
                    inflection("iba", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("ibas", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("iba", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("íbamos", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("ibais", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("iban", "ir", &["v"], &["v"], RuleType::WholeWord),
                    // ver
                    inflection("veía", "ver", &["v"], &["v"], RuleType::WholeWord),
                    inflection("veías", "ver", &["v"], &["v"], RuleType::WholeWord),
                    inflection("veía", "ver", &["v"], &["v"], RuleType::WholeWord),
                    inflection("veíamos", "ver", &["v"], &["v"], RuleType::WholeWord),
                    inflection("veíais", "ver", &["v"], &["v"], RuleType::WholeWord),
                    inflection("veían", "ver", &["v"], &["v"], RuleType::WholeWord),
                ],
                i18n: None,
            },
        ),
        (
            "progressive",
            Transform {
                name: "progressive",
                description: Some("Progressive form of a verb"),
                rules: vec![
                    // e->i for -ir
                    generic_stem_change_rule("i", "e", "(iendo)", "ir", &["v_ir"], &["v_ir"]),
                    // o->u for -er
                    generic_stem_change_rule("u", "o", "(iendo)", "er", &["v_er"], &["v_er"]),
                    // o->u for -ir
                    generic_stem_change_rule("u", "o", "(iendo)", "ir", &["v_ir"], &["v_ir"]),
                    // regular
                    inflection("ando", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("iendo", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("iendo", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    // vowel before the ending (-yendo)
                    inflection("ayendo", "aer", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("eyendo", "eer", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("uyendo", "uir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    // irregular
                    inflection("oyendo", "oír", &["v"], &["v"], RuleType::WholeWord),
                    inflection("yendo", "ir", &["v"], &["v"], RuleType::WholeWord),
                ],
                i18n: None,
            },
        ),
        (
            "imperative",
            Transform {
                name: "imperative",
                description: Some("Imperative form of a verb"),
                rules: vec![
                    // Stem-changing verbs
                    generic_stem_change_rule("ie", "e", "(a|e|en)", "ar", &["v_ar"], &["v_ar"]),
                    generic_stem_change_rule("ie", "e", "(e|a|an)", "er", &["v_er"], &["v_er"]),
                    generic_stem_change_rule("ie", "e", "(e|a|an)", "ir", &["v_ir"], &["v_ir"]),
                    // Special case for 'jugar'
                    special_cased_stem_change_rule(
                        "ue",
                        "jue",
                        "ue",
                        "u",
                        "ue",
                        "o",
                        "(a|ue|uen)",
                        "ar",
                        &["v_ar"],
                        &["v_ar"],
                    ),
                    // Special case for 'oler'
                    special_cased_stem_change_rule(
                        "ue",
                        "hue",
                        "hue",
                        "o",
                        "ue",
                        "o",
                        "(e|a|an)",
                        "er",
                        &["v_er"],
                        &["v_er"],
                    ),
                    // Other stem changes
                    generic_stem_change_rule("ue", "o", "(e|a|an)", "ir", &["v_ir"], &["v_ir"]),
                    generic_stem_change_rule("i", "e", "(e|a|an)", "ir", &["v_ir"], &["v_ir"]),
                    // --- Affirmative Commands ---
                    // -ar verbs
                    inflection("a", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("emos", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("ad", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    // -er verbs
                    inflection("e", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("amos", "ar", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ed", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    // -ir verbs
                    inflection("e", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("amos", "ar", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("id", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    // --- Irregular Affirmative Commands ---
                    inflection("diga", "decir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sé", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("ve", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("ten", "tener", &["v"], &["v"], RuleType::WholeWord),
                    inflection("ven", "venir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("haz", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("di", "decir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pon", "poner", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sal", "salir", &["v"], &["v"], RuleType::WholeWord),
                    // --- Negative Commands ---
                    // -ar verbs
                    inflection("es", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("emos", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("éis", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    // -er verbs
                    inflection("as", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("amos", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("áis", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    // -ir verbs
                    inflection("as", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("amos", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("áis", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
        (
            "conditional",
            Transform {
                name: "conditional",
                description: Some("Conditional form of a verb"),
                rules: vec![
                    // Regular conditional endings
                    inflection("ía", "", &["v"], &["v"], RuleType::Suffix),
                    inflection("ías", "", &["v"], &["v"], RuleType::Suffix),
                    // Note: The third rule for 'ía' is a duplicate and can be omitted,
                    // but included here for a direct 1:1 translation.
                    inflection("ía", "", &["v"], &["v"], RuleType::Suffix),
                    inflection("íamos", "", &["v"], &["v"], RuleType::Suffix),
                    inflection("íais", "", &["v"], &["v"], RuleType::Suffix),
                    inflection("ían", "", &["v"], &["v"], RuleType::Suffix),
                    // Irregular verbs
                    // decir
                    inflection("diría", "decir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("dirías", "decir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("diría", "decir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("diríamos", "decir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("diríais", "decir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("dirían", "decir", &["v"], &["v"], RuleType::WholeWord),
                    // hacer
                    inflection("haría", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("harías", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("haría", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("haríamos", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("haríais", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("harían", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    // poner
                    inflection("pondría", "poner", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pondrías", "poner", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pondría", "poner", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pondríamos", "poner", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pondríais", "poner", &["v"], &["v"], RuleType::WholeWord),
                    inflection("pondrían", "poner", &["v"], &["v"], RuleType::WholeWord),
                    // salir
                    inflection("saldría", "salir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("saldrías", "salir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("saldría", "salir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("saldríamos", "salir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("saldríais", "salir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("saldrían", "salir", &["v"], &["v"], RuleType::WholeWord),
                    // tener
                    inflection("tendría", "tener", &["v"], &["v"], RuleType::WholeWord),
                    inflection("tendrías", "tener", &["v"], &["v"], RuleType::WholeWord),
                    inflection("tendría", "tener", &["v"], &["v"], RuleType::WholeWord),
                    inflection("tendríamos", "tener", &["v"], &["v"], RuleType::WholeWord),
                    inflection("tendríais", "tener", &["v"], &["v"], RuleType::WholeWord),
                    inflection("tendrían", "tener", &["v"], &["v"], RuleType::WholeWord),
                    // venir
                    inflection("vendría", "venir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vendrías", "venir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vendría", "venir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vendríamos", "venir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vendríais", "venir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vendrían", "venir", &["v"], &["v"], RuleType::WholeWord),
                    // querer
                    inflection("querría", "querer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("querrías", "querer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("querría", "querer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("querríamos", "querer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("querríais", "querer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("querrían", "querer", &["v"], &["v"], RuleType::WholeWord),
                    // poder
                    inflection("podría", "poder", &["v"], &["v"], RuleType::WholeWord),
                    inflection("podrías", "poder", &["v"], &["v"], RuleType::WholeWord),
                    inflection("podría", "poder", &["v"], &["v"], RuleType::WholeWord),
                    inflection("podríamos", "poder", &["v"], &["v"], RuleType::WholeWord),
                    inflection("podríais", "poder", &["v"], &["v"], RuleType::WholeWord),
                    inflection("podrían", "poder", &["v"], &["v"], RuleType::WholeWord),
                    // saber
                    inflection("sabría", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sabrías", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sabría", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sabríamos", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sabríais", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sabrían", "saber", &["v"], &["v"], RuleType::WholeWord),
                ],
                i18n: None,
            },
        ),
        (
            "future",
            Transform {
                name: "future",
                description: Some("Future form of a verb"),
                rules: vec![
                    // Regular future endings
                    inflection("é", "", &["v"], &["v"], RuleType::Suffix),
                    inflection("ás", "", &["v"], &["v"], RuleType::Suffix),
                    inflection("á", "", &["v"], &["v"], RuleType::Suffix),
                    inflection("emos", "", &["v"], &["v"], RuleType::Suffix),
                    inflection("éis", "", &["v"], &["v"], RuleType::Suffix),
                    inflection("án", "", &["v"], &["v"], RuleType::Suffix),
                    // Irregular verbs
                    // decir
                    inflection("diré", "decir", &["v"], &["v"], RuleType::Suffix),
                    inflection("dirás", "decir", &["v"], &["v"], RuleType::Suffix),
                    inflection("dirá", "decir", &["v"], &["v"], RuleType::Suffix),
                    inflection("diremos", "decir", &["v"], &["v"], RuleType::Suffix),
                    inflection("diréis", "decir", &["v"], &["v"], RuleType::Suffix),
                    inflection("dirán", "decir", &["v"], &["v"], RuleType::Suffix),
                    // hacer
                    inflection("haré", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("harás", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("hará", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("haremos", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("haréis", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    inflection("harán", "hacer", &["v"], &["v"], RuleType::WholeWord),
                    // poner
                    inflection("pondré", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("pondrás", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("pondrá", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("pondremos", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("pondréis", "poner", &["v"], &["v"], RuleType::Suffix),
                    inflection("pondrán", "poner", &["v"], &["v"], RuleType::Suffix),
                    // salir
                    inflection("saldré", "salir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("saldrás", "salir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("saldrá", "salir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("saldremos", "salir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("saldréis", "salir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("saldrán", "salir", &["v"], &["v"], RuleType::WholeWord),
                    // tener
                    inflection("tendré", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tendrás", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tendrá", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tendremos", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tendréis", "tener", &["v"], &["v"], RuleType::Suffix),
                    inflection("tendrán", "tener", &["v"], &["v"], RuleType::Suffix),
                    // venir
                    inflection("vendré", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vendrás", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vendrá", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vendremos", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vendréis", "venir", &["v"], &["v"], RuleType::Suffix),
                    inflection("vendrán", "venir", &["v"], &["v"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
        (
            "present subjunctive",
            Transform {
                name: "present subjunctive",
                description: Some("Present subjunctive form of a verb"),
                rules: vec![
                    // STEM-CHANGING RULES FIRST
                    // e->ie for -ar
                    generic_stem_change_rule("ie", "e", "(e|es|e|en)", "ar", &["v_ar"], &["v_ar"]),
                    // e->ie for -er
                    generic_stem_change_rule("ie", "e", "(a|as|a|an)", "er", &["v_er"], &["v_er"]),
                    // e->ie for -ir
                    generic_stem_change_rule("ie", "e", "(a|as|a|an)", "ir", &["v_ir"], &["v_ir"]),
                    // o->ue for -ar ("jugar")
                    special_cased_stem_change_rule(
                        "ue",
                        "jue",
                        "ue",
                        "u",
                        "ue",
                        "o",
                        "(ue|ues|ue|uen)",
                        "ar",
                        &["v_ar"],
                        &["v_ar"],
                    ),
                    // o->ue for -er ("oler")
                    special_cased_stem_change_rule(
                        "ue",
                        "hue",
                        "hue",
                        "o",
                        "ue",
                        "o",
                        "(a|as|a|an)",
                        "er",
                        &["v_er"],
                        &["v_er"],
                    ),
                    // o->ue for -ir
                    generic_stem_change_rule("ue", "o", "(a|as|a|an)", "ir", &["v_ir"], &["v_ir"]),
                    // e->i for -ir
                    generic_stem_change_rule("i", "e", "(a|as|a|an)", "ir", &["v_ir"], &["v_ir"]),
                    // Regular subjunctive endings
                    // -ar verbs
                    inflection("e", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("es", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("e", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("emos", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("éis", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("en", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    // -er verbs
                    inflection("a", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("as", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("a", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("amos", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("áis", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("an", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    // -ir verbs
                    inflection("a", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("as", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("a", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("amos", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("áis", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("an", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    // Irregular verbs
                    // dar
                    inflection("dé", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("des", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("dé", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("demos", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("deis", "dar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("den", "dar", &["v"], &["v"], RuleType::WholeWord),
                    // estar
                    inflection("esté", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estés", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("esté", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estemos", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estéis", "estar", &["v"], &["v"], RuleType::WholeWord),
                    inflection("estén", "estar", &["v"], &["v"], RuleType::WholeWord),
                    // ser
                    inflection("sea", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("seas", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sea", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("seamos", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("seáis", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sean", "ser", &["v"], &["v"], RuleType::WholeWord),
                    // ir
                    inflection("vaya", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vayas", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vaya", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vayamos", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vayáis", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("vayan", "ir", &["v"], &["v"], RuleType::WholeWord),
                    // haber
                    inflection("haya", "haber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("hayas", "haber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("haya", "haber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("hayamos", "haber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("hayáis", "haber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("hayan", "haber", &["v"], &["v"], RuleType::WholeWord),
                    // saber
                    inflection("sepa", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sepas", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sepa", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sepamos", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sepáis", "saber", &["v"], &["v"], RuleType::WholeWord),
                    inflection("sepan", "saber", &["v"], &["v"], RuleType::WholeWord),
                ],
                i18n: None,
            },
        ),
        (
            "imperfect subjunctive",
            Transform {
                name: "imperfect subjunctive",
                description: Some("Imperfect subjunctive form of a verb"),
                rules: vec![
                    // -ar verbs
                    inflection("ara", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("ase", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("aras", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("ases", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("ara", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("ase", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("áramos", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("ásemos", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("arais", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("aseis", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("aran", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("asen", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    // -er verbs
                    inflection("iera", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("iese", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ieras", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ieses", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("iera", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("iese", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("iéramos", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("iésemos", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ierais", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ieseis", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ieran", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("iesen", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    // -ir verbs
                    inflection("iera", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("iese", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ieras", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ieses", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("iera", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("iese", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("iéramos", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("iésemos", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ierais", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ieseis", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("ieran", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("iesen", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    // irregular verbs
                    // ser
                    inflection("fuera", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuese", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fueras", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fueses", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuera", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuese", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuéramos", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuésemos", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuerais", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fueseis", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fueran", "ser", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuesen", "ser", &["v"], &["v"], RuleType::WholeWord),
                    // ir
                    inflection("fuera", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuese", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fueras", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fueses", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuera", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuese", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuéramos", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuésemos", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuerais", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fueseis", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fueran", "ir", &["v"], &["v"], RuleType::WholeWord),
                    inflection("fuesen", "ir", &["v"], &["v"], RuleType::WholeWord),
                ],
                i18n: None,
            },
        ),
        (
            "participle",
            Transform {
                name: "participle",
                description: Some("Participle form of a verb"),
                rules: vec![
                    inflection("ado", "ar", &["adj"], &["v_ar"], RuleType::Suffix),
                    inflection("ido", "er", &["adj"], &["v_er"], RuleType::Suffix),
                    inflection("ido", "ir", &["adj"], &["v_ir"], RuleType::Suffix),
                    // IRREGULAR PAST PARTICIPLES
                    inflection("oído", "oír", &["adj"], &["v"], RuleType::Suffix),
                    inflection("dicho", "decir", &["adj"], &["v"], RuleType::WholeWord),
                    inflection("escrito", "escribir", &["adj"], &["v"], RuleType::WholeWord),
                    inflection("hecho", "hacer", &["adj"], &["v"], RuleType::WholeWord),
                    inflection("muerto", "morir", &["adj"], &["v"], RuleType::WholeWord),
                    inflection("puesto", "poner", &["adj"], &["v"], RuleType::WholeWord),
                    inflection("roto", "romper", &["adj"], &["v"], RuleType::WholeWord),
                    inflection("visto", "ver", &["adj"], &["v"], RuleType::WholeWord),
                    inflection("vuelto", "volver", &["adj"], &["v"], RuleType::WholeWord),
                ],
                i18n: None,
            },
        ),
        (
            "reflexive",
            Transform {
                name: "reflexive",
                description: Some("Reflexive form of a verb"),
                rules: vec![
                    // -ar reflexive verbs: replace 'ar' with 'arse'
                    // e.g., lavar -> lavarse
                    inflection("arse", "ar", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    // -er reflexive verbs: replace 'er' with 'erse'
                    // e.g., poner -> ponerse
                    inflection("erse", "er", &["v_er"], &["v_er"], RuleType::Suffix),
                    // -ir reflexive verbs: replace 'ir' with 'irse'
                    // e.g., vestir -> vestirse
                    inflection("irse", "ir", &["v_ir"], &["v_ir"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
        (
            "pronoun substitution",
            Transform {
                name: "pronoun substitution",
                description: Some("Substituted pronoun of a reflexive verb"),
                rules: vec![
                    // Rules for -ar verbs (e.g., lavarse -> lavarme, lavarte, lavarnos)
                    inflection("arme", "arse", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    inflection("arte", "arse", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    // Corrected v_er to v_ar, as 'arnos' derives from an -ar verb.
                    inflection("arnos", "arse", &["v_ar"], &["v_ar"], RuleType::Suffix),
                    // Rules for -er verbs (e.g., ponerse -> ponerme, ponerte, ponernos)
                    inflection("erme", "erse", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("erte", "erse", &["v_er"], &["v_er"], RuleType::Suffix),
                    inflection("ernos", "erse", &["v_er"], &["v_er"], RuleType::Suffix),
                    // Rules for -ir verbs (e.g., vestirse -> vestirme, vestirte, vestirnos)
                    inflection("irme", "irse", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("irte", "irse", &["v_ir"], &["v_ir"], RuleType::Suffix),
                    inflection("irnos", "irse", &["v_ir"], &["v_ir"], RuleType::Suffix),
                ],
                i18n: None,
            },
        ),
        (
            "pronominal",
            Transform {
                name: "pronominal",
                description: Some("Pronominal form of a verb"),
                rules: vec![Rule {
                    rule_type: RuleType::Other,
                    is_inflected: REFLEXIVE_PATTERN.clone(), // Use the LazyLock Regex
                    deinflected: None,
                    // Use a custom deinflect function
                    deinflect_fn: DeinflectFnType::Pronominal,
                    inflected_str: Some(r"\b(me|te|se|nos|os)\s+(\w+)(ar|er|ir)\b".to_string()),
                    conditions_in: &["v"],
                    conditions_out: &["v"],
                }],
                i18n: None,
            },
        ),
    ]))
});

pub(crate) static ES_TRANSFORM_TESTS: LazyLock<[&[TransformTest]; 5]> = LazyLock::new(|| {
    [
        &*ES_PRESENT_INDICITIVE_VERB_TESTS,
        &*ES_NOUN_TESTS,
        &*ES_FEMININE_ADJECTIVE_TESTS,
        &*ES_PARTICIPLE_TESTS,
        &*ES_REFLEXIVE_TESTS,
    ]
});

pub(crate) static ES_PRESENT_INDICITIVE_VERB_TESTS: LazyLock<[TransformTest; 3]> =
    LazyLock::new(|| {
        [
            TransformTest {
                term: "hablar",
                sources: vec![
                    LanguageTransformerTestCase {
                        inner: "hablo",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "hablas",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "habla",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "hablamos",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "habláis",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "hablan",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                ],
            },
            TransformTest {
                term: "comer",
                sources: vec![
                    LanguageTransformerTestCase {
                        inner: "como",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "comes",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "come",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "comemos",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "coméis",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "comen",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                ],
            },
            TransformTest {
                term: "vivir",
                sources: vec![
                    LanguageTransformerTestCase {
                        inner: "vivo",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "vives",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "vive",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "vivimos",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "vivís",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                    LanguageTransformerTestCase {
                        inner: "viven",
                        rule: "v",
                        reasons: vec!["present indicative"],
                    },
                ],
            },
        ]
    });

pub(crate) static ES_NOUN_TESTS: LazyLock<[TransformTest; 11]> = LazyLock::new(|| {
    [
        TransformTest {
            term: "gato",
            sources: vec![LanguageTransformerTestCase {
                inner: "gatos",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
        TransformTest {
            term: "sofá",
            sources: vec![LanguageTransformerTestCase {
                inner: "sofás",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
        TransformTest {
            term: "tisú",
            sources: vec![LanguageTransformerTestCase {
                inner: "tisús",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
        TransformTest {
            term: "tisú",
            sources: vec![LanguageTransformerTestCase {
                inner: "tisúes",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
        TransformTest {
            term: "autobús",
            sources: vec![LanguageTransformerTestCase {
                inner: "autobuses",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
        TransformTest {
            term: "ciudad",
            sources: vec![LanguageTransformerTestCase {
                inner: "ciudades",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
        TransformTest {
            term: "clic",
            sources: vec![LanguageTransformerTestCase {
                inner: "clics",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
        TransformTest {
            term: "sí",
            sources: vec![LanguageTransformerTestCase {
                inner: "síes",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
        TransformTest {
            term: "zigzag",
            sources: vec![LanguageTransformerTestCase {
                inner: "zigzags",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
        TransformTest {
            term: "luz",
            sources: vec![LanguageTransformerTestCase {
                inner: "luces",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
        TransformTest {
            term: "canción",
            sources: vec![LanguageTransformerTestCase {
                inner: "canciones",
                rule: "ns",
                reasons: vec!["plural"],
            }],
        },
    ]
});

pub(crate) static ES_FEMININE_ADJECTIVE_TESTS: LazyLock<[TransformTest; 4]> = LazyLock::new(|| {
    [
        // Rule: suffixInflection('a', 'o', ['adj'], ['adj'])
        TransformTest {
            term: "rojo",
            sources: vec![LanguageTransformerTestCase {
                inner: "roja",
                rule: "adj",
                reasons: vec!["feminine adjective"],
            }],
        },
        // Rule: suffixInflection('a', '', ['adj'], ['adj'])
        TransformTest {
            term: "español",
            sources: vec![LanguageTransformerTestCase {
                inner: "española",
                rule: "adj",
                reasons: vec!["feminine adjective"],
            }],
        },
        // Rule: ...map((v) => suffixInflection(`${v}na`, `${addAccent(v)}n`...
        TransformTest {
            term: "dormilón",
            sources: vec![LanguageTransformerTestCase {
                inner: "dormilona",
                rule: "adj",
                reasons: vec!["feminine adjective"],
            }],
        },
        // Rule: ...map((v) => suffixInflection(`${v}sa`, `${addAccent(v)}s`...
        TransformTest {
            term: "francés",
            sources: vec![LanguageTransformerTestCase {
                inner: "francesa",
                rule: "adj",
                reasons: vec!["feminine adjective"],
            }],
        },
    ]
});

pub(crate) static ES_PARTICIPLE_TESTS: LazyLock<[TransformTest; 5]> = LazyLock::new(|| {
    [
        // -ar verbs: ado -> ar
        TransformTest {
            term: "escuchar",
            sources: vec![LanguageTransformerTestCase {
                inner: "escuchado",
                rule: "v", // Using 'v' is okay, as 'v_ar' is a sub-condition
                reasons: vec!["participle"],
            }],
        },
        // -er verbs: ido -> er
        TransformTest {
            term: "comer",
            sources: vec![LanguageTransformerTestCase {
                inner: "comido",
                rule: "v",
                reasons: vec!["participle"],
            }],
        },
        // -ir verbs: ido -> ir
        TransformTest {
            term: "vivir",
            sources: vec![LanguageTransformerTestCase {
                inner: "vivido",
                rule: "v",
                reasons: vec!["participle"],
            }],
        },
        // Irregular: dicho -> decir
        TransformTest {
            term: "decir",
            sources: vec![LanguageTransformerTestCase {
                inner: "dicho",
                rule: "v",
                reasons: vec!["participle"],
            }],
        },
        // Irregular: roto -> romper
        TransformTest {
            term: "romper",
            sources: vec![LanguageTransformerTestCase {
                inner: "roto",
                rule: "v",
                reasons: vec!["participle"],
            }],
        },
    ]
});

pub(crate) static ES_REFLEXIVE_TESTS: LazyLock<[TransformTest; 3]> = LazyLock::new(|| {
    [
        // 'reflexive' transform: lavarse -> lavar
        // Your JS had term: 'lavar', source: 'lavarse'. This is backwards.
        // The de-inflector goes from source -> term. So, `lavarse` de-inflects to `lavar`.
        TransformTest {
            term: "lavar",
            sources: vec![LanguageTransformerTestCase {
                inner: "lavarse",
                rule: "v",
                reasons: vec!["reflexive"],
            }],
        },
        // 'pronoun substitution' transform: lavarte -> lavarse
        TransformTest {
            term: "lavarse",
            sources: vec![LanguageTransformerTestCase {
                inner: "lavarte",
                rule: "v",
                reasons: vec!["pronoun substitution"],
            }],
        },
        // 'pronominal' transform: me lavar -> lavarse
        // Note: The JS test `me lavar` is slightly malformed. Spanish grammar
        // would require 'lavarme' or 'me lavo'. The pattern is for `pronoun + infinitive`.
        // Let's test `me despertar` as a better example since `despertar` is often reflexive.
        TransformTest {
            term: "despertarse",
            sources: vec![LanguageTransformerTestCase {
                inner: "me despertar",
                rule: "v",
                reasons: vec!["pronominal"],
            }],
        },
    ]
});

#[cfg(test)]
mod estransforms {
    use crate::{
        es::es_transforms::{ES_TRANSFORM_TESTS, SPANISH_TRANSFORMS_DESCRIPTOR},
        ja::ja_transforms::has_term_reasons,
        transformer::LanguageTransformer,
    };

    #[test]
    fn transforms() {
        let mut lt = LanguageTransformer::new();
        lt.add_descriptor(&SPANISH_TRANSFORMS_DESCRIPTOR).unwrap();

        for test_vec in ES_TRANSFORM_TESTS.into_iter() {
            for test in test_vec {
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
}

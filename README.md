### Multi Language Transformer (for [yomichan_rs](https://github.com/aramrw/yomichan_rs))
- This library is the language agnostic internal deinflector for [yomichan_rs](https://github.com/aramrw/yomichan_rs). 
- It attempts to be a 1 to 1 reimplementation of [Yomitan's](https://github.com/yomidevs/yomitan?tab=readme-ov-file#yomitan) [MultiLanguageTransformer](https://github.com/yomidevs/yomitan/blob/2fc09f9b2d2f130ea18ae117be15f5683bc13440/ext/js/language/multi-language-transformer.js#L21). 

### Roadmap (Soft Priority Ordering)
- [x] Japanese
- [x] English
- [x] Spanish
- [x] German
- [ ] Arabic
- [ ] Farsi
- [ ] Russian
- [ ] Korean
- [ ] Italian

### Usage Example (Library)

```rust
use deinflector::multi_language_transformer::MultiLanguageTransformer;

fn main() {
    let mlt = MultiLanguageTransformer::default();

    for candidate in mlt.transform("de", "aufgemacht") {
        let reasons: Vec<&str> = candidate.trace.iter().map(|f| f.transform.as_str()).collect();
        println!("{} ({:?})", candidate.text, reasons);
    }

    // Also works with "ja", "en", "es", etc.
    for candidate in mlt.transform("es", "atorada") {
        println!("{}", candidate.text);
    }
}
```

### Usage Example (CLI)

```sh
# Deinflect a word
cargo run --bin deinflector -- -l de aufgemacht
cargo run --bin deinflector -- -l es --json atorada

# Match tokens in a sentence against a dictionary form
cargo run --bin deinflector -- match -l es --target atorar --json comida atorada

# Persistent JSON-lines server mode (stdin -> stdout)
cargo run --bin deinflector -- serve
```

### Adding a new language
- Should read this first before adding a language [Adding Language Yomitan Docs](https://github.com/yomidevs/yomitan/blob/master/docs/development/language-features.md)
#### Rough Outline
- Fork this repo's main branch, then make a branch for your PR. 
- Find new [language/`<iso>` folder](https://github.com/yomidevs/yomitan/tree/2fc09f9b2d2f130ea18ae117be15f5683bc13440/ext/js/language) from Yomitan
- Translate `<iso>`.transforms (and helper) files to rust
   - Convert Yomitan's `LanguageTransformer` js tests to rust tests at the bottom of the new files.

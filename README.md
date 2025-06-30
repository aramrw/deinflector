### Multi Language Transformer (for [yomichan_rs](https://github.com/aramrw/yomichan_rs))
- This library is the language agnostic internal deinflector for [yomichan_rs](https://github.com/aramrw/yomichan_rs). 
- It attempts to be a 1 to 1 reimplementation of [Yomitan's](https://github.com/yomidevs/yomitan?tab=readme-ov-file#yomitan) [MultiLanguageTransformer](https://github.com/yomidevs/yomitan/blob/2fc09f9b2d2f130ea18ae117be15f5683bc13440/ext/js/language/multi-language-transformer.js#L21). 

### Roadmap (Soft Priority Ordering)
- [x] Japanese
- [x] English
- [x] Spanish
- [ ] Arabic
- [ ] Farsi
- [ ] Russian
- [ ] Korean
- [ ] Italian

### [Adding Language Yomitan Docs](https://github.com/yomidevs/yomitan/blob/master/docs/development/language-features.md)
#### Rough Outline
- Fork this repo's main branch, then make a branch for your PR. 
- Find new [language/`<iso>` folder](https://github.com/yomidevs/yomitan/tree/2fc09f9b2d2f130ea18ae117be15f5683bc13440/ext/js/language) from Yomitan
- Translate `<iso>`.transforms (and helper) files to rust
   - Convert Yomitan's `LanguageTransformer` js tests to rust tests at the bottom of the new files.

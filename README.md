# XML

XML for Flareite: highlighting from a Tree-sitter grammar compiled to WebAssembly, and closing
tags typed for you.

## Closing tags

Typing the `>` of a start tag adds its end tag after the cursor: `<item id="1">` gets `</item>`.
Typing `</` finishes the innermost open element's end tag. Empty-element tags (`<item/>`) get
none, and nothing is added inside comments, CDATA sections, processing instructions, or
attribute values. Names keep their case. Undo takes back just the added tag. Reading the text
before the cursor is why the extension asks to read documents.

## Building

Closing tags are a small Rust extension (`src/lib.rs`, `src/tags.rs`), built against the
extension API in the editor's repository, checked out beside this one as `editor`:

```sh
cargo build --release
cargo test --target x86_64-pc-windows-msvc   # or your platform's target: the tests run natively
```

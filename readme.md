# 🤓 Asciidork

> An Asciidoc parser/backend written in Rust

## Installation

```sh
# requires rust/cargo toolchain
cargo install asciidork-cli
```

<details>

<summary>or install from source</summary>

```sh
git clone https://github.com/jaredh159/asciidork
cd asciidork
cargo build --release --bins

# vvvv -- OPTIONAL: or use rel path to `./target/release/asciidork` instead
sudo cp ./target/release/asciidork /usr/local/bin
```

</details>

## Usage

```sh
# read the friendly manual
asciidork --help

# parse/convert/print from a file
asciidork --input test.adoc

# without a --input arg, it reads from stdin
echo "foo _bar_ *baz*" | asciidork
asciidork < test.adoc

# don't include enclosing document structure by passing `--embedded`
asciidork --input test.adoc --embedded

# send output to a file (alternatively just redirect stdout)
asciidork --input test.adoc --embedded --output test.html

# print information about perf (did i mention it's written in Rust btw?)
asciidork --input test.adoc --print-timings

# print pretty html (requires `pretter` -- install w/ `npm i -g prettier`)
asciidork --input test.adoc --embedded --format dr-html-prettier
```

## WASM

The Asciidork parser and dr-html backend compiles to WASM to run in the browser! (Did I
mention it's written in Rust?) NPM package coming soon, but for now you can see it in
action here:

https://asciidork-playground.vercel.app

Be sure to pop the dev tools to see timing info.

## Caveats

> [!WARNING]
> Asciidork is not complete. It implements a majority of the
> documented behavior of Asciidoc, but there are several unfinished areas,
> missing error handling, edge cases galore! Consider it a technology preview
> only for now.

Known **unfinished or unimplemented** areas include:

- [ ] CSV Tables
- [ ] STEM
- [ ] Source highlighting
- [ ] Include directives
- [ ] All entity refs

PRs welcome! 👍

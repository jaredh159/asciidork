_default:
  @just --choose

build-playground:
  @cd dr-html-wasm && wasm-pack build \
    --target web \
    --out-dir ../web-playground/public/wasm

playground: build-playground
  @cd web-playground && pnpm run dev

check:
  @export RUSTFLAGS="-D warnings" && \
    cargo check && \
    cargo clippy && \
    cargo fmt -- --check && \
    cargo test --all --no-fail-fast && \
    cargo build
  @just check-ast-json
  @just build-playground

check-ast-json:
  @cargo run -- --input kitchen-sink.adoc --format ast | jq &> /dev/null

reset-wasm:
  @git restore web-playground/public/wasm/dr_html_wasm_bg.wasm

publish type:
  @cargo workspaces publish {{type}} --message "release v%v" --no-individual-tags

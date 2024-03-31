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
  @just build-playground

reset-wasm:
  @git restore web-playground/public/wasm/dr_html_wasm_bg.wasm

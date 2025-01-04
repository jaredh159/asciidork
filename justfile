_default:
  @just --choose

build-playground:
  @cd dr-html-wasm && RUSTFLAGS='-D warnings -A unused-imports' wasm-pack build \
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

test-parser:
  @cd parser && bacon test

test-backend:
  @cd dr-html-backend && bacon test

reset-wasm:
  @git restore web-playground/public/wasm/dr_html_wasm_bg.wasm

publish type:
  @cargo workspaces publish {{type}} --message "release v%v" --no-individual-tags

fixtures:
  @cd cli/tests/all && bash make-fixtures.sh

_default:
  @just --choose

check:
  @export RUSTFLAGS="-D warnings" && \
  cargo check && \
  cargo clippy && \
  cargo fmt -- --check && \
  cargo test --all --no-fail-fast && \
  cargo build

watch-print-ast file:
  @watchexec --no-vcs-ignore --restart --clear \
  --watch ast/src \
  --watch backend/src \
  --watch backend-asciidoctor-html/src \
  --watch cli/src \
  --watch eval/src \
  --watch parser/src \
  --watch {{file}} \
  cargo run print-ast {{file}}

watch-print-html file:
  @watchexec --no-vcs-ignore --restart --clear \
  --watch ast/src \
  --watch backend/src \
  --watch backend-asciidoctor-html/src \
  --watch cli/src \
  --watch eval/src \
  --watch parser/src \
  --watch {{file}} \
  cargo run print-html {{file}}


watch-test isolate="":
  @watchexec --restart --clear \
  --watch ast/src \
  --watch backend/src \
  --watch backend-asciidoctor-html/src \
  --watch backend-asciidoctor-html/tests \
  --watch cli/src \
  --watch eval/src \
  --watch parser/src \
  cargo test {{isolate}}

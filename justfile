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
  @watchexec --no-vcs-ignore --restart --clear --watch parser/src --watch adork-cli/src --watch {{file}} cargo run print-ast {{file}}

watch-test isolate="":
  @watchexec --restart --clear --watch parser/src --watch adork-cli/src --watch test-utils/src cargo test {{isolate}}

test-new isolate="":
  @cd parser && watchexec --restart --clear --watch . cargo test {{isolate}}

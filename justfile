_default:
  @just --choose


watch-print-ast file:
  @watchexec --no-vcs-ignore --restart --clear --watch adork/src --watch adork-cli/src --watch {{file}} cargo run print-ast {{file}}

watch-test isolate="":
  @watchexec --restart --clear --watch adork/src --watch adork-cli/src --watch test-utils/src cargo test {{isolate}}

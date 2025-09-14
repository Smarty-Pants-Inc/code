# Upstream Merge Plan

- Mode: by-bucket (merge upstream/main into upstream-merge)
- Policy application:
  - prefer_ours_globs: keep our forked UI/core wiring; only adopt upstream if clearly compatible and beneficial.
  - prefer_theirs_globs: adopt upstream for common/exec/file-search unless it breaks build or fork-specific invariants.
  - purge_globs: ensure reintroduced CLI images are removed.
- Buckets to reconcile:
  1) Core protocol + models and UA/version helpers
  2) Tools and handlers (openai_tools, browser_*, agent_*, web_fetch)
  3) TUI components and history/rendering
  4) Common/exec/file-search crates (lean upstream)
  5) Workflows/docs/scripts (preserve ours unless security/correctness fix)
- Invariants to preserve:
  - Tool handler↔schema parity (verify.sh)
  - Browser gating and screenshot queue semantics
  - codex_version::version() and get_codex_user_agent_default()
  - codex-core public re-exports + models alias
- Procedure:
  - git merge --no-ff --no-commit upstream/main
  - Resolve conflicts honoring policy globs
  - Remove purged assets if reintroduced
  - Run scripts/upstream-merge/verify.sh and ./build-fast.sh
  - Commit with conventional message and push

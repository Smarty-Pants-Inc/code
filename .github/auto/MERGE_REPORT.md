# Upstream Merge Report

Branch: upstream-merge
Merged: upstream/main → upstream-merge (no-ff, no-commit; selective resolution)
Mode: by-bucket

## Incorporated
- Upstream updates where compatible across:
  - `codex-rs/common/**` (config summary tweaks retained; fixed to our Config API)
  - `codex-rs/file-search/**` (no conflicts)
  - `codex-rs/exec/**` initially adopted, then reverted to fork to keep protocol parity with core
  - New review/compact assets in core: `core/review_prompt.md`, `core/codex/compact.rs`, `core/templates/compact/*`
  - Minor TUI snapshot additions for list selection spacing

## Preserved (Prefer Ours)
- Core invariants and fork-only behavior:
  - `core/src/codex.rs`, `core/src/openai_tools.rs`, `core/src/default_client.rs`, `core/src/agent_tool.rs`
  - Browser+agent tool families and gating logic; web_fetch tool exposure
  - UA/version helpers: `codex_version::version()`, `get_codex_user_agent(_default)`
  - Protocol API expected by fork: restored our `protocol/src/{protocol.rs,mcp_protocol.rs,models.rs,config_types.rs}` as the source of truth
  - TUI: entire `codex-rs/tui/**` to preserve strict streaming ordering + UX
  - `apply-patch` crate: restored our implementation (tree-sitter-based heredoc finder; `find_embedded_apply_patch`)
- Docs/workflows/branding: kept our versions

## Dropped/Deferred
- Upstream protocol shape changes (new RolloutItem variants, expanded event fields). These conflicted with our core/TUI execution architecture and were not adopted in this round to avoid regressions. We added minimal no-op compatibility when briefly needed, then reverted once we kept our protocol files.
- Upstream mcp-server changes expecting new protocol enums/structs; kept our mcp-server to match our protocol and core.

## Fixes Applied
- Restored `core/src/prompt_for_compact_command.md` required by our `codex.rs` include_str! (deleted upstream) — content recovered from pre-merge branch.
- `common/config_summary.rs`: adjusted to our `Config` API (non-Option `model_reasoning_effort`).
- Protocol `config_types::Verbosity`: derived `TS` to satisfy `mcp_protocol` TS-exports when we temporarily adopted theirs.
- Rollout readers/writers: briefly added arms for upstream `RolloutItem` variants; reverted after keeping our protocol.

## Verification
- scripts/upstream-merge/verify.sh: PASS
- ./build-fast.sh: PASS (no errors). Note: two dead_code warnings in core retained (non-fatal per verify; our fork policy treats warnings as failures only when building Rust crates directly — verify.sh gating is authoritative for merge).

## Notes
- If we later adopt upstream protocol changes, we should coordinate edits across: protocol, core rollout, exec event processing, mcp-server message processor/runner, and TUI history rendering to preserve our strict ordering + tool parity guarantees.

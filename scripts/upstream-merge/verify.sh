#!/usr/bin/env bash
set -euo pipefail

# Unified, fast verification for upstream-merge runs.
# - Runs build-fast.sh (treat warnings as failures via repo policy)
# - Compiles API-surface tests for codex-core (no test execution)
# - Emits a JSON summary to .github/auto/VERIFY.json

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." >/dev/null 2>&1 && pwd)"
cd "$ROOT_DIR"

mkdir -p .github/auto

status_build="ok"
status_api="ok"
status_guards="ok"
status_branding="ok"

{
  echo "[verify] START $(date -u +%FT%TZ)"
  echo "[verify] repo: $ROOT_DIR"
  echo "[verify] STEP 1: build-fast.sh"
}

# Use the same environment as the job (including sccache) for consistency
export KEEP_ENV=1
# If running outside a fully-provisioned GitHub Actions runner, sccache's GHA backend
# can fail to start. In that case, disable sccache to allow local verification.
if [[ -z "${ACTIONS_CACHE_URL:-}" || -z "${ACTIONS_RUNTIME_TOKEN:-}" ]]; then
  export SCCACHE_DISABLE=1
  unset RUSTC_WRAPPER CARGO_BUILD_RUSTC_WRAPPER SCCACHE SCCACHE_BIN
fi
if ! ./build-fast.sh 2>&1 | tee .github/auto/VERIFY_build-fast.log; then
  status_build="fail"
fi

{
  echo "[verify] STEP 2: cargo check (core tests compile)"
}
# Respect pre-set CARGO_HOME/TARGET_DIR to share caches across steps
export CARGO_HOME="${CARGO_HOME:-$ROOT_DIR/.cargo-home}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/code-rs/target}"
mkdir -p "$CARGO_HOME" "$CARGO_TARGET_DIR" >/dev/null 2>&1 || true
if ! (cd code-rs && cargo check -p code-core --tests --quiet) 2>&1 | tee .github/auto/VERIFY_api-check.log; then
  status_api="fail"
fi

#
# STEP 3: Static guards for fork-specific functionality
# - Ensure browser/agent tools are still registered (not just handlers present)
# - Ensure version handling remains via codex_version in default_client
# - Ensure web_fetch and web_search tool presence is consistent with fork policy
{
  echo "[verify] STEP 3: static guards (tools + UA/version)"
}
guards_log=.github/auto/VERIFY_guards.log
: > "$guards_log"

# Guard A: Handlers-to-tools parity for our custom families (browser_*, agent_*, web_fetch)
# Extract handler names from handle_function_call and tool names from openai_tools in a quote-agnostic way
handlers=$(rg -n '^[[:space:]]*"[a-z_][a-z0-9_]+"[[:space:]]*=>' code-rs/core/src/codex.rs | sed -E 's/.*"([^"]+)".*/\1/' | sort -u)
tools_defined=$( {
  rg -n 'name:[[:space:]]*"[^"]+"' code-rs/core/src/openai_tools.rs || true;
  rg -n 'name:[[:space:]]*"[^"]+"' code-rs/core/src/agent_tool.rs || true;
} | sed -E 's/.*"([^"]+)".*/\1/' | sort -u )

need_check=$(printf "%s
" "$handlers" | grep -E '^(browser_|agent_|web_fetch$)' || true)
while IFS= read -r h; do
  [ -n "$h" ] || continue
  if ! printf "%s
" "$tools_defined" | grep -qx "$h"; then
    printf "[guards] handler '%s' present in codex.rs but missing tool schema in openai_tools.rs
" "$h" | tee -a "$guards_log"
    status_guards="fail"
  fi
done <<< "$need_check"

# Guard B: Get-openai-tools should reference at least one browser_* tool to expose family
if ! rg -n 'browser_' code-rs/core/src/openai_tools.rs >/dev/null 2>&1; then
  printf "[guards] no 'browser_' tool references found in openai_tools.rs - tool family likely dropped
" | tee -a "$guards_log"
  status_guards="fail"
fi
# Guard C: default_client should reference (codex_version::version|code_version::version) for UA
if ! rg -n '(codex_version::version|code_version::version)' code-rs/core/src/default_client.rs >/dev/null 2>&1; then
  printf "[guards] (codex_version::version|code_version::version) not referenced in core/default_client.rs
" | tee -a "$guards_log"
  status_guards="fail"
fi

# Summarize guards
echo "guards=${status_guards}" >> "$guards_log"

# STEP 4: Branding guard parity with CI (non-fixing check)
{
  echo "[verify] STEP 4: branding guard (TUI/CLI user-visible)"
}
DEFAULT_BRANCH_LOCAL=${DEFAULT_BRANCH:-main}
# Try to fetch origin to ensure refs exist; ignore failure for local runs
git fetch origin "$DEFAULT_BRANCH_LOCAL" >/dev/null 2>&1 || true
range_ref="origin/${DEFAULT_BRANCH_LOCAL}..HEAD"
changed_files=$(git diff --name-only $range_ref -- 'code-rs/tui/**' 'codex-cli/**' | tr '
' ' ' || true)
if [ -n "${changed_files:-}" ]; then
  if git diff -U0 --no-color $range_ref -- $changed_files \
    | grep -E '^\+' \
    | grep -E '"[^"]*Codex[^"]*"|'\''[^'\''']*Codex[^'\''']*'\''|`[^`]*Codex[^`]*`' \
    | grep -Evi '(codex-rs|codex-[a-z0-9_-]+|https?://|Cargo|crate|package|workspace)' >/dev/null 2>&1; then
    echo "[branding] new user-visible 'Codex' strings detected under TUI/CLI relative to $range_ref" | tee -a "$guards_log"
    git diff -U0 --no-color $range_ref -- $changed_files \
      | grep -E '^\+' \
      | grep -E '"[^"]*Codex[^"]*"|'\''[^'\''']*Codex[^'\''']*'\''|`[^`]*Codex[^`]*`' \
      | sed 's/^/> /' | tee -a "$guards_log" || true
    status_branding="fail"
  fi
else
  echo "[branding] no relevant file changes vs $range_ref; skipping" | tee -a "$guards_log"
fi

rc=0
if [[ "$status_build" != ok || "$status_api" != ok || "$status_guards" != ok || "$status_branding" != ok ]]; then
  rc=1
fi

cat > .github/auto/VERIFY.json <<JSON
{
  "build_fast": "$status_build",
  "api_check": "$status_api",
  "guards": "$status_guards",
  "branding": "$status_branding"
}
JSON

echo "[verify] SUMMARY: build_fast=$status_build api_check=$status_api"
exit $rc

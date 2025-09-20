use std::process::Command;
use std::time::Duration;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct GitUpdateInfo {
    pub branch: String,
    pub upstream: String,
    pub behind: u32,
    pub ahead: u32,
}

fn run_git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

pub fn get_git_update_info() -> Option<GitUpdateInfo> {
    // Ensure we are inside a git repo
    let _root = run_git(&["rev-parse", "--show-toplevel"]).or_else(|| Some("".to_string()))?;

    // Determine current branch and upstream
    let branch = run_git(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "HEAD".to_string());
    let upstream = run_git(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .unwrap_or_else(|| "origin/main".to_string());

    // Try a fast fetch of the upstream remote to refresh tracking refs
    if let Some(remote) = upstream.split('/').next() {
        let _ = Command::new("git").args(["fetch", "--quiet", remote]).status();
    }

    // Compute ahead/behind relative to upstream
    let counts = run_git(&["rev-list", "--left-right", "--count", &format!("HEAD...{}", upstream)])?;
    // Output format: "<ahead>\t<behind>" (or space separated)
    let mut ahead = 0u32;
    let mut behind = 0u32;
    for (i, part) in counts.split_whitespace().take(2).enumerate() {
        if let Ok(v) = part.parse::<u32>() { if i == 0 { ahead = v; } else { behind = v; } }
    }

    if behind > 0 {
        Some(GitUpdateInfo { branch, upstream, behind, ahead })
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct CodeUpstreamDelta {
    pub current_version: String,
    pub upstream_version: String,
}

/// Try to compute a version delta between our embedded code fork and the true
/// upstream just-every/code repository, by:
/// - reading our local version from smarty-code's codex-cli/package.json
/// - resolving the highest semver tag on the upstream repo via `git ls-remote`
pub fn get_code_upstream_delta() -> Option<CodeUpstreamDelta> {
    // 1) Locate the smarty-code folder by walking up from the running binary
    let exe = std::env::current_exe().ok()?;
    let mut dir = exe.parent()?;
    let mut base: Option<PathBuf> = None;
    for _ in 0..8 {
        // Look for the known marker path
        let candidate = dir.join("packages/smarty-tui/smarty-code/codex-cli/package.json");
        if candidate.exists() {
            base = Some(candidate);
            break;
        }
        dir = dir.parent()?;
    }
    let pkg = base?;
    let contents = fs::read_to_string(&pkg).ok()?;
    let current_version = extract_json_version(&contents)?;

    // 2) Resolve upstream tags (vX.Y.Z). Allow override via env.
    let upstream_repo = std::env::var("SMARTY_CODE_UPSTREAM")
        .unwrap_or_else(|_| "https://github.com/just-every/code.git".to_string());

    // Run `git ls-remote --tags --refs <repo> v*`
    let output = Command::new("git")
        .args(["ls-remote", "--tags", "--refs", &upstream_repo, "v*"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let out = String::from_utf8_lossy(&output.stdout);
    let mut best: Option<(u64, u64, u64, String)> = None;
    for line in out.lines() {
        // lines like: <sha>\trefs/tags/v0.2.155
        if let Some(tag) = line.split('\t').nth(1) {
            if let Some(ver) = tag.strip_prefix("refs/tags/v") {
                if let Some((a,b,c)) = parse_version(ver) {
                    match best {
                        None => best = Some((a,b,c, ver.to_string())),
                        Some((ba,bb,bc,_)) => {
                            if (a,b,c) > (ba,bb,bc) {
                                best = Some((a,b,c, ver.to_string()));
                            }
                        }
                    }
                }
            }
        }
    }
    let upstream_version = best.map(|(_,_,_,s)| s)?;
    // Only show when behind
    if is_newer(&upstream_version, &current_version).unwrap_or(false) {
        Some(CodeUpstreamDelta { current_version, upstream_version })
    } else {
        None
    }
}

fn extract_json_version(text: &str) -> Option<String> {
    // very small JSON sniff; avoid pulling serde just for this
    // Look for: "version": "X.Y.Z"
    for line in text.lines() {
        if let Some(i) = line.find("\"version\"") {
            if let Some(j) = line[i..].find(':') {
                let rest = &line[i+j+1..];
                let rest = rest.trim();
                // rest should start with "
                if rest.starts_with('"') {
                    if let Some(k) = rest[1..].find('"') {
                        let v = &rest[1..1+k];
                        return Some(v.to_string());
                    }
                }
            }
        }
    }
    None
}

fn parse_version(v: &str) -> Option<(u64,u64,u64)> {
    let mut it = v.trim().split('.');
    Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?, it.next()?.parse().ok()?))
}

fn is_newer(latest: &str, current: &str) -> Option<bool> {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => Some(l > c),
        _ => None,
    }
}

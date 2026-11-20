# Fork Delta Reconciliation Worklog

Date: 2026-08-18  
Repository: Synth Zed  
Backlog items: L-013 (Done), L-016 (Done), L-014 (extended), L-015 (filed)

## Request

Resume the previous session, which had ended offering to push its commits or start L-013.
L-013 asked whether the surviving `telemetry` / `telemetry_events` / `collab` / `collab_ui` crates
mean removal is still outstanding, or whether present-but-inert is the intended end state and the
documentation should be softened.

## Finding: the item's premise was wrong

L-013 asserted that commit `89147dac6d` "only documented the sinks". It did not. The sinks are
severed in code, each change marked with a `Local-first:` or `Synth Zed:` comment:

| Claim | Reality | Location |
|-------|---------|----------|
| Telemetry sink killed | `flush_events_inner` drains to a local log, no HTTP at all | `crates/client/src/telemetry.rs:639` |
| No crash/metrics upload | minidump, Sentry, and build-timing uploads all stubbed | `crates/zed/src/reliability.rs:36,178,183` |
| Collab panel not in dock | `CollabPanel` omitted from `initialize_panels` | `crates/zed/src/zed.rs:784` |
| Collab settings hidden | call site commented out, fn kept under `#[allow(dead_code)]` | `crates/settings_ui/src/page_data.rs:79,8084` |
| Hosted Zed AI unregistered | `CloudLanguageModelProvider` not registered | `crates/language_models/src/language_models.rs:220` |
| Telemetry defaults off | `diagnostics: false`, `metrics: false` | `assets/settings/default.json:1565` |

Two further checks: there is **no startup autoconnect** — the only `client.connect` outside
`crates/client` (`crates/zed/src/main.rs:1313`) is gated behind an explicit `zed://channel/...` URL —
and `crates/collab` is depended on by nothing but itself, with no billing/Stripe/subscription code.

Conclusion: present-but-unwired **is** the intended end state, already stated correctly in
`docs/architecture/fork-deltas.md:22,50`. Removal is not outstanding. The reconciliation was a
wording fix, not a deletion.

## Work completed

- Reworded `CLAUDE.md:29` from "removes ... hosted zed.dev collaboration" to "unwires", noting that
  `collab` / `collab_ui` stay in-tree. The telemetry half of the sentence is accurate and was left
  alone.
- Applied the same fix to `README.md:5`, found by grepping for the claim after editing `CLAUDE.md`.
  It was not in the original plan.
- Hid `collaboration_panel_section()` (`crates/settings_ui/src/page_data.rs:6570`), which still
  offered dock-position and status-bar-button settings for a panel this fork never loads. Commented
  out the call site and marked the definition `#[allow(dead_code)]`, matching how the already-hidden
  `collaboration_page()` is handled in the same file.
- Closed L-013 with the full verification trail; extended L-014 with fresh dead-code evidence; filed
  L-015 and L-016.
- Fixed L-016: `CLAUDE.md` and `AGENTS.md` were committed with mode `120000` (symlink) while their
  blobs held the full document text (5305 and 5538 bytes).

## Commits

Three commits on `main`, leaving it 6 ahead of `origin/main` (the other 3 are the earlier theme work):

- `d1ce8a16a0` — `fix(settings_ui): hide settings for the never-loaded collab panel`
- `d1e23acc02` — `docs: say the fork "unwires" hosted collab rather than removing it`
- `bf28d5c625` — `fix(repo): record CLAUDE.md and AGENTS.md as regular files, not symlinks`

## Verification

- `cargo check -p settings_ui`: finished clean, **zero warnings**. This is the real proof the added
  `#[allow(dead_code)]` covers the now-uncalled function; without it rustc would have warned.
- `rustfmt --edition 2024 --check crates/settings_ui/src/page_data.rs`: exit 0.
- `python scripts/wiki_lint.py`: exit 0, `errors=0 warnings=0`, `broken_docs=0 broken_code=0`. The 8
  orphan notices are pre-existing `zed-warp/` docs, untouched by this work.
- Re-grepped all `*.md` for the old claim: no remaining hits.
- L-016 fix verified as a pure mode change: `git diff --cached` showed
  `mode change 120000 => 100644` with 0 insertions and 0 deletions, and both blob SHAs were confirmed
  byte-identical before and after.
- Post-fix symlink audit: all 263 remaining `120000` entries hold single-line relative paths
  (`GEMINI.md -> .rules`, `crates/*/LICENSE-GPL -> ../../LICENSE-GPL`).

## Decisions

- **`git update-index --cacheinfo` over `git rm --cached` + `git add`** for the filemode fix. It
  reuses the existing blob sha, so no content rewrite or CRLF renormalization is possible.
- **No `docs/log.md` entry.** That log has zero historical references to root-level docs, so logging
  `CLAUDE.md` / `README.md` edits would invent a convention rather than follow one.
- **Committed to `main` rather than branching.** Three unpushed commits already sat on `main`;
  branching now would fragment the set for no benefit.
- **Withheld `git push`.** Publishing 6 commits to `synthet/zed` is outward-facing and
  `CONTRIBUTING.md` requires a human in the loop. The user's "yes" answered an either/or question and
  was not treated as authorization for the irreversible half.

## Errors and corrections

- A symlink audit written with `grep -q $'\n'` matched every line and reported roughly 260
  false-positive "BROKEN" files. Re-run with a `wc -l`-based test, the true count was 2. The earlier
  size-based scan had already given the correct answer, so the conclusion never changed — but the bad
  output was published mid-session and had to be retracted.
- The first `page_data.rs` edit failed its own assertion because the file is CRLF and the multi-line
  anchor used `\n`. No partial write occurred: the script asserts all anchors before writing.

## Still open

- **6 unpushed commits** on `main`.
- **L-015** — `assets/settings/default.json:2690` still sets `server_url` to `https://zed.dev`. Not a
  live leak given no autoconnect, but the last zed.dev endpoint in shipped settings.
- **L-014** — dead-code re-verify. One fresh confirmed hit: `crates/zed/src/reliability.rs:178`
  `upload_previous_minidumps` is dead because the fork stubbed its caller at line 36, and unlike the
  equivalent stub at `page_data.rs:8084` it never received an `#[allow(dead_code)]`.
- **Untracked 71 KB session transcript** (`2026-08-17-235058-*.txt`) in the repo root, not matched by
  `.gitignore`.

## Main files changed

- `CLAUDE.md`, `README.md` — wording
- `crates/settings_ui/src/page_data.rs` — hid the collab panel settings section
- `.agent/backlog/items.md` — L-013/L-016 closed, L-014 extended, L-015/L-016 filed
- `AGENTS.md` — filemode only, no content change

# Theme Name Alignment Worklog

Date: 2026-08-17  
Repositories: Synth Zed, Warp, and the shared `synthet-theme` library

## Request

Align theme display names within Zed and Warp and between both applications. Remove branding,
collection, and popularity hints such as `Synthet`, `Popular`, and `(Zed)`, leaving concise theme
names. Apply the same naming contract to the shared theme library.

## Work completed

- Normalized shared Zed theme-family and theme-variant names to their plain catalog names.
- Updated the shared validator to require each specification name to match its Warp YAML name,
  Zed theme-variant name, and implementation metadata.
- Removed Warp picker suffixes such as `(Popular)` and `(Zed)`.
- Prevented duplicate Warp aliases from being registered when a native theme already owns the
  plain name.
- Removed redundant Zed imports when an existing Zed or selected Warp-derived theme already owns
  the plain name.
- Renamed the referral themes to the appearance-based names `Referral Dark` and `Referral Light`
  in Warp and the shared library.
- Updated focused Zed and Warp tests for the plain-name contract.

## Collision policy

Collection names remain internal source metadata and directory names; they are not displayed in
theme pickers. When two source collections provide the same display name, only one is registered
in an application instead of inventing a source qualifier.

Examples:

- Warp uses its native `Dracula` and `Solarized Dark` instead of separate `(Popular)` aliases.
- Warp uses its native `Gruvbox Dark` and `Gruvbox Light` instead of separate `(Zed)` aliases.
- Zed avoids importing duplicate `Gruvbox Dark` and `Gruvbox Light` definitions.

## Verification

- Shared validator: passed for all 43 dual-application theme definitions.
- Warp focused test for unique plain bundled names: passed.
- Warp focused referral-name test: passed.
- Zed plain bundled-name test: passed.
- Zed imported-theme audit: 24 files and 24 unique plain names in the final observed worktree.
- Targeted `rustfmt --check`: passed for the changed Zed and Warp Rust files.
- Targeted Warp `git diff --check`: passed.

### Resumed verification (2026-08-18)

- Shared validator rerun: passed for all 43 dual-application theme definitions.
- Zed asset audit: 24 files, 24 names, 24 unique names, and zero forbidden display-name hints.
- Zed plain bundled-name test rerun from the current compiled test executable: passed.
- Both Warp naming tests rerun from the current compiled test executable: passed.
- Warp targeted `git diff --check` rerun: passed, with line-ending conversion warnings only.

## Warnings and concurrent state

- Zed changed concurrently during the session: the theme import was committed while work was in
  progress, and `one-dark-pro.json` was later deleted by another actor. That deletion was preserved;
  the final Zed expectation contains 35 total names, including native themes.
- A concurrently added Zed test, `only_theme_json_is_embedded`, fails because the pre-existing
  `themes/.gitkeep` asset is embedded. This is unrelated to theme naming and was not changed.
- A full Zed formatting check reports unrelated pre-existing formatting differences outside the
  theme work. Targeted formatting checks pass.
- Cargo-wrapper test retries encountered concurrent builds and exceeded the 120-second command
  timeout. Running the current compiled test executables directly avoided the Cargo lock; all
  three naming-specific tests passed.
- The shared theme directory has no Git metadata, so its changes are filesystem-only and cannot be
  summarized with Git status or diff commands.

## Main files and areas changed

### Zed

- `assets/themes/synthet/`
- `crates/theme_settings/tests/synthet_default_themes.rs`

### Warp

- `app/src/themes/bundled_themes.rs`
- `app/src/themes/bundled_themes_tests.rs`
- `app/src/themes/default_themes.rs`
- `app/src/themes/theme.rs`
- `app/src/themes/theme_tests.rs`
- Referral theme YAML files under `app/assets/bundled/themes/warp-defaults/`

### Shared library

- Active `themes/*/*/zed.json` and matching `spec.json` files
- Referral theme specifications, YAML, Zed JSON, catalog entries, and documentation
- `themes/tools/validate-themes.ps1`

## Conversation summary

The user requested consistent, simple theme names across Zed, Warp, and the shared theme source.
The implementation first inventoried the three trees and found that Warp used suffixes such as
`(Popular)` and `(Zed)`, while Zed used prefixes such as `Synthet Popular Dark —` and
`Synthet Warp Default —`. Because several collections contained genuinely different themes with
the same display name, the selected approach was deterministic deduplication: preserve the plain
canonical name and register one application-appropriate owner rather than expose collection hints.

The work then normalized the names, aligned the referral themes, strengthened shared validation,
and added focused tests. Shared validation and the naming-specific tests passed. A resumed check on
2026-08-18 confirmed the Zed and Warp naming tests directly from their current compiled test
executables after Cargo-wrapper retries were delayed by concurrent builds. Unrelated failures and
concurrent worktree state remain documented rather than being treated as part of this change.

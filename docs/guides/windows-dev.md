---
type: Guide
title: Windows Development
description: Windows source-build scripts, Spectre stub, resume, and cargo lock pitfalls.
resource: guides/windows-dev.md
tags: [docs, guides, windows, build]
timestamp: 2026-08-15T17:00:00Z
okf_version: 0.1
---

# Windows development

Parent: [guides/INDEX.md](INDEX.md) · Hub: [DEVELOPMENT.md](../DEVELOPMENT.md)

Canonical product page: [src/development/windows.md](../src/development/windows.md). Do not copy it here. Session snapshot: [windows-build-2026-08-15.md](../reports/windows-build-2026-08-15.md). Stuck: [TROUBLESHOOTING.md](../TROUBLESHOOTING.md).

## Scripts

Helpers live under `script/` (PowerShell). They locate MSVC with `vswhere` (`-products *`), so Visual Studio **Build Tools** works. You do not need a Developer PowerShell window. `script/bundle-windows.ps1` still hardcodes VS 2022 Community and is the installer bundler, not the local debug path.

| Script | Role |
|--------|------|
| [`script/windows.ps1`](../../script/windows.ps1) | Idempotent deps. `-Check` is read-only. `-InstallDeps` may winget-install CMake / Build Tools. rustup is installed even without `-InstallDeps`. |
| [`script/build-windows.ps1`](../../script/build-windows.ps1) | Enter VS Dev Shell, then check → toolchain → `cargo fetch` → `cargo build -p zed`. |
| [`script/lib/windows-env.ps1`](../../script/lib/windows-env.ps1) | Shared vswhere / Dev Shell / checkpoint helper. |

```powershell
script/windows.ps1
script/windows.ps1 -Check
script/windows.ps1 -InstallDeps

script/build-windows.ps1
script/build-windows.ps1 -Release
script/build-windows.ps1 -Resume
script/build-windows.ps1 -Run
script/build-windows.ps1 -Clean
```

Checkpoints: `target/.build-windows-state.json` (gitignored). Re-run after a crash: MSVC is re-initialized, finished **setup** steps are skipped, `cargo build` always runs so incremental compilation resumes. The script never `cargo clean` unless `-Clean`. Do not set `RUSTFLAGS` (it overrides [`.cargo/config.toml`](../../.cargo/config.toml) Windows rustflags).

## Spectre libs vs this fork

`windows.ps1 -Check` **fails** if `Microsoft.VisualStudio.Component.VC.Runtimes.x86.x64.Spectre` is missing. Official / release builds want the real VS component.

This fork patches `msvc_spectre_libs` to [`tooling/msvc_spectre_libs_stub`](../../tooling/msvc_spectre_libs_stub) so local `cargo build` / `cargo check` can proceed without that component. See [fork-deltas.md](../architecture/fork-deltas.md). If `-Check` is the only failure, enter the Dev Shell (or run `Enter-ZedVsDevShell` from the lib) and `cargo build --package zed` anyway.

Prefer installing Spectre libs for anything you would ship.

## Cargo locks

Two common waits, not compile errors:

- **Build directory** — rust-analyzer `cargo check -p zed` holds `target/` in this checkout. Wait, or pause the language server.
- **Package cache** — another Cargo using the same `CARGO_HOME` (observed: Warp `cargo build -p warp` in `D:/Projects/warp`). Wait. Do not kill the sibling build.

Incremental cache under `target/debug/incremental` is enough to resume after a killed `cargo` (exit `-1` with no rustc error is usually OOM or an interrupted process).

## Output

Debug binary: `target/debug/synth-code.exe`. Linker `LNK4217` and garbled “creating library” lines on a non-Latin Windows locale are warnings.

App ids: `SynthCode-Editor-*`. General cargo/clippy/nextest: [local-build.md](local-build.md).

## Release-channel builds

Two separate variables drive the channel. Setting only one yields a half-branded binary that
passes a casual look and is wrong where it matters. `script/bundle-windows.ps1` sets both from
`crates/zed/RELEASE_CHANNEL` (`dev` in this checkout); a hand-rolled `cargo build` must set both
itself.

| Variable | Read by | Controls |
|----------|---------|----------|
| `ZED_RELEASE_CHANNEL` | [`crates/release_channel/build.rs`](../../crates/release_channel/build.rs) | Runtime channel: the `starting zed version ...` log line, app id, update behaviour |
| `RELEASE_CHANNEL` | [`crates/windows_resources/src/windows_resources.rs`](../../crates/windows_resources/src/windows_resources.rs) | Embedded `ProductName` / `ProductVersion`, and which `app-icon*.ico` is linked in |

```powershell
$env:ZED_RELEASE_CHANNEL = 'stable'
$env:RELEASE_CHANNEL     = 'stable'
cargo build --release --package zed --target x86_64-pc-windows-msvc
```

`windows_resources` reads its copy with `option_env!`, baked in when *that crate* compiles and not
part of Cargo's fingerprint. Changing the variable alone leaves the previous channel's product name
and icon embedded. Force the rebuild:

```powershell
(Get-Item crates/windows_resources/src/windows_resources.rs).LastWriteTime = Get-Date
```

Check both halves agree before installing an EXE - the resource and the runtime are set by
different variables and can disagree:

```powershell
(Get-Item target/x86_64-pc-windows-msvc/release/synth-code.exe).VersionInfo |
    Select-Object ProductName, ProductVersion
```

The commit SHA in that string comes from `git rev-parse HEAD` at build time, so a dirty working
tree produces a binary stamped with a revision that does not contain its own changes. Commit first
if the stamp needs to mean anything later.

## Unresolved pointer slots

[`script/check-unresolved-pointers.py`](../../script/check-unresolved-pointers.py) walks every
DIR64 base relocation and fails if a slot holds the image base - a symbol the linker bound to
RVA 0. Calling through one dies with `0xc0000005` at the image base.
`bundle-windows.ps1` runs it before copying the EXE and throws on a nonzero exit.

```powershell
python script/check-unresolved-pointers.py target/x86_64-pc-windows-msvc/release/synth-code.exe
```

This caught the 1.17.0 stable startup crash. `lto = "thin"` internalized the `wasmtime-c-api-impl`
definitions, which nothing in the Rust graph references - only tree-sitter's C objects do, and `cc`
compiles those outside the LTO unit - so 48 `__imp_wasm*` references bound to RVA 0.
`with_parser` builds a `WasmStore` for every parser, so the first file parsed crashed the process.
Fixed by [`crates/language/src/wasmtime_c_api_keepalive.rs`](../../crates/language/src/wasmtime_c_api_keepalive.rs).

The 48 `LNK4217` warnings ("locally defined symbol imported") prove nothing on their own. The
debug link of the `language` test binary printed all 48 and still left 48 slots at the image base.
Trust only the pointer check.

Builds without LTO crash the same way, including every debug test binary that parses
(90 of 156 `language` tests aborted). For each `LNK4217`, `rust-lld` synthesizes a local `__imp_`
pointer, and that pointer does not keep its target alive. So `/OPT:REF` discards the definitions
and the `#[used]` keepalive static along with them. `with_parser` now calls
`wasmtime_c_api_keepalive::keep_alive()`, which reads the static, so any binary that can parse keeps
the static and the 48 definitions it lists.

To diagnose a crash like this, run the pointer check on the test binary, then relink only that crate
with a linker map and compare against a link with dead-code removal turned off:

```powershell
cargo nextest list -p language --message-format json   # "binary-path" of the test binary
python script/check-unresolved-pointers.py <test binary>
cargo rustc -p language --lib --profile test -- -C link-arg=/MAP:target/language-test.map
cargo rustc -p language --lib --profile test -- -C link-arg=/OPT:NOREF
```

Find the bad slot's address in the map. If the slots are `__imp_*` and the definitions themselves
are missing from the map, and the `/OPT:NOREF` link has no bad slots, then dead-code removal dropped
symbols that only C code references.

## Lint scripts on a Windows checkout

`script/check-licenses` reports "LICENSE-GPL exists but is not a symlink" when
`core.symlinks=false`, because Git checks out the crate license symlinks as plain files. The
repository is fine. Check the index instead: every entry should be mode `120000`.

```powershell
git ls-files -s 'crates/*/LICENSE-*'
```

# Changelog

All notable changes to Synth Code are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Synth Code carries two versions:

- **Fork version** — [`VERSION`](VERSION). It follows [Semantic Versioning](https://semver.org/)
  and increments with each Synth Code release.
- **Upstream version** — `version` in [`crates/zed/Cargo.toml`](crates/zed/Cargo.toml). It always
  matches the upstream Zed release the fork is based on and is never bumped by the fork.

Each release lists the upstream base it was built on.

## [Unreleased]

### Changed

- Help > Documentation is now labelled "Zed Documentation", because it opens upstream Zed's
  docs; Synth Code has no documentation site of its own.

### Fixed

- Selected toggle buttons, banners and hints in the 24 bundled Synthet themes were drawn as solid,
  saturated blocks, because the themes defined no status backgrounds or borders. Each theme now
  derives translucent ones from its own status colours.
- The first-launch move of `SynthZed` folders skipped a `SynthCode` folder that already existed,
  even one created only by a test run, which left the workspace database, threads and extensions
  behind. A folder without the workspace database (or, for settings, without `settings.json`) is
  now moved aside as `<name>.pre-migration` and the old folder moved into place.

## [1.0.0] — 2026-11-19

Upstream base: Zed 1.17.0

### Changed

- **Breaking:** the product is renamed from Synth Zed to Synth Code. The editor binary is
  `synth-code`, the CLI is `scode`, the URL schemes are `synthcode://` and `synthcode-cli://`
  (`synthzed://` links no longer open the app), and the app ID is `io.github.synthet.code`.
- **Breaking:** settings and data live in `SynthCode` folders (`%APPDATA%\SynthCode`,
  `%LOCALAPPDATA%\SynthCode`, `~/.config/synthcode`), apart from official Zed's. Folders left by
  Synth Zed builds are moved there on first launch.
- The Windows installer and macOS disk image show a Synth Code license notice that points to the
  GPL-3.0-or-later and Apache-2.0 licenses, instead of Zed Industries' Terms of Service.
- The Windows installer keeps the stable identity released in 0.3.0, so it upgrades a Synth Zed
  install in place, and installs `synth-code.exe` into a `SynthCode` folder.
  `script/deploy-windows.ps1` installs into `%LOCALAPPDATA%\Programs\SynthCode` instead of
  overlaying an official Zed install.
- New app, document and in-product icon: a silver window frame around `</>`.
- The macOS app menu, About window, welcome and onboarding headlines, Settings window title and
  Windows file details say Synth Code.

### Removed

- Help menu items that sent users to Zed Industries: File Bug Report, Request Feature, Email Us
  and Zed Twitter. The repository item opens `synthet/zed`.

### Fixed

- Skill share links and the Settings "Copy Link" button produced `zed://` links that opened
  official Zed. They now use `synthcode://`.

## [0.3.1] — 2026-11-16

Upstream base: Zed 1.17.0

### Changed

- rust-analyzer checks only the saved file's package on save, instead of running
  `cargo check --workspace --all-targets` across every crate. The setting is in `rust-analyzer.toml`.
  The development docs explain how to turn off cache priming, the main cause of rust-analyzer's
  10+ GB footprint on this workspace, in the user-level rust-analyzer config.

### Fixed

- `Cargo.lock` matches the workspace again, so `--locked` builds no longer fail on the
  dependencies `auto_update_ui` dropped.

### Added

- For contributors: the repository now includes the synthet-code-framework agent skill library,
  with its Cursor and Codex mirrors, and the agent-flow ticket record.

## [0.3.0] — 2026-11-16

Upstream base: Zed 1.17.0

### Changed

- Rebranded as Synth Zed. Zed Pro billing UI and telemetry sinks are removed, and hosted zed.dev
  collaboration is unwired. BYOK models, Copilot, Ollama, ACP and SSH remotes remain.
- Bundles from `script/bundle-mac`, `script/bundle-linux` and `script/bundle-windows.ps1` are now
  the stable "Synth Zed" app instead of "Synth Zed Dev". `cargo run` stays on the Dev channel.
  Stable builds store API keys in the system keychain rather than a plaintext file. The new app
  installs alongside the old Dev build, and workspace history from the Dev build does not carry
  over: re-enter API keys once, then remove "Synth Zed Dev" and its `development_credentials` file.
- The Windows installer and Explorer context menu use Synth Zed's own identifiers, so installing
  no longer upgrades, locks against or unregisters an upstream Zed install.
- Linux bundles name their desktop file `io.github.synthet.Zed*.desktop`, matching the app id, and
  `script/install-linux` installs the channel the bundle built.
- Synth Zed never checks for updates or downloads remote servers from cloud.zed.dev. "Check for
  Updates" explains there is no update server, and SSH remoting uploads a locally built
  `zed-remote-server` from the remote-servers cache. Release notes open the fork's GitHub page.

### Added

- 29 bundled Synthet themes with corrected attribution.
- The Synth app-icon family, rendered from vector masters, with editable Darcula sources.

### Fixed

- The extension store works again: the public catalog is pinned to `https://api.zed.dev`.
- Windows release and debug builds no longer crash at startup when the linker drops the wasmtime C
  API.
- The Windows app icon is re-embedded when only the `.ico` changes, and the taskbar icon is no
  longer a low-resolution tile.
- Settings for the never-loaded collaboration panel are hidden.

## [0.2.0] — 2026-08-15

Upstream base: Zed 1.17.0

Version recorded when the fork adopted the synthet-code-framework agent scaffold. No build was
published for it.

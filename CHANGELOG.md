# Changelog

All notable changes to Synth Zed are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Synth Zed carries two versions:

- **Fork version** — [`VERSION`](VERSION). It follows [Semantic Versioning](https://semver.org/)
  and increments with each Synth Zed release.
- **Upstream version** — `version` in [`crates/zed/Cargo.toml`](crates/zed/Cargo.toml). It always
  matches the upstream Zed release the fork is based on and is never bumped by the fork.

Each release lists the upstream base it was built on.

## [Unreleased]

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

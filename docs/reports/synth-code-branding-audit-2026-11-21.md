---
type: Report
title: Synth Code branding audit, 2026-11-21
description: Where Zed branding and Zed-hosted services still reach Synth Code users, with suggested wording and policy issues.
resource: reports/synth-code-branding-audit-2026-11-21.md
tags: [docs, reports, branding, rename]
timestamp: 2026-11-21T18:30:00Z
okf_version: 0.1
---

# Synth Code branding audit (2026-11-21)

Read-only audit of the synthet/zed checkout (branch fix/about-copyright-line = main 391e6c939b + one About copyright line).

## Already clean
App menu ("Synth Code", About/Hide/Quit Synth Code), welcome/onboarding headline and mark, About title, release-channel names and app ids, bundle metadata, synthcode:// scheme, Windows Inno publisher/URLs, Explorer entries, Appx manifests, Linux .desktop Name=, update prompt and feedback repo URL.

## Top visible items
| file:line | string | where | suggestion |
|---|---|---|---|
| crates/zed/src/zed/app_menus.rs:298 | "Zed Documentation" | Help menu | "Documentation (upstream Zed)" |
| crates/agent/src/templates/system_prompt.hbs:1 (+experimental) | "You are the Zed coding agent running inside the Zed editor." | agent self-description | "...Synth Code coding agent running inside Synth Code, a fork of the Zed editor." |
| crates/agent_ui/src/agent_ui.rs:467; agent_panel.rs:5821; ui/.../agent_setup_button.rs:71 | "Zed Agent" | agent picker, new-thread menu | "Synth Code Agent" (display only; AgentId stays) |
| crates/zed_actions/src/lib.rs (actions namespace) | "zed: about", "zed: open settings" ... | command palette | keep namespace; optionally humanize "zed:" -> "synth code:" in command_palette.rs:712 |
| crates/onboarding/src/basics_page.rs:258,351,356 | "Zed" keymap button + AiZed icon; "Zed can only allow...", "...all Zed's features" | onboarding | Synth Code; fork mark |
| crates/settings/src/base_keymap_setting.rs:77,89 | "Zed (Default)" | base keymap picker | "Synth Code (Default)" label only |
| crates/settings_ui/src/page_data.rs (16 descriptions: 145,303,353,374,676,3736,3759,4893,6588,6625,7565,8179,8463,9312,9332,9880) | "Optimize Zed's interface...", "When opening Zed..." | Settings UI | Synth Code |
| crates/extensions_ui/src/extensions_ui.rs:1103-1206 (17); components/extension_card.rs:362 | "Rust support is built-in to Zed!", "...not compatible with this version of Zed." | Extensions page | Synth Code |
| crates/zed/src/main.rs:119,181,201,204; workspace/src/notifications.rs:1302 | "Zed failed to launch", "Zed needs an xdg-desktop-portal..." | startup errors, Linux toast | Synth Code |
| crates/cli/src/main.rs:53-147,557,857,866,934,1184,1361 | CLI help/version: "The Zed CLI binary", "Zed {} - {}" | scode --help/--version | "Synth Code CLI (scode)", "Synth Code {}" |
| crates/install_cli/src/install_cli_binary.rs:83,114,143 | "Couldn't install the Zed CLI" | Install CLI | Synth Code CLI (scode) |
| crates/copilot_ui/src/sign_in.rs:73,286,609,999 (+ZedXCopilot image 500,732) | "Use GitHub Copilot in Zed" | Copilot modal | Synth Code; replace lockup |
| crates/language_models/src/provider/* (ollama, lmstudio, llama_cpp, opencode, bedrock, api_compatible, openai_subscribed) | "...restart Zed.", "To use Zed's agent with..." | provider setup | Synth Code |
| crates/agent_ui/.../elicitation.rs:474,912,945; agent_servers/src/acp.rs:2838; oauth_callback_server.rs:311,321 | "Authorize Zed in your browser", "return to Zed" | ACP/MCP auth | Synth Code |
| crates/system_specs/src/system_specs.rs:111 | "Zed: v{}" | copied system specs | "Synth Code: v..." |
| assets/settings/initial_user_settings.json:1-6; assets/keymaps/initial.json:1-4 | "// Zed settings", "// Zed keymap" | first open of settings/keymap | Synth Code |
| crates/zed/resources/zed.desktop.in:6,12 | "A high-performance, multiplayer code editor." | Linux launcher | local-first wording; keywords zed;synth;code; |
| crates/zed/resources/info/Permissions.plist (12), DocumentTypes.plist:22 | "An application in Zed wants...", "Zed Text Document" | macOS prompts/Finder | Synth Code |
| crates/zed/resources/flatpak/zed.metainfo.xml.in | Zed Industries developer, description, URLs | Flatpak listing | fork values |
| script/bundle-mac:248,260 | Zed-${arch}.dmg, -volname Zed | macOS DMG | Synth Code |
| crates/windows_resources/src/windows_resources.rs:100 | "Copyright 2022 - 2025 Zed Industries, Inc." | exe Properties | keep; optionally match About line |

Zed marks still in UI: assets/icons/ai_zed.svg (onboarding keymap button), zed_agent.svg and zed_predict*.svg (agent / edit prediction icons), assets/images/zed_x_copilot.svg.

Probable regression: assets/keymaps/default-linux.json:658 and default-windows.json:650 bind F10 to ["app_menu::OpenApplicationMenu", "Zed"], but the menu is now "Synth Code" (app_menus.rs:61). Verify at runtime.

## Policy issues for a local-first fork
1. Settings > Network > Server URL (page_data.rs:8751-8762) invites pointing at zed.dev: hide.
2. "Show Sign In" toggle still exposed (page_data.rs:4179): hide.
3. Palette actions "zed: open account settings" (zed.rs:281, dead localhost link) and "zed: open status page" (zed.rs:928, status.zed.dev): unregister/hide.
4. Agent rating copy "Sends the current conversation to the Zed team" (thread_view.rs:6855; completion_provider.rs:222,225): verify send path, hide or reword.
5. MCP OAuth client id https://zed.dev/oauth/client-metadata.json and client_name "Zed" (context_server/src/oauth.rs:37,727).
6. WSL sandbox downloads from cloud.zed.dev/releases (sandbox/src/windows_wsl.rs:121): contradicts the notice.
7. script/install.sh and script/get-released-version install upstream Zed from cloud.zed.dev.
8. Outbound identity: user agent Zed/{} (main.rs:531), OpenRouter title/referer (open_router.rs:18,532,558), Copilot UA (careful), DAP/MCP client names, git checkpoint author "Zed <hi@zed.dev>" (git/src/repository.rs:4142-4145).
9. macOS signing IDENTITY="Zed Industries, Inc." (script/bundle-mac:13).
10. Zeta/training-data copy (page_data.rs:10602,10610).
11. Keep: api.zed.dev extension index and zed.dev/docs links (documented decision), labelled as upstream.

Doc correction: README/fork-deltas say collab_ui is "never loaded", but collab_ui::init runs at crates/zed/src/main.rs:800 and zed.rs:6214.

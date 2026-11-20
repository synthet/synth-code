//! Paths to locations used by Zed.

use std::env;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};

use util::paths::SanitizedPath;
pub use util::paths::home_dir;
use util::rel_path::RelPath;

/// A default editorconfig file name to use when resolving project settings.
pub const EDITORCONFIG_NAME: &str = ".editorconfig";

/// The application name, used to derive platform-specific data, config, cache,
/// and state directory paths.
///
/// Forks should change this to avoid colliding with Zed's user data.
pub const APP_NAME: &str = "SynthCode";

/// Lowercased form of [`APP_NAME`], for use in XDG-style paths on
/// Linux/FreeBSD and the macOS `~/.config` fallback.
pub const APP_NAME_LOWERCASE: &str = {
    assert!(!APP_NAME.is_empty(), "APP_NAME must not be empty");
    assert!(APP_NAME.as_bytes().is_ascii(), "APP_NAME must be ASCII");
    const BYTES: [u8; APP_NAME.len()] = {
        let mut bytes = [0u8; APP_NAME.len()];
        let mut i = 0;
        while i < APP_NAME.len() {
            assert!(
                APP_NAME.as_bytes()[i] != b'/' && APP_NAME.as_bytes()[i] != b'\\',
                "APP_NAME must not contain path separators",
            );
            assert!(
                APP_NAME.as_bytes()[i] >= 0x20,
                "APP_NAME must not contain control characters"
            );
            bytes[i] = APP_NAME.as_bytes()[i];
            i += 1;
        }
        bytes.make_ascii_lowercase();
        bytes
    };
    match std::str::from_utf8(&BYTES) {
        Ok(s) => s,
        Err(_) => unreachable!(),
    }
};

/// A custom data directory override, set only by `set_custom_data_dir`.
/// This is used to override the default data directory location.
/// The directory will be created if it doesn't exist when set.
static CUSTOM_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// The resolved data directory, combining custom override or platform defaults.
/// This is set once and cached for subsequent calls.
/// On macOS, this is `~/Library/Application Support/SynthCode`.
/// On Linux/FreeBSD, this is `$XDG_DATA_HOME/synthcode`.
/// On Windows, this is `%LOCALAPPDATA%\SynthCode`.
static CURRENT_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// The resolved config directory, combining custom override or platform defaults.
/// This is set once and cached for subsequent calls.
/// On macOS, this is `~/.config/synthcode`.
/// On Linux/FreeBSD, this is `$XDG_CONFIG_HOME/synthcode`.
/// On Windows, this is `%APPDATA%\SynthCode`.
static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Returns the relative path to the zed_server directory on the ssh host.
pub fn remote_server_dir_relative() -> &'static RelPath {
    static CACHED: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::from_unix_str(".zed_server").unwrap());
    *CACHED
}

// Remove this once 223 goes stable
/// Returns the relative path to the zed_wsl_server directory on the wsl host.
pub fn remote_wsl_server_dir_relative() -> &'static RelPath {
    static CACHED: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::from_unix_str(".zed_wsl_server").unwrap());
    *CACHED
}

/// Sets a custom directory for all user data, overriding the default data directory.
/// This function must be called before any other path operations that depend on the data directory.
/// The directory's path will be canonicalized to an absolute path by a blocking FS operation.
/// The directory will be created if it doesn't exist.
///
/// # Arguments
///
/// * `dir` - The path to use as the custom data directory. This will be used as the base
///   directory for all user data, including databases, extensions, and logs.
///
/// # Returns
///
/// A reference to the static `PathBuf` containing the custom data directory path.
///
/// # Panics
///
/// Panics if:
/// * Called after the data directory has been initialized (e.g., via `data_dir` or `config_dir`)
/// * The directory's path cannot be canonicalized to an absolute path
/// * The directory cannot be created
pub fn set_custom_data_dir(dir: &str) -> &'static PathBuf {
    if CURRENT_DATA_DIR.get().is_some() || CONFIG_DIR.get().is_some() {
        panic!("set_custom_data_dir called after data_dir or config_dir was initialized");
    }
    CUSTOM_DATA_DIR.get_or_init(|| {
        let path = PathBuf::from(dir);
        std::fs::create_dir_all(&path).expect("failed to create custom data directory");
        let canonicalized = path
            .canonicalize()
            .expect("failed to canonicalize custom data directory's path to an absolute path");
        // On Windows, `canonicalize` produces extended-length paths prefixed
        // with `\\?\`. Strip that prefix so downstream consumers (e.g.
        // Node.js language servers) that receive derived paths as arguments
        // don't choke on the verbatim syntax.
        SanitizedPath::new(&canonicalized).as_path().to_path_buf()
    })
}

/// Returns the path to the configuration directory used by Zed.
pub fn config_dir() -> &'static PathBuf {
    CONFIG_DIR.get_or_init(|| {
        if let Some(custom_dir) = CUSTOM_DATA_DIR.get() {
            custom_dir.join("config")
        } else if cfg!(target_os = "windows") {
            dirs::config_dir()
                .expect("failed to determine RoamingAppData directory")
                .join(APP_NAME)
        } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            if let Ok(flatpak_xdg_config) = std::env::var("FLATPAK_XDG_CONFIG_HOME") {
                flatpak_xdg_config.into()
            } else {
                dirs::config_dir().expect("failed to determine XDG_CONFIG_HOME directory")
            }
            .join(APP_NAME_LOWERCASE)
        } else {
            home_dir().join(".config").join(APP_NAME_LOWERCASE)
        }
    })
}

/// Returns the path to the data directory used by Zed.
pub fn data_dir() -> &'static PathBuf {
    CURRENT_DATA_DIR.get_or_init(|| {
        if let Some(custom_dir) = CUSTOM_DATA_DIR.get() {
            custom_dir.clone()
        } else if cfg!(target_os = "macos") {
            home_dir()
                .join("Library/Application Support")
                .join(APP_NAME)
        } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            if let Ok(flatpak_xdg_data) = std::env::var("FLATPAK_XDG_DATA_HOME") {
                flatpak_xdg_data.into()
            } else {
                dirs::data_local_dir().expect("failed to determine XDG_DATA_HOME directory")
            }
            .join(APP_NAME_LOWERCASE)
        } else if cfg!(target_os = "windows") {
            dirs::data_local_dir()
                .expect("failed to determine LocalAppData directory")
                .join(APP_NAME)
        } else {
            config_dir().clone() // Fallback
        }
    })
}

pub fn state_dir() -> &'static PathBuf {
    static STATE_DIR: OnceLock<PathBuf> = OnceLock::new();
    STATE_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            return home_dir().join(".local").join("state").join(APP_NAME);
        }

        if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            return if let Ok(flatpak_xdg_state) = std::env::var("FLATPAK_XDG_STATE_HOME") {
                flatpak_xdg_state.into()
            } else {
                dirs::state_dir().expect("failed to determine XDG_STATE_HOME directory")
            }
            .join(APP_NAME_LOWERCASE);
        } else {
            // Windows
            return dirs::data_local_dir()
                .expect("failed to determine LocalAppData directory")
                .join(APP_NAME);
        }
    })
}

/// Returns the path to the temp directory used by Zed.
pub fn temp_dir() -> &'static PathBuf {
    static TEMP_DIR: OnceLock<PathBuf> = OnceLock::new();
    TEMP_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            return dirs::cache_dir()
                .expect("failed to determine cachesDirectory directory")
                .join(APP_NAME);
        }

        if cfg!(target_os = "windows") {
            return dirs::cache_dir()
                .expect("failed to determine LocalAppData directory")
                .join(APP_NAME);
        }

        if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            return if let Ok(flatpak_xdg_cache) = std::env::var("FLATPAK_XDG_CACHE_HOME") {
                flatpak_xdg_cache.into()
            } else {
                dirs::cache_dir().expect("failed to determine XDG_CACHE_HOME directory")
            }
            .join(APP_NAME_LOWERCASE);
        }

        home_dir().join(".cache").join(APP_NAME_LOWERCASE)
    })
}

/// Returns the path to the hang traces directory.
pub fn hang_traces_dir() -> &'static PathBuf {
    static LOGS_DIR: OnceLock<PathBuf> = OnceLock::new();
    LOGS_DIR.get_or_init(|| data_dir().join("hang_traces"))
}

/// Returns the path to the logs directory.
pub fn logs_dir() -> &'static PathBuf {
    static LOGS_DIR: OnceLock<PathBuf> = OnceLock::new();
    LOGS_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            home_dir().join("Library/Logs").join(APP_NAME)
        } else {
            data_dir().join("logs")
        }
    })
}

/// Returns the path to the Zed server directory on this SSH host.
pub fn remote_server_state_dir() -> &'static PathBuf {
    static REMOTE_SERVER_STATE: OnceLock<PathBuf> = OnceLock::new();
    REMOTE_SERVER_STATE.get_or_init(|| data_dir().join("server_state"))
}

/// Returns the path to the `SynthCode.log` file.
pub fn log_file() -> &'static PathBuf {
    static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();
    LOG_FILE.get_or_init(|| logs_dir().join(format!("{}.log", APP_NAME)))
}

/// Returns the path to the `SynthCode.log.old` file.
pub fn old_log_file() -> &'static PathBuf {
    static OLD_LOG_FILE: OnceLock<PathBuf> = OnceLock::new();
    OLD_LOG_FILE.get_or_init(|| logs_dir().join(format!("{}.log.old", APP_NAME)))
}

/// Returns the path to the database directory.
pub fn database_dir() -> &'static PathBuf {
    static DATABASE_DIR: OnceLock<PathBuf> = OnceLock::new();
    DATABASE_DIR.get_or_init(|| data_dir().join("db"))
}

/// Returns the path to the crashes directory, if it exists for the current platform.
pub fn crashes_dir() -> &'static Option<PathBuf> {
    static CRASHES_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    CRASHES_DIR.get_or_init(|| {
        cfg!(target_os = "macos").then_some(home_dir().join("Library/Logs/DiagnosticReports"))
    })
}

/// Returns the path to the retired crashes directory, if it exists for the current platform.
pub fn crashes_retired_dir() -> &'static Option<PathBuf> {
    static CRASHES_RETIRED_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    CRASHES_RETIRED_DIR.get_or_init(|| crashes_dir().as_ref().map(|dir| dir.join("Retired")))
}

/// Returns the path to the `settings.json` file.
pub fn settings_file() -> &'static PathBuf {
    static SETTINGS_FILE: OnceLock<PathBuf> = OnceLock::new();
    SETTINGS_FILE.get_or_init(|| config_dir().join("settings.json"))
}

/// Returns the path to the global settings file.
pub fn global_settings_file() -> &'static PathBuf {
    static GLOBAL_SETTINGS_FILE: OnceLock<PathBuf> = OnceLock::new();
    GLOBAL_SETTINGS_FILE.get_or_init(|| config_dir().join("global_settings.json"))
}

/// Returns the path to the `settings_backup.json` file.
pub fn settings_backup_file() -> &'static PathBuf {
    static SETTINGS_FILE: OnceLock<PathBuf> = OnceLock::new();
    SETTINGS_FILE.get_or_init(|| config_dir().join("settings_backup.json"))
}

/// Returns the path to the `keymap.json` file.
pub fn keymap_file() -> &'static PathBuf {
    static KEYMAP_FILE: OnceLock<PathBuf> = OnceLock::new();
    KEYMAP_FILE.get_or_init(|| config_dir().join("keymap.json"))
}

/// Returns the path to the `keymap_backup.json` file.
pub fn keymap_backup_file() -> &'static PathBuf {
    static KEYMAP_FILE: OnceLock<PathBuf> = OnceLock::new();
    KEYMAP_FILE.get_or_init(|| config_dir().join("keymap_backup.json"))
}

/// Returns the path to the `tasks.json` file.
pub fn tasks_file() -> &'static PathBuf {
    static TASKS_FILE: OnceLock<PathBuf> = OnceLock::new();
    TASKS_FILE.get_or_init(|| config_dir().join("tasks.json"))
}

/// Returns the path to the `debug.json` file.
pub fn debug_scenarios_file() -> &'static PathBuf {
    static DEBUG_SCENARIOS_FILE: OnceLock<PathBuf> = OnceLock::new();
    DEBUG_SCENARIOS_FILE.get_or_init(|| config_dir().join("debug.json"))
}

/// Returns the path to the user-global `AGENTS.md` file.
///
/// This file holds personal agent instructions that apply to every project the
/// user opens, and is loaded into the native Zed agent's system prompt.
pub fn agents_file() -> &'static PathBuf {
    static AGENTS_FILE: OnceLock<PathBuf> = OnceLock::new();
    AGENTS_FILE.get_or_init(|| config_dir().join("AGENTS.md"))
}

/// User-facing display form of the user-global `AGENTS.md` file path —
/// i.e. what a human should see in messages and prompts, with the
/// platform's native path separator and home/config directory shorthand.
///
/// Windows doesn't recognize `~` as the home directory, so the env-var
/// form (`%APPDATA%`) is used there instead. Note that this is the
/// *typical* location: a user with `XDG_CONFIG_HOME` set or running in a
/// Flatpak sandbox would see a different `agents_file()` at runtime than
/// this displays. The display string trades that precision for
/// readability in announcement copy.
#[cfg(target_os = "windows")]
pub const GLOBAL_AGENTS_FILE_DISPLAY: &str =
    const_format::concatcp!("%APPDATA%\\", APP_NAME, "\\AGENTS.md");
#[cfg(not(target_os = "windows"))]
pub const GLOBAL_AGENTS_FILE_DISPLAY: &str =
    const_format::concatcp!("~/.config/", APP_NAME_LOWERCASE, "/AGENTS.md");

/// Returns the path to the extensions directory.
///
/// This is where installed extensions are stored.
pub fn extensions_dir() -> &'static PathBuf {
    static EXTENSIONS_DIR: OnceLock<PathBuf> = OnceLock::new();
    EXTENSIONS_DIR.get_or_init(|| data_dir().join("extensions"))
}

/// Returns the path to the extensions directory.
///
/// This is where installed extensions are stored on a remote.
pub fn remote_extensions_dir() -> &'static PathBuf {
    static EXTENSIONS_DIR: OnceLock<PathBuf> = OnceLock::new();
    EXTENSIONS_DIR.get_or_init(|| data_dir().join("remote_extensions"))
}

/// Returns the path to the extensions directory.
///
/// This is where installed extensions are stored on a remote.
pub fn remote_extensions_uploads_dir() -> &'static PathBuf {
    static UPLOAD_DIR: OnceLock<PathBuf> = OnceLock::new();
    UPLOAD_DIR.get_or_init(|| remote_extensions_dir().join("uploads"))
}

/// Returns the path to the themes directory.
///
/// This is where themes that are not provided by extensions are stored.
pub fn themes_dir() -> &'static PathBuf {
    static THEMES_DIR: OnceLock<PathBuf> = OnceLock::new();
    THEMES_DIR.get_or_init(|| config_dir().join("themes"))
}

/// Returns the path to the snippets directory.
pub fn snippets_dir() -> &'static PathBuf {
    static SNIPPETS_DIR: OnceLock<PathBuf> = OnceLock::new();
    SNIPPETS_DIR.get_or_init(|| config_dir().join("snippets"))
}

/// Returns the path to the contexts directory.
///
/// This is where the prompts for use with the Assistant are stored.
pub fn prompts_dir() -> &'static PathBuf {
    static PROMPTS_DIR: OnceLock<PathBuf> = OnceLock::new();
    PROMPTS_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            config_dir().join("prompts")
        } else {
            data_dir().join("prompts")
        }
    })
}

/// Returns the path to the prompt templates directory.
///
/// This is where the prompt templates for core features can be overridden with templates.
///
/// # Arguments
///
/// * `dev_mode` - If true, assumes the current working directory is the Zed repository.
pub fn prompt_overrides_dir(repo_path: Option<&Path>) -> PathBuf {
    if let Some(path) = repo_path {
        let dev_path = path.join("assets").join("prompts");
        if dev_path.exists() {
            return dev_path;
        }
    }

    static PROMPT_TEMPLATES_DIR: OnceLock<PathBuf> = OnceLock::new();
    PROMPT_TEMPLATES_DIR
        .get_or_init(|| {
            if cfg!(target_os = "macos") {
                config_dir().join("prompt_overrides")
            } else {
                data_dir().join("prompt_overrides")
            }
        })
        .clone()
}

/// Returns the path to the semantic search's embeddings directory.
///
/// This is where the embeddings used to power semantic search are stored.
pub fn embeddings_dir() -> &'static PathBuf {
    static EMBEDDINGS_DIR: OnceLock<PathBuf> = OnceLock::new();
    EMBEDDINGS_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            config_dir().join("embeddings")
        } else {
            data_dir().join("embeddings")
        }
    })
}

/// Returns the path to the languages directory.
///
/// This is where language servers are downloaded to for languages built-in to Zed.
pub fn languages_dir() -> &'static PathBuf {
    static LANGUAGES_DIR: OnceLock<PathBuf> = OnceLock::new();
    LANGUAGES_DIR.get_or_init(|| data_dir().join("languages"))
}

/// Returns the path to the debug adapters directory
///
/// This is where debug adapters are downloaded to for DAPs that are built-in to Zed.
pub fn debug_adapters_dir() -> &'static PathBuf {
    static DEBUG_ADAPTERS_DIR: OnceLock<PathBuf> = OnceLock::new();
    DEBUG_ADAPTERS_DIR.get_or_init(|| data_dir().join("debug_adapters"))
}

/// Returns the path to the external agents directory
///
/// This is where agent servers are downloaded to
pub fn external_agents_dir() -> &'static PathBuf {
    static EXTERNAL_AGENTS_DIR: OnceLock<PathBuf> = OnceLock::new();
    EXTERNAL_AGENTS_DIR.get_or_init(|| data_dir().join("external_agents"))
}

/// Returns the path to the Copilot directory.
pub fn copilot_dir() -> &'static PathBuf {
    static COPILOT_DIR: OnceLock<PathBuf> = OnceLock::new();
    COPILOT_DIR.get_or_init(|| data_dir().join("copilot"))
}

/// Returns the path to the default Prettier directory.
pub fn default_prettier_dir() -> &'static PathBuf {
    static DEFAULT_PRETTIER_DIR: OnceLock<PathBuf> = OnceLock::new();
    DEFAULT_PRETTIER_DIR.get_or_init(|| data_dir().join("prettier"))
}

/// Returns the path to the remote server binaries directory.
pub fn remote_servers_dir() -> &'static PathBuf {
    static REMOTE_SERVERS_DIR: OnceLock<PathBuf> = OnceLock::new();
    REMOTE_SERVERS_DIR.get_or_init(|| data_dir().join("remote_servers"))
}

/// Returns the path to the directory where the devcontainer CLI is installed.
pub fn devcontainer_dir() -> &'static PathBuf {
    static DEVCONTAINER_DIR: OnceLock<PathBuf> = OnceLock::new();
    DEVCONTAINER_DIR.get_or_init(|| data_dir().join("devcontainer"))
}

/// Returns the relative path to a `.zed` folder within a project.
pub fn local_settings_folder_name() -> &'static str {
    ".zed"
}

/// Returns the relative path to a `.vscode` folder within a project.
pub fn local_vscode_folder_name() -> &'static str {
    ".vscode"
}

/// Returns the relative path to a `settings.json` file within a project.
pub fn local_settings_file_relative_path() -> &'static RelPath {
    static CACHED: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::from_unix_str(".zed/settings.json").unwrap());
    *CACHED
}

/// Returns the relative path to a `tasks.json` file within a project.
pub fn local_tasks_file_relative_path() -> &'static RelPath {
    static CACHED: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::from_unix_str(".zed/tasks.json").unwrap());
    *CACHED
}

/// Returns the relative path to a `.vscode/tasks.json` file within a project.
pub fn local_vscode_tasks_file_relative_path() -> &'static RelPath {
    static CACHED: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::from_unix_str(".vscode/tasks.json").unwrap());
    *CACHED
}

pub fn debug_task_file_name() -> &'static str {
    "debug.json"
}

pub fn task_file_name() -> &'static str {
    "tasks.json"
}

/// Returns the relative path to a `debug.json` file within a project.
/// .zed/debug.json
pub fn local_debug_file_relative_path() -> &'static RelPath {
    static CACHED: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::from_unix_str(".zed/debug.json").unwrap());
    *CACHED
}

/// Returns the relative path to a `.vscode/launch.json` file within a project.
pub fn local_vscode_launch_file_relative_path() -> &'static RelPath {
    static CACHED: LazyLock<&'static RelPath> =
        LazyLock::new(|| RelPath::from_unix_str(".vscode/launch.json").unwrap());
    *CACHED
}

pub fn user_ssh_config_file() -> PathBuf {
    home_dir().join(".ssh/config")
}

pub fn global_ssh_config_file() -> Option<&'static Path> {
    if cfg!(windows) {
        None
    } else {
        Some(Path::new("/etc/ssh/ssh_config"))
    }
}

/// Returns candidate paths for the vscode user settings file
pub fn vscode_settings_file_paths() -> Vec<PathBuf> {
    let mut paths = vscode_user_data_paths();
    for path in paths.iter_mut() {
        path.push("User/settings.json");
    }
    paths
}

/// Returns candidate paths for the cursor user settings file
pub fn cursor_settings_file_paths() -> Vec<PathBuf> {
    let mut paths = cursor_user_data_paths();
    for path in paths.iter_mut() {
        path.push("User/settings.json");
    }
    paths
}

fn vscode_user_data_paths() -> Vec<PathBuf> {
    // https://github.com/microsoft/vscode/blob/23e7148cdb6d8a27f0109ff77e5b1e019f8da051/src/vs/platform/environment/node/userDataPath.ts#L45
    const VSCODE_PRODUCT_NAMES: &[&str] = &[
        "Code",
        "Code - Insiders",
        "Code - OSS",
        "VSCodium",
        "VSCodium - Insiders",
        "Code Dev",
        "Code - OSS Dev",
        "code-oss-dev",
    ];
    let mut paths = Vec::new();
    if let Ok(portable_path) = env::var("VSCODE_PORTABLE") {
        paths.push(Path::new(&portable_path).join("user-data"));
    }
    if let Ok(vscode_appdata) = env::var("VSCODE_APPDATA") {
        for product_name in VSCODE_PRODUCT_NAMES {
            paths.push(Path::new(&vscode_appdata).join(product_name));
        }
    }
    for product_name in VSCODE_PRODUCT_NAMES {
        add_vscode_user_data_paths(&mut paths, product_name);
    }
    paths
}

fn cursor_user_data_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    add_vscode_user_data_paths(&mut paths, "Cursor");
    paths
}

fn add_vscode_user_data_paths(paths: &mut Vec<PathBuf>, product_name: &str) {
    if cfg!(target_os = "macos") {
        paths.push(
            home_dir()
                .join("Library/Application Support")
                .join(product_name),
        );
    } else if cfg!(target_os = "windows") {
        if let Some(data_local_dir) = dirs::data_local_dir() {
            paths.push(data_local_dir.join(product_name));
        }
        if let Some(data_dir) = dirs::data_dir() {
            paths.push(data_dir.join(product_name));
        }
    } else {
        paths.push(
            dirs::config_dir()
                .unwrap_or(home_dir().join(".config"))
                .join(product_name),
        );
    }
}

#[cfg(any(test, feature = "test-support"))]
pub fn global_gitignore_path() -> Option<PathBuf> {
    Some(home_dir().join(".config").join("git").join("ignore"))
}

#[cfg(not(any(test, feature = "test-support")))]
pub fn global_gitignore_path() -> Option<PathBuf> {
    static GLOBAL_GITIGNORE_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
    GLOBAL_GITIGNORE_PATH
        .get_or_init(::ignore::gitignore::gitconfig_excludes_path)
        .clone()
}

/// Synth Code fork: the previous app name. Its folders are moved on first launch.
const LEGACY_APP_NAME: &str = "SynthZed";

/// Synth Code fork: moves each per-app folder left by the previous app name ("SynthZed") to its
/// current location. Call it before anything creates those folders. Returns one line per moved or
/// failed folder, for logging once logging is up.
///
/// A current folder that exists but was never used by a real launch (it lacks the marker below,
/// for example because a test run created it) is moved aside as `<name>.pre-migration` first.
pub fn migrate_legacy_app_dirs() -> Vec<String> {
    // `settings.json` and `db` are written by the first real launch; a folder without them holds
    // nothing the user made.
    let config = (config_dir(), Some("settings.json"));
    // A custom data directory is the user's own choice, so only the config folder is migrated.
    let dirs: Vec<(&'static PathBuf, Option<&str>)> = if CUSTOM_DATA_DIR.get().is_some() {
        vec![config]
    } else {
        vec![
            config,
            (data_dir(), Some("db")),
            (state_dir(), None),
            (temp_dir(), None),
            (logs_dir(), None),
        ]
    };
    let mut report = Vec::new();
    for (dir, marker) in dirs {
        let Some(legacy) = legacy_app_dir(dir) else {
            continue;
        };
        let had_legacy = legacy.is_dir();
        match migrate_dir(&legacy, dir, marker) {
            Ok(aside) => {
                if let Some(aside) = aside {
                    report.push(format!(
                        "set aside the unused {} as {}",
                        dir.display(),
                        aside.display()
                    ));
                }
                if had_legacy && !legacy.exists() {
                    report.push(format!("moved {} to {}", legacy.display(), dir.display()));
                }
            }
            Err(error) => report.push(format!(
                "could not move {} to {}: {error}",
                legacy.display(),
                dir.display()
            )),
        }
    }
    report
}

/// Moves `legacy` to `current`. When `current` exists, it is moved aside first only if
/// `marker` is given and missing from it. Returns where an unused `current` was set aside.
fn migrate_dir(
    legacy: &Path,
    current: &Path,
    marker: Option<&str>,
) -> std::io::Result<Option<PathBuf>> {
    if !legacy.is_dir() {
        return Ok(None);
    }
    let mut aside = None;
    if current.exists() {
        let Some(marker) = marker else {
            return Ok(None);
        };
        if current.join(marker).exists() {
            return Ok(None);
        }
        let Some(name) = current.file_name() else {
            return Ok(None);
        };
        let target = current.with_file_name(format!("{}.pre-migration", name.to_string_lossy()));
        if target.exists() {
            return Ok(None);
        }
        std::fs::rename(current, &target)?;
        aside = Some(target);
    }
    std::fs::rename(legacy, current)?;
    Ok(aside)
}

fn legacy_app_dir(dir: &Path) -> Option<PathBuf> {
    let name = dir.file_name()?.to_str()?;
    let legacy = if name == APP_NAME {
        LEGACY_APP_NAME.to_string()
    } else if name == APP_NAME_LOWERCASE {
        LEGACY_APP_NAME.to_ascii_lowercase()
    } else {
        return None;
    };
    Some(dir.with_file_name(legacy))
}

#[cfg(test)]
mod legacy_migration_tests {
    use super::*;

    #[test]
    fn maps_app_dir_names_to_the_legacy_ones() {
        assert_eq!(
            legacy_app_dir(Path::new("/root/SynthCode")),
            Some(PathBuf::from("/root/SynthZed"))
        );
        assert_eq!(
            legacy_app_dir(Path::new("/root/synthcode")),
            Some(PathBuf::from("/root/synthzed"))
        );
        assert_eq!(legacy_app_dir(Path::new("/root/logs")), None);
    }

    fn scratch_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "paths-legacy-migration-{name}-{}",
            std::process::id()
        ));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn moves_the_legacy_dir_when_the_current_one_is_missing() {
        let root = scratch_root("missing");
        let legacy = root.join("SynthZed");
        let current = root.join("SynthCode");
        std::fs::create_dir_all(legacy.join("db")).unwrap();

        assert_eq!(migrate_dir(&legacy, &current, Some("db")).unwrap(), None);
        assert!(current.join("db").is_dir());
        assert!(!legacy.exists());

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn keeps_a_used_current_dir() {
        let root = scratch_root("used");
        let legacy = root.join("SynthZed");
        let current = root.join("SynthCode");
        std::fs::create_dir_all(legacy.join("db")).unwrap();
        std::fs::create_dir_all(current.join("db")).unwrap();

        assert_eq!(migrate_dir(&legacy, &current, Some("db")).unwrap(), None);
        assert!(legacy.exists());
        assert!(!root.join("SynthCode.pre-migration").exists());

        // Without a marker, an existing current folder always wins.
        std::fs::remove_dir_all(current.join("db")).unwrap();
        assert_eq!(migrate_dir(&legacy, &current, None).unwrap(), None);
        assert!(legacy.exists());

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn sets_aside_an_unused_current_dir() {
        let root = scratch_root("unused");
        let legacy = root.join("SynthZed");
        let current = root.join("SynthCode");
        std::fs::create_dir_all(legacy.join("db")).unwrap();
        // What a test run leaves behind: a prompts database, but no workspace db.
        std::fs::create_dir_all(current.join("prompts")).unwrap();
        std::fs::write(current.join("prompts").join("data.mdb"), "x").unwrap();

        let aside = root.join("SynthCode.pre-migration");
        assert_eq!(
            migrate_dir(&legacy, &current, Some("db")).unwrap(),
            Some(aside.clone())
        );
        assert!(current.join("db").is_dir());
        assert!(aside.join("prompts").join("data.mdb").is_file());
        assert!(!legacy.exists());

        // A second unused folder never overwrites an earlier one set aside.
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::remove_dir_all(&current).unwrap();
        std::fs::create_dir_all(&current).unwrap();
        assert_eq!(migrate_dir(&legacy, &current, Some("db")).unwrap(), None);
        assert!(legacy.exists());

        std::fs::remove_dir_all(&root).unwrap();
    }
}

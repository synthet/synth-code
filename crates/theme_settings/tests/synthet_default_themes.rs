use std::collections::HashSet;

use theme::ThemeRegistry;

const SHARED_DEFAULT_THEME_NAMES: [&str; 35] = [
    "Adeberry",
    "Ayu Dark",
    "Ayu Light",
    "Ayu Mirage",
    "Catppuccin Mocha",
    "Cyber Wave",
    "Dark",
    "Dark City",
    "Dracula",
    "Fancy Dracula",
    "GitHub Dark Default",
    "Gruvbox Dark",
    "Gruvbox Dark Hard",
    "Gruvbox Dark Soft",
    "Gruvbox Light",
    "Gruvbox Light Hard",
    "Gruvbox Light Soft",
    "Jellyfish",
    "JetBrains Darcula",
    "Koi",
    "Leafy",
    "Light",
    "Marble",
    "Nord",
    "One Dark",
    "One Light",
    "Phenomenon",
    "Pink City",
    "Red Rock",
    "Snowy",
    "Solar Flare",
    "Solarized Dark",
    "Solarized Light",
    "Tokyo Night",
    "Willow Dream",
];

#[test]
fn shared_catalog_themes_have_plain_bundled_names() {
    let registry = ThemeRegistry::new(Box::new(assets::Assets));
    theme_settings::load_bundled_themes(&registry);
    let bundled_names = registry.list_names().into_iter().collect::<HashSet<_>>();

    let missing = SHARED_DEFAULT_THEME_NAMES
        .into_iter()
        .filter(|name| !bundled_names.contains(*name))
        .collect::<Vec<_>>();
    let redundant_names = bundled_names
        .iter()
        .filter(|name| name.starts_with("Synthet "))
        .collect::<Vec<_>>();

    assert!(missing.is_empty(), "missing bundled themes: {missing:?}");
    assert!(
        redundant_names.is_empty(),
        "themes with redundant name hints: {redundant_names:?}"
    );
}

#[test]
fn only_theme_json_is_embedded() {
    let non_json = gpui::AssetSource::list(&assets::Assets, "themes/")
        .expect("failed to list theme assets")
        .into_iter()
        .filter(|path| !path.ends_with(".json"))
        .collect::<Vec<_>>();

    assert!(
        non_json.is_empty(),
        "non-theme assets embedded under themes/: {non_json:?}"
    );
}

/// Tinted buttons (`ButtonStyle::Tinted`) and status callouts draw with the
/// status background and border colors. A fork theme that omits them falls back
/// to the default theme's bright blue, which clashes with themes such as
/// JetBrains Darcula.
#[test]
fn synthet_themes_define_status_backgrounds_and_borders() {
    let mut missing = Vec::new();
    for path in gpui::AssetSource::list(&assets::Assets, "themes/synthet/")
        .expect("failed to list synthet theme assets")
    {
        let bytes = gpui::AssetSource::load(&assets::Assets, &path)
            .expect("failed to load theme asset")
            .expect("theme asset missing");
        let family: serde_json::Value =
            serde_json::from_slice(&bytes).expect("theme asset is not valid JSON");
        for theme in family["themes"].as_array().into_iter().flatten() {
            let style = &theme["style"];
            for status in ["info", "error", "warning", "success"] {
                for suffix in ["background", "border"] {
                    let key = format!("{status}.{suffix}");
                    if style[&key].as_str().is_none_or(str::is_empty) {
                        missing.push(format!("{}: {key}", theme["name"]));
                    }
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "themes missing status colors: {missing:?}"
    );
}

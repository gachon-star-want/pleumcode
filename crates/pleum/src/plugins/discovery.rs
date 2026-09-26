use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{paths::Paths, Config};
use crate::plugins::plugin_install_dir;

pub(in crate::plugins) const PLUGINS_CONFIG_KEY: &str = "plugins";

/// Per-plugin entry stored under the `plugins` map in `config.yaml`, keyed by
/// the plugin's filesystem path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(in crate::plugins) struct PluginConfigEntry {
    pub enabled: bool,
}

/// Every plugin root ever discovered on disk, regardless of scope or enabled
/// state. Deliberately separate from [`PLUGINS_CONFIG_KEY`]: that map is a trust
/// decision (which `filter_by_config` won't auto-write for a project-scope
/// plugin — see its doc comment), while this is pure "have we ever seen a
/// plugin at this path" bookkeeping, safe to record unconditionally. Consumers
/// like [`crate::plugins::configured_project_plugin_skill_dirs`] use it to
/// recognize a project-plugin-owned path (so a symlink-following fallback
/// lookup never serves it) independent of whether that plugin is enabled.
pub(in crate::plugins) const KNOWN_PLUGIN_PATHS_KEY: &str = "known_plugin_paths";

pub(in crate::plugins) fn record_known_plugin_path(config: &Config, root: &Path) {
    let mut known: HashSet<String> = config.get_param(KNOWN_PLUGIN_PATHS_KEY).unwrap_or_default();
    if known.insert(root.to_string_lossy().into_owned()) {
        if let Err(e) = config.set_param(KNOWN_PLUGIN_PATHS_KEY, known) {
            tracing::warn!(error = %e, "Failed to persist known plugin path");
        }
    }
}

pub(crate) fn known_plugin_paths(config: &Config) -> HashSet<String> {
    config.get_param(KNOWN_PLUGIN_PATHS_KEY).unwrap_or_default()
}

/// A plugin found on disk and not disabled by any settings file.
#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    pub name: String,
    pub root: PathBuf,
    pub scope: PluginScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginScope {
    User,
    Project,
}

/// Settings file format from <https://open-plugins.com/plugin-builders/installation>.
#[derive(Debug, Default, Deserialize)]
struct PluginSettings {
    #[serde(default, rename = "enabledPlugins")]
    enabled: Vec<String>,
    #[serde(default, rename = "disabledPlugins")]
    disabled: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SettingsScope {
    Local,
    Project,
    User,
}

/// Discover all plugins that should be considered active.
///
/// `project_root`, when supplied, enables project + local scope settings and
/// project-scope `.agents/plugins/` lookups.
pub fn discover_enabled_plugins(project_root: Option<&Path>) -> Vec<DiscoveredPlugin> {
    discover_enabled_plugins_with_config(project_root, Config::global())
}

pub(crate) fn discover_enabled_plugins_with_config(
    project_root: Option<&Path>,
    config: &Config,
) -> Vec<DiscoveredPlugin> {
    let scoped_settings = load_all_settings(project_root);
    let user_plugins_dir = plugin_install_dir();
    let mut found = Vec::new();

    if let Some(root) = project_root {
        let project_plugins_dir = project_plugin_dir(root);
        if !equivalent_paths(&project_plugins_dir, &user_plugins_dir) {
            found.extend(list_dir_children(&project_plugins_dir).into_iter().map(
                |(name, root)| DiscoveredPlugin {
                    name,
                    root,
                    scope: PluginScope::Project,
                },
            ));
        }
    }
    found.extend(
        list_dir_children(&user_plugins_dir)
            .into_iter()
            .map(|(name, root)| DiscoveredPlugin {
                name,
                root,
                scope: PluginScope::User,
            }),
    );

    let mut enabled_plugins: Vec<DiscoveredPlugin> = filter_by_config(found, config)
        .into_iter()
        // `decided`: config.yaml already recorded a `true` for this plugin, from
        // outside the repo (an explicit prior entry, or the auto-write below for a
        // user-scope plugin) — that decision is authoritative on its own. Anything
        // left undecided (a project-scope plugin nobody has approved yet) still has
        // to clear the settings.json scope gate in `is_enabled`.
        .filter(|(plugin, decided)| *decided || is_enabled(plugin, &scoped_settings))
        .map(|(plugin, _)| plugin)
        .collect();
    enabled_plugins.sort_by(|left, right| {
        plugin_scope_rank(left.scope)
            .cmp(&plugin_scope_rank(right.scope))
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.root.cmp(&right.root))
    });

    let mut seen_names = HashSet::new();
    enabled_plugins
        .into_iter()
        .filter(|plugin| seen_names.insert(plugin.name.clone()))
        .collect()
}

fn equivalent_paths(left: &Path, right: &Path) -> bool {
    left == right
        || left
            .canonicalize()
            .ok()
            .zip(right.canonicalize().ok())
            .is_some_and(|(left, right)| left == right)
}

fn plugin_scope_rank(scope: PluginScope) -> u8 {
    match scope {
        PluginScope::Project => 0,
        PluginScope::User => 1,
    }
}

/// Apply the `plugins` map in `config.yaml`, which lives outside any repo
/// (`~/.config/pleum/config.yaml`, or under `PLEUM_PATH_ROOT`) and so can only be
/// written by the user or another trusted local component — never by the repo
/// itself. An existing `true` entry (however it got there) is returned as
/// already `decided`, meaning the caller can skip the settings.json trust-scope
/// gate for it. A plugin with no entry yet is auto-added as `enabled: true`
/// (and marked `decided`) only for [`PluginScope::User`] — that scope only ever
/// gets populated by an explicit `pleum plugin install`, so first sight already
/// *is* the trust decision. A [`PluginScope::Project`] plugin is just a file
/// that showed up in a repo someone opened; it is passed through undecided,
/// leaving [`is_enabled`]'s scope-gated default (closed) to apply instead of
/// silently writing an unearned `true` into the user's own config.
fn filter_by_config(
    plugins: Vec<DiscoveredPlugin>,
    config: &Config,
) -> Vec<(DiscoveredPlugin, bool)> {
    let mut entries: HashMap<String, PluginConfigEntry> =
        config.get_param(PLUGINS_CONFIG_KEY).unwrap_or_default();

    let mut dirty = false;
    let mut kept = Vec::new();
    for plugin in plugins {
        record_known_plugin_path(config, &plugin.root);
        let key = plugin.root.to_string_lossy().to_string();
        match entries.get(&key) {
            Some(entry) => {
                if entry.enabled {
                    kept.push((plugin, true));
                }
            }
            None => {
                let decided = plugin.scope == PluginScope::User;
                if decided {
                    entries.insert(key, PluginConfigEntry { enabled: true });
                    dirty = true;
                }
                kept.push((plugin, decided));
            }
        }
    }

    if dirty {
        if let Err(e) = config.set_param(PLUGINS_CONFIG_KEY, entries) {
            tracing::warn!(error = %e, "Failed to persist plugin config entries");
        }
    }

    kept
}

/// A [`PluginScope::Project`] plugin lives inside the repo someone just opened, so
/// it is untrusted by default: its own repo-committed `settings.json` (Project
/// scope) may still *disable* it (restricting your own repo is always safe), but
/// may never be the reason it turns *on* — otherwise a hostile repo would just
/// re-enable itself by shipping the matching `enabledPlugins` entry alongside it.
/// Only `settings.local.json` (Local, git-ignored by convention) or the user's own
/// `~/.config/pleum/settings.json` (User, outside any repo) can enable it. A
/// [`PluginScope::User`] plugin was already an explicit `pleum plugin install`, so
/// any scope enabling it (the old, unrestricted behavior) is fine.
fn is_enabled(
    plugin: &DiscoveredPlugin,
    scoped_settings: &[(SettingsScope, PluginSettings)],
) -> bool {
    for scope in [
        SettingsScope::Local,
        SettingsScope::Project,
        SettingsScope::User,
    ] {
        let Some(settings) = scoped_settings
            .iter()
            .find_map(|(s, settings)| (*s == scope).then_some(settings))
        else {
            continue;
        };

        if settings.disabled.iter().any(|n| n == plugin.name.as_str()) {
            return false;
        }
        let listed_enabled = settings.enabled.iter().any(|n| n == plugin.name.as_str());
        let scope_may_enable =
            scope != SettingsScope::Project || plugin.scope != PluginScope::Project;
        if listed_enabled && scope_may_enable {
            return true;
        }
        // Mentioned in `enabledPlugins` but by an ineligible scope (a project-scope
        // plugin's own committed settings): not a valid signal either way, keep
        // scanning later scopes instead of treating this as "settled".
    }

    plugin.scope == PluginScope::User
}

fn project_plugin_dir(project_root: &Path) -> PathBuf {
    project_root.join(".agents").join("plugins")
}

fn list_dir_children(dir: &Path) -> Vec<(String, PathBuf)> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let name = path.file_name()?.to_str()?.to_string();
            Some((name, path))
        })
        .collect()
}

fn load_all_settings(project_root: Option<&Path>) -> Vec<(SettingsScope, PluginSettings)> {
    let mut paths: Vec<(SettingsScope, PathBuf)> = Vec::new();
    if let Some(path) = user_settings_path() {
        paths.push((SettingsScope::User, path));
    }
    if let Some(root) = project_root {
        paths.push((SettingsScope::Project, project_settings_path(root, false)));
        paths.push((SettingsScope::Local, project_settings_path(root, true)));
    }

    paths
        .into_iter()
        .filter_map(|(scope, path)| match read_settings(&path) {
            Ok(Some(s)) => Some((scope, s)),
            Ok(None) => None,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "Failed to read plugin settings");
                None
            }
        })
        .collect()
}

fn user_settings_path() -> Option<PathBuf> {
    if let Some(path_root) = Paths::path_root() {
        return Some(
            path_root
                .join(".config")
                .join("pleum")
                .join("settings.json"),
        );
    }
    Some(
        dirs::home_dir()?
            .join(".config")
            .join("pleum")
            .join("settings.json"),
    )
}

fn project_settings_path(project_root: &Path, local: bool) -> PathBuf {
    let file = if local {
        "settings.local.json"
    } else {
        "settings.json"
    };
    project_root.join(".config").join("pleum").join(file)
}

fn read_settings(path: &Path) -> anyhow::Result<Option<PluginSettings>> {
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)?;
    let parsed: PluginSettings = serde_json::from_str(&text)?;
    Ok(Some(parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_plugin_dir(root: &Path, name: &str) {
        let dir = root.join(name);
        std::fs::create_dir_all(dir.join("hooks")).unwrap();
        std::fs::write(
            dir.join("hooks").join("hooks.json"),
            r#"{"hooks":{"SessionStart":[{"hooks":[]}]}}"#,
        )
        .unwrap();
    }

    fn write_settings(dir: &Path, contents: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("settings.json"), contents).unwrap();
    }

    fn write_local_settings(dir: &Path, contents: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("settings.local.json"), contents).unwrap();
    }

    fn test_config(dir: &Path) -> Config {
        Config::new(dir.join("config.yaml"), "pleum-discovery-test").unwrap()
    }

    fn discover_with_config(project: &Path, config: &Config) -> Vec<DiscoveredPlugin> {
        let _guard = env_lock::lock_env([("PLEUM_PATH_ROOT", None::<&str>)]);
        discover_enabled_plugins_with_config(Some(project), config)
    }

    fn discover(project: &Path) -> Vec<DiscoveredPlugin> {
        let cfg_dir = tempfile::tempdir().unwrap();
        discover_with_config(project, &test_config(cfg_dir.path()))
    }

    /// The core fix: a plugin that merely showed up in a repo you opened (nothing
    /// installed it, nothing enabled it) must not run. This is the CVE-2025-59536
    /// shape — a hostile repo shipping a plugin whose SessionStart hook would
    /// otherwise fire, unsandboxed, the moment you `cd` in and start a session.
    #[test]
    fn project_scope_plugin_is_not_enabled_by_default() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        write_plugin_dir(&project.join(".agents").join("plugins"), "demo");

        let found = discover(project);
        assert!(
            found.iter().all(|p| p.name != "demo"),
            "a bare project-scope plugin must default to disabled; got: {:?}",
            found.iter().map(|p| &p.name).collect::<Vec<_>>()
        );
    }

    /// And it stays untrusted even across repeated sessions: nothing about
    /// discovering it once persists a silent "enabled" decision into the user's
    /// own config (contrast `newly_discovered_user_plugin_is_added_to_config_as_enabled`).
    #[test]
    fn project_scope_plugin_is_not_persisted_to_config() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        let plugin_root = project.join(".agents/plugins/demo");
        write_plugin_dir(&project.join(".agents").join("plugins"), "demo");

        let cfg_dir = tempfile::tempdir().unwrap();
        let config = test_config(cfg_dir.path());
        discover_with_config(project, &config);

        let entries: HashMap<String, PluginConfigEntry> =
            config.get_param(PLUGINS_CONFIG_KEY).unwrap_or_default();
        assert!(
            entries
                .get(&plugin_root.to_string_lossy().into_owned())
                .is_none(),
            "got: {entries:?}"
        );
    }

    /// A repo can still restrict its own project-scope plugin via its own
    /// committed settings.json — disabling is always safe, only enabling isn't.
    #[test]
    fn disabled_in_project_settings_drops_plugin() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        write_plugin_dir(&project.join(".agents").join("plugins"), "demo");

        write_settings(
            &project.join(".config").join("pleum"),
            r#"{"disabledPlugins":["demo"]}"#,
        );

        let found = discover(project);
        assert!(found.iter().all(|p| p.name != "demo"));
    }

    /// Listing a project-scope plugin in `enabledPlugins` does nothing for that
    /// plugin (its own repo can't self-enable) and, since the setting exists but
    /// doesn't mention "other" at all, "other" stays on its own default (also off).
    #[test]
    fn project_settings_enabled_list_does_not_enable_project_scope_plugins() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        write_plugin_dir(&project.join(".agents").join("plugins"), "demo");
        write_plugin_dir(&project.join(".agents").join("plugins"), "other");

        write_settings(
            &project.join(".config").join("pleum"),
            r#"{"enabledPlugins":["demo"]}"#,
        );

        let found = discover(project);
        let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
        assert!(!names.contains(&"demo"), "got: {names:?}");
        assert!(!names.contains(&"other"), "got: {names:?}");
    }

    /// `settings.local.json` is git-ignored by convention (same pattern as this
    /// repo's own `.claude/settings.local.json`), so it's a real per-machine user
    /// decision, not something a repo can ship — eligible to enable.
    #[test]
    fn local_settings_enabled_list_enables_project_scope_plugin() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        write_plugin_dir(&project.join(".agents").join("plugins"), "demo");
        write_plugin_dir(&project.join(".agents").join("plugins"), "other");

        write_local_settings(
            &project.join(".config").join("pleum"),
            r#"{"enabledPlugins":["demo"]}"#,
        );

        let found = discover(project);
        let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"demo"), "got: {names:?}");
        assert!(!names.contains(&"other"), "got: {names:?}");
    }

    #[test]
    fn local_scope_overrides_project_scope() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        write_plugin_dir(&project.join(".agents").join("plugins"), "demo");

        write_settings(
            &project.join(".config").join("pleum"),
            r#"{"disabledPlugins":["demo"]}"#,
        );
        write_local_settings(
            &project.join(".config").join("pleum"),
            r#"{"enabledPlugins":["demo"]}"#,
        );

        let found = discover(project);
        assert!(
            found.iter().any(|p| p.name == "demo"),
            "local scope should win; got: {:?}",
            found.iter().map(|p| &p.name).collect::<Vec<_>>()
        );
    }

    /// A repo cannot re-enable, via its own committed settings.json, a
    /// project-scope plugin of its own that the user disabled globally. Before
    /// the fix this was the reverse: project scope unconditionally beat user
    /// scope, so a hostile repo could override a global disable just by shipping
    /// the matching `enabledPlugins` entry next to the plugin.
    #[test]
    fn project_settings_cannot_override_user_disable() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        write_plugin_dir(&project.join(".agents").join("plugins"), "demo");

        let fake_home = tempfile::tempdir().unwrap();
        write_settings(
            &fake_home.path().join(".config").join("pleum"),
            r#"{"disabledPlugins":["demo"]}"#,
        );

        write_settings(
            &project.join(".config").join("pleum"),
            r#"{"enabledPlugins":["demo"]}"#,
        );

        let cfg_dir = tempfile::tempdir().unwrap();
        let found = {
            let _guard =
                env_lock::lock_env([("PLEUM_PATH_ROOT", Some(fake_home.path().to_str().unwrap()))]);
            discover_enabled_plugins_with_config(Some(project), &test_config(cfg_dir.path()))
        };

        assert!(
            found.iter().all(|p| p.name != "demo"),
            "got: {:?}",
            found.iter().map(|p| &p.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn disabled_project_plugin_falls_back_to_enabled_user_plugin_with_same_name() {
        let project = tempfile::tempdir().unwrap();
        let project_plugins = project.path().join(".agents/plugins");
        write_plugin_dir(&project_plugins, "demo");

        let path_root = tempfile::tempdir().unwrap();
        let user_plugins = path_root.path().join(".agents/plugins");
        write_plugin_dir(&user_plugins, "demo");
        let config_dir = tempfile::tempdir().unwrap();
        let config = test_config(config_dir.path());
        config
            .set_param(
                PLUGINS_CONFIG_KEY,
                HashMap::from([
                    (
                        project_plugins.join("demo").to_string_lossy().into_owned(),
                        PluginConfigEntry { enabled: false },
                    ),
                    (
                        user_plugins.join("demo").to_string_lossy().into_owned(),
                        PluginConfigEntry { enabled: true },
                    ),
                ]),
            )
            .unwrap();
        let _guard = env_lock::lock_env([
            ("PLEUM_PATH_ROOT", path_root.path().to_str()),
            ("PLUGINS", None),
        ]);

        let found = discover_enabled_plugins_with_config(Some(project.path()), &config);
        let demo: Vec<_> = found
            .into_iter()
            .filter(|plugin| plugin.name == "demo")
            .collect();

        assert_eq!(demo.len(), 1);
        assert_eq!(demo[0].scope, PluginScope::User);
        assert_eq!(demo[0].root, user_plugins.join("demo"));
    }

    /// Unlike a project-scope plugin, a user-scope one only ever gets there via an
    /// explicit `pleum plugin install` — first sight already is the trust
    /// decision, so it's fine to persist `enabled: true` on discovery.
    #[test]
    fn newly_discovered_user_plugin_is_added_to_config_as_enabled() {
        let path_root = tempfile::tempdir().unwrap();
        let user_plugins = path_root.path().join(".agents/plugins");
        write_plugin_dir(&user_plugins, "demo");

        let cfg_dir = tempfile::tempdir().unwrap();
        let config = test_config(cfg_dir.path());
        let found = {
            let _guard =
                env_lock::lock_env([("PLEUM_PATH_ROOT", Some(path_root.path().to_str().unwrap()))]);
            discover_enabled_plugins_with_config(None, &config)
        };
        assert!(found.iter().any(|p| p.name == "demo"));

        let entries: HashMap<String, PluginConfigEntry> =
            config.get_param(PLUGINS_CONFIG_KEY).unwrap();
        let key = user_plugins.join("demo").to_string_lossy().to_string();
        assert!(
            entries.get(&key).is_some_and(|e| e.enabled),
            "got: {entries:?}"
        );
    }

    #[test]
    fn disabled_in_config_drops_plugin() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        write_plugin_dir(&project.join(".agents").join("plugins"), "demo");

        let cfg_dir = tempfile::tempdir().unwrap();
        let config = test_config(cfg_dir.path());
        let key = project
            .join(".agents")
            .join("plugins")
            .join("demo")
            .to_string_lossy()
            .to_string();
        let entries = HashMap::from([(key, PluginConfigEntry { enabled: false })]);
        config.set_param(PLUGINS_CONFIG_KEY, entries).unwrap();

        let found = discover_with_config(project, &config);
        assert!(found.iter().all(|p| p.name != "demo"));
    }

    #[test]
    fn enabled_in_config_keeps_plugin_without_modifying_config() {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path();
        write_plugin_dir(&project.join(".agents").join("plugins"), "demo");

        let cfg_dir = tempfile::tempdir().unwrap();
        let config = test_config(cfg_dir.path());
        let key = project
            .join(".agents")
            .join("plugins")
            .join("demo")
            .to_string_lossy()
            .to_string();
        config
            .set_param(
                PLUGINS_CONFIG_KEY,
                HashMap::from([(key.clone(), PluginConfigEntry { enabled: true })]),
            )
            .unwrap();

        let found = discover_with_config(project, &config);
        assert!(found.iter().any(|p| p.name == "demo"));

        let entries: HashMap<String, PluginConfigEntry> =
            config.get_param(PLUGINS_CONFIG_KEY).unwrap();
        assert!(entries.get(&key).is_some_and(|e| e.enabled));
    }

    #[test]
    fn orders_plugins_by_scope_then_name() {
        let project = tempfile::tempdir().unwrap();
        write_plugin_dir(&project.path().join(".agents/plugins"), "z-project-plugin");
        write_plugin_dir(&project.path().join(".agents/plugins"), "a-project-plugin");
        // Project-scope plugins default closed; enable them via Local scope (a
        // real per-machine decision) so this test isolates ordering from trust.
        write_local_settings(
            &project.path().join(".config").join("pleum"),
            r#"{"enabledPlugins":["z-project-plugin","a-project-plugin"]}"#,
        );

        let path_root = tempfile::tempdir().unwrap();
        write_plugin_dir(&path_root.path().join(".agents/plugins"), "z-user-plugin");
        write_plugin_dir(&path_root.path().join(".agents/plugins"), "a-user-plugin");
        let config_dir = tempfile::tempdir().unwrap();
        let config = test_config(config_dir.path());
        let _guard = env_lock::lock_env([
            ("PLEUM_PATH_ROOT", path_root.path().to_str()),
            ("PLUGINS", None),
        ]);

        let found = discover_enabled_plugins_with_config(Some(project.path()), &config);
        let ordered: Vec<_> = found
            .into_iter()
            .map(|plugin| (plugin.name, plugin.scope))
            .collect();

        assert_eq!(
            ordered,
            vec![
                ("a-project-plugin".to_string(), PluginScope::Project),
                ("z-project-plugin".to_string(), PluginScope::Project),
                ("a-user-plugin".to_string(), PluginScope::User),
                ("z-user-plugin".to_string(), PluginScope::User),
            ]
        );
    }
}

use crate::models::Config;
use crate::vault_settings;
use std::path::PathBuf;

/// `~/.shared-docs-viewer` — the single home directory for everything this
/// viewer persists (config, recent files, diagram snapshots).
///
/// Split off workhub's `~/.workhub` on purpose: the viewer is a separate app
/// and must never read or write the workhub config (T-1001).
pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".shared-docs-viewer")
}

fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

pub fn workspaces_dir() -> PathBuf {
    config_dir().join("workspaces")
}

pub fn load() -> Config {
    let mut cfg = match std::fs::read_to_string(config_file()) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Config::default(),
    };
    migrate_default_templates(&mut cfg);
    // The configured vault's own settings win over the local copy for the
    // keys it owns (T-0206); see `vault_settings` for the split.
    vault_settings::overlay(&mut cfg);
    cfg
}

/// Rewrites saved command templates that still exactly match a previous
/// default to the current default (pwsh → Windows PowerShell). Customized
/// templates are left untouched.
fn migrate_default_templates(cfg: &mut Config) {
    const OLD_AGENT_CMD: &str = "wt -d {path} pwsh -NoExit -Command claude";
    const OLD_OPENCODE_CMD: &str = "wt -d {path} pwsh -NoExit -Command opencode";
    let defaults = crate::models::Settings::default();
    if cfg.settings.agent_cmd == OLD_AGENT_CMD {
        cfg.settings.agent_cmd = defaults.agent_cmd;
    }
    if cfg.settings.opencode_cmd == OLD_OPENCODE_CMD {
        cfg.settings.opencode_cmd = defaults.opencode_cmd;
    }
}

/// Persists `cfg` to `config.json`, returning a descriptive error instead of
/// silently discarding a write failure (T-0064: on at least one machine
/// folder-shielding antivirus software blocked every write here for over a
/// week without any visible symptom other than settings reverting).
pub fn save(cfg: &Config) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("cannot create config directory {}: {e}", dir.display()))?;
    let text =
        serde_json::to_string_pretty(cfg).map_err(|e| format!("cannot serialize config: {e}"))?;
    let file = config_file();
    std::fs::write(&file, text).map_err(|e| format!("cannot write {}: {e}", file.display()))?;
    // Mirror the vault-scoped keys into the vault so another machine that
    // clones it gets them (T-0206). The local file keeps them too, which is
    // what makes deleting the vault file a safe way back.
    vault_settings::write(cfg)
}

/// One-time migration from workhub's home to the viewer's own (T-1001).
/// Only `config.json` is carried over, so a registered document root survives
/// the move; run history and models stay with workhub. Best-effort: failures
/// are logged to stderr and never stop startup, and the old location is never
/// deleted so this is always reversible by hand. Skips entirely once the new
/// location already has a `config.json`, so it only ever runs once per
/// machine. Must run before the first `load()` call.
pub fn migrate_from_appdata() {
    let new_dir = config_dir();
    if new_dir.join("config.json").exists() {
        return;
    }
    let old_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".workhub");
    if !old_dir.join("config.json").exists() {
        return;
    }

    if let Err(e) = std::fs::create_dir_all(&new_dir) {
        crate::diag!(
            "shared-docs-viewer migration: cannot create {}: {e}",
            new_dir.display()
        );
        return;
    }

    let src = old_dir.join("config.json");
    let dst = new_dir.join("config.json");
    if let Err(e) = std::fs::copy(&src, &dst) {
        crate::diag!(
            "shared-docs-viewer migration: cannot copy {} to {}: {e}",
            src.display(),
            dst.display()
        );
    }
}

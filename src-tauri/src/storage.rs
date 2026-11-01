use crate::models::Config;
use std::path::PathBuf;

/// `~/.shared-docs-viewer` — the single home directory for everything this
/// viewer persists (config, recent files, diagram snapshots).
pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".shared-docs-viewer")
}

fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load() -> Config {
    let mut cfg = match std::fs::read_to_string(config_file()) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Config::default(),
    };
    migrate_default_templates(&mut cfg);
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
    Ok(())
}

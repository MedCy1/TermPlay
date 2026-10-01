use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub effects_volume: f32,
    pub music_volume: f32,
    pub audio_enabled: bool,
    pub music_enabled: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            effects_volume: 0.7,
            music_volume: 0.3,
            audio_enabled: true,
            music_enabled: true,
        }
    }
}

/// Niveau d'effets visuels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FxMode {
    #[default]
    Full,
    Low,
    Disabled,
}

/// Mode couleur voulu (`Auto` = détection de l'environnement).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColorPref {
    #[default]
    Auto,
    TrueColor,
    Palette256,
    Mono,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualConfig {
    pub fx_mode: FxMode,
    pub color_mode: ColorPref,
    /// Intensité du screen shake: 0, 50 ou 100 (%).
    pub shake_percent: u8,
}

impl Default for VisualConfig {
    fn default() -> Self {
        Self {
            fx_mode: FxMode::Full,
            color_mode: ColorPref::Auto,
            shake_percent: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameplayConfig {
    pub ghost_piece: bool,
    pub floating_scores: bool,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            ghost_piece: true,
            floating_scores: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GameConfig {
    pub audio: AudioConfig,
    pub visuals: VisualConfig,
    pub gameplay: GameplayConfig,
}

impl GameConfig {
    /// Applique les réglages visuels et de gameplay au moteur (les drapeaux CLI priment).
    pub fn apply(&self) {
        use crate::engine::fx;
        let level = match self.visuals.fx_mode {
            FxMode::Full => 2,
            FxMode::Low => 1,
            FxMode::Disabled => 0,
        };
        let color = match self.visuals.color_mode {
            ColorPref::Auto => 0,
            ColorPref::TrueColor => 1,
            ColorPref::Palette256 => 2,
            ColorPref::Mono => 3,
        };
        fx::set_prefs(level, color, self.visuals.shake_percent);
        fx::set_gameplay(self.gameplay.ghost_piece, self.gameplay.floating_scores);
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: GameConfig,
}

impl ConfigManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        let config = Self::load_config(&config_path)?;

        Ok(Self {
            config_path,
            config,
        })
    }

    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = base_config_dir()
            .ok_or("Could not find config directory")?
            .join("termplay");

        // Créer le répertoire s'il n'existe pas
        fs::create_dir_all(&config_dir)?;

        Ok(config_dir.join("config.json"))
    }

    fn load_config(path: &PathBuf) -> Result<GameConfig, Box<dyn std::error::Error>> {
        if path.exists() {
            let contents = fs::read_to_string(path)?;
            // Fichier vide ou corrompu (écriture interrompue, autre instance): réglages par défaut
            Ok(serde_json::from_str(&contents).unwrap_or_default())
        } else {
            // Créer la config par défaut si le fichier n'existe pas
            let default_config = GameConfig::default();
            Self::save_config_to_file(&default_config, path)?;
            Ok(default_config)
        }
    }

    fn save_config_to_file(
        config: &GameConfig,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(config)?;
        write_atomic(path, json.as_bytes())?;
        Ok(())
    }

    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        Self::save_config_to_file(&self.config, &self.config_path)
    }

    pub fn get_audio_config(&self) -> &AudioConfig {
        &self.config.audio
    }

    pub fn get(&self) -> &GameConfig {
        &self.config
    }

    /// Modifie la configuration, l'applique au moteur et la sauvegarde.
    pub fn update<F: FnOnce(&mut GameConfig)>(
        &mut self,
        updater: F,
    ) -> Result<(), Box<dyn std::error::Error>> {
        updater(&mut self.config);
        self.config.apply();
        self.save_config()
    }

    pub fn update_audio_config<F>(&mut self, updater: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut AudioConfig),
    {
        updater(&mut self.config.audio);
        self.save_config()?;
        Ok(())
    }
}

/// Écrit via un fichier temporaire puis renomme: un lecteur voit l'ancien ou le nouveau
/// contenu, jamais un fichier tronqué.
pub fn write_atomic(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(
        ".{}.{:?}.tmp",
        std::process::id(),
        std::thread::current().id()
    ));
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

/// Dossier de configuration de l'utilisateur. Sous `cargo test`, un dossier temporaire par
/// processus: les tests ne doivent jamais lire ni écrire le vrai profil (scores, réglages).
pub fn base_config_dir() -> Option<PathBuf> {
    #[cfg(test)]
    {
        use std::sync::OnceLock;
        static DIR: OnceLock<PathBuf> = OnceLock::new();
        Some(
            DIR.get_or_init(|| {
                std::env::temp_dir().join(format!("termplay-test-{}", std::process::id()))
            })
            .clone(),
        )
    }
    #[cfg(not(test))]
    {
        dirs::config_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_config_files_still_load_with_defaults() {
        let old = r#"{"audio":{"master_volume":0.5,"effects_volume":0.7,"music_volume":0.3,"audio_enabled":true,"music_enabled":false}}"#;
        let c: GameConfig = serde_json::from_str(old).unwrap();
        assert_eq!(c.audio.master_volume, 0.5);
        assert_eq!(c.visuals.fx_mode, FxMode::Full);
        assert_eq!(c.visuals.shake_percent, 100);
        assert!(c.gameplay.ghost_piece);
    }

    #[test]
    fn atomic_write_replaces_content_and_leaves_no_temp_file() {
        let dir = base_config_dir().unwrap().join("atomic");
        fs::create_dir_all(&dir).unwrap();
        let f = dir.join("x.json");
        write_atomic(&f, b"one").unwrap();
        write_atomic(&f, b"two").unwrap();
        assert_eq!(fs::read_to_string(&f).unwrap(), "two");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    }

    #[test]
    fn corrupt_config_falls_back_to_defaults() {
        let f = base_config_dir().unwrap().join("corrupt.json");
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(&f, "").unwrap();
        let c = ConfigManager::load_config(&f).unwrap();
        assert_eq!(c.audio.master_volume, AudioConfig::default().master_volume);
    }

    #[test]
    fn config_round_trips() {
        let mut c = GameConfig::default();
        c.visuals.fx_mode = FxMode::Low;
        c.visuals.color_mode = ColorPref::Palette256;
        c.visuals.shake_percent = 50;
        c.gameplay.ghost_piece = false;
        let back: GameConfig = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back.visuals.fx_mode, FxMode::Low);
        assert_eq!(back.visuals.color_mode, ColorPref::Palette256);
        assert_eq!(back.visuals.shake_percent, 50);
        assert!(!back.gameplay.ghost_piece);
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub audio: AudioConfig,
    // Ici on pourra ajouter plus tard : high_scores, game_settings, etc.
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
        }
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
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("termplay");
        
        // Créer le répertoire s'il n'existe pas
        fs::create_dir_all(&config_dir)?;
        
        Ok(config_dir.join("config.json"))
    }
    
    fn load_config(path: &PathBuf) -> Result<GameConfig, Box<dyn std::error::Error>> {
        if path.exists() {
            let contents = fs::read_to_string(path)?;
            let config: GameConfig = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            // Créer la config par défaut si le fichier n'existe pas
            let default_config = GameConfig::default();
            Self::save_config_to_file(&default_config, path)?;
            Ok(default_config)
        }
    }
    
    fn save_config_to_file(config: &GameConfig, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(config)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        Self::save_config_to_file(&self.config, &self.config_path)
    }
    
    pub fn get_audio_config(&self) -> &AudioConfig {
        &self.config.audio
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
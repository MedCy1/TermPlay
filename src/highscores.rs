use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Représente un score individuel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub player_name: String,
    pub score: u32,
    pub timestamp: DateTime<Utc>,
    pub game_data: GameData,
}

/// Données spécifiques à chaque jeu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameData {
    Snake {
        length: usize,
        duration_seconds: u64,
    },
    Tetris {
        level: u32,
        lines_cleared: u32,
        duration_seconds: u64,
    },
    Pong {
        opponent_score: u32,
        duration_seconds: u64,
    },
    Game2048 {
        highest_tile: u32,
        moves: u32,
        duration_seconds: u64,
    },
    Minesweeper {
        grid_size: (u32, u32),
        mines_count: u32,
        duration_seconds: u64,
    },
    Breakout {
        level: u32,
        bricks_broken: u32,
        duration_seconds: u64,
    },
    GameOfLife {
        generations: u32,
        duration_seconds: u64,
    },
}

/// Gère les high scores pour tous les jeux
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HighScores {
    pub games: HashMap<String, Vec<Score>>,
}

/// Manager principal pour les high scores
pub struct HighScoreManager {
    scores: HighScores,
    _config_dir: PathBuf,
    scores_file: PathBuf,
}

impl HighScoreManager {
    /// Crée un nouveau manager de high scores
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Unable to find config directory")?
            .join("termplay");

        // Créer le répertoire de configuration s'il n'existe pas
        fs::create_dir_all(&config_dir)?;

        let scores_file = config_dir.join("highscores.json");

        let scores = if scores_file.exists() {
            let content = fs::read_to_string(&scores_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HighScores::default()
        };

        Ok(Self {
            scores,
            _config_dir: config_dir,
            scores_file,
        })
    }

    /// Ajoute un nouveau score pour un jeu
    pub fn add_score(
        &mut self,
        game_name: &str,
        score: Score,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let game_scores = self.scores.games.entry(game_name.to_string()).or_default();

        // Ajouter le score
        game_scores.push(score);

        // Trier par score décroissant
        game_scores.sort_by(|a, b| b.score.cmp(&a.score));

        // Garder seulement les 10 meilleurs
        let is_top_10 = game_scores.len() <= 10;
        if game_scores.len() > 10 {
            game_scores.truncate(10);
        }

        // Sauvegarder
        self.save()?;

        Ok(is_top_10)
    }

    /// Récupère les high scores pour un jeu
    pub fn get_scores(&self, game_name: &str) -> Vec<&Score> {
        self.scores
            .games
            .get(game_name)
            .map(|scores| scores.iter().collect())
            .unwrap_or_default()
    }

    /// Récupère le meilleur score pour un jeu
    pub fn get_best_score(&self, game_name: &str) -> Option<&Score> {
        self.scores.games.get(game_name)?.first()
    }

    /// Vérifie si un score fait partie du top 10
    pub fn is_high_score(&self, game_name: &str, score: u32) -> bool {
        let game_scores = match self.scores.games.get(game_name) {
            Some(scores) => scores,
            None => return true, // Premier score = high score
        };

        if game_scores.len() < 10 {
            return true; // Moins de 10 scores = toujours high score
        }

        // Vérifier si le score est meilleur que le 10ème
        game_scores.get(9).is_none_or(|tenth| score > tenth.score)
    }

    /// Réinitialise les scores d'un jeu
    pub fn clear_game_scores(&mut self, game_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.scores.games.remove(game_name);
        self.save()
    }

    /// Réinitialise tous les scores
    #[allow(dead_code)]
    pub fn clear_all_scores(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.scores.games.clear();
        self.save()
    }

    /// Récupère la liste de tous les jeux avec des scores
    pub fn get_games_with_scores(&self) -> Vec<String> {
        self.scores.games.keys().cloned().collect()
    }

    /// Sauvegarde les scores sur disque
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.scores)?;
        fs::write(&self.scores_file, content)?;
        Ok(())
    }

    /// Recharge les scores depuis le disque
    pub fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.scores_file.exists() {
            let content = fs::read_to_string(&self.scores_file)?;
            self.scores = serde_json::from_str(&content).unwrap_or_default();
        } else {
            self.scores = HighScores::default();
        }
        Ok(())
    }

    /// Récupère le chemin du fichier de scores
    #[allow(dead_code)]
    pub fn get_scores_file_path(&self) -> &PathBuf {
        &self.scores_file
    }
}

impl Default for HighScoreManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback si on ne peut pas créer le manager
            let config_dir = PathBuf::from(".");
            let scores_file = config_dir.join("highscores.json");

            Self {
                scores: HighScores::default(),
                _config_dir: config_dir,
                scores_file,
            }
        })
    }
}

/// Helper pour créer des scores facilement
impl Score {
    pub fn new(player_name: String, score: u32, game_data: GameData) -> Self {
        Self {
            player_name,
            score,
            timestamp: Utc::now(),
            game_data,
        }
    }

    /// Formate la durée en string lisible
    pub fn format_duration(&self) -> String {
        let seconds = match &self.game_data {
            GameData::Snake {
                duration_seconds, ..
            } => *duration_seconds,
            GameData::Tetris {
                duration_seconds, ..
            } => *duration_seconds,
            GameData::Pong {
                duration_seconds, ..
            } => *duration_seconds,
            GameData::Game2048 {
                duration_seconds, ..
            } => *duration_seconds,
            GameData::Minesweeper {
                duration_seconds, ..
            } => *duration_seconds,
            GameData::Breakout {
                duration_seconds, ..
            } => *duration_seconds,
            GameData::GameOfLife {
                duration_seconds, ..
            } => *duration_seconds,
        };

        let minutes = seconds / 60;
        let seconds = seconds % 60;

        if minutes > 0 {
            format!("{minutes}m {seconds}s")
        } else {
            format!("{seconds}s")
        }
    }

    /// Formate la date en string lisible
    pub fn format_date(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M").to_string()
    }
}

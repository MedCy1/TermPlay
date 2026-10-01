//! Écran Settings: modèle des lignes (par onglet) et rendu. La logique de modification vit
//! dans `MainMenu` (il possède l'audio et la configuration persistante).
use crate::config::{ColorPref, FxMode, GameConfig};
use crate::engine::fx::{self, Rgb};
use crate::menu_ui;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub const TABS: [&str; 3] = [
    "🎨 Visuals & Graphics",
    "🔊 Audio & Feedback",
    "🎮 Gameplay",
];
const ACCENTS: [Rgb; 3] = [(190, 90, 235), (60, 190, 255), (255, 170, 50)];

pub const FX_OPTIONS: [&str; 3] = ["Full (60 FPS)", "Low FX", "Disabled (--no-fx)"];
pub const COLOR_OPTIONS: [&str; 4] = [
    "Auto-detect",
    "24-bit TrueColor",
    "256 Colors",
    "Monochrome",
];
pub const SHAKE_OPTIONS: [&str; 3] = ["0 % (off)", "50 %", "100 %"];

pub enum Value {
    Choice {
        options: &'static [&'static str],
        index: usize,
    },
    Slider(f32),
    Toggle(bool),
}

pub struct Row {
    pub label: &'static str,
    pub value: Value,
    /// Remarque affichée en gris (ex: réglage imposé par un drapeau CLI).
    pub note: Option<&'static str>,
}

pub fn row_count(tab: usize) -> usize {
    [3, 5, 2][tab.min(2)]
}

pub fn fx_index(c: &GameConfig) -> usize {
    match c.visuals.fx_mode {
        FxMode::Full => 0,
        FxMode::Low => 1,
        FxMode::Disabled => 2,
    }
}

pub fn color_index(c: &GameConfig) -> usize {
    match c.visuals.color_mode {
        ColorPref::Auto => 0,
        ColorPref::TrueColor => 1,
        ColorPref::Palette256 => 2,
        ColorPref::Mono => 3,
    }
}

pub fn shake_index(c: &GameConfig) -> usize {
    match c.visuals.shake_percent {
        0 => 0,
        1..=74 => 1,
        _ => 2,
    }
}

/// Lignes d'un onglet; `audio` = (master, effets, musique, effets actifs, musique active).
pub fn rows(tab: usize, cfg: &GameConfig, audio: (f32, f32, f32, bool, bool)) -> Vec<Row> {
    let (forced_fx, forced_color) = fx::forced();
    match tab {
        0 => vec![
            Row {
                label: "Effects",
                value: Value::Choice {
                    options: &FX_OPTIONS,
                    index: fx_index(cfg),
                },
                note: forced_fx.then_some("forced by --no-fx / NO_COLOR"),
            },
            Row {
                label: "Colors",
                value: Value::Choice {
                    options: &COLOR_OPTIONS,
                    index: color_index(cfg),
                },
                note: forced_color.then_some("forced by --no-color / NO_COLOR"),
            },
            Row {
                label: "Screen shake",
                value: Value::Choice {
                    options: &SHAKE_OPTIONS,
                    index: shake_index(cfg),
                },
                note: None,
            },
        ],
        1 => vec![
            Row {
                label: "Master volume",
                value: Value::Slider(audio.0),
                note: None,
            },
            Row {
                label: "Effects volume",
                value: Value::Slider(audio.1),
                note: None,
            },
            Row {
                label: "Music volume",
                value: Value::Slider(audio.2),
                note: None,
            },
            Row {
                label: "Sound effects",
                value: Value::Toggle(audio.3),
                note: None,
            },
            Row {
                label: "Music",
                value: Value::Toggle(audio.4),
                note: None,
            },
        ],
        _ => vec![
            Row {
                label: "Tetris ghost piece",
                value: Value::Toggle(cfg.gameplay.ghost_piece),
                note: None,
            },
            Row {
                label: "2048 floating scores",
                value: Value::Toggle(cfg.gameplay.floating_scores),
                note: None,
            },
        ],
    }
}

/// `━━━━━━●─────  60%`: trait fin dégradé vert → jaune, curseur `●`, reste en trait sombre.
fn slider_spans(v: f32) -> Vec<Span<'static>> {
    const W: usize = 14;
    let v = v.clamp(0.0, 1.0);
    let pos = (v * (W - 1) as f32).round() as usize;
    let mut spans = Vec::with_capacity(W + 1);
    for i in 0..W {
        spans.push(if i < pos {
            let c = fx::lerp((80, 230, 120), (255, 200, 60), i as f32 / (W - 1) as f32);
            Span::styled("━", Style::new().fg(fx::color(c)))
        } else if i == pos {
            let c = fx::lerp((80, 230, 120), (255, 200, 60), v);
            Span::styled("●", Style::new().fg(fx::color(c)).bold())
        } else {
            Span::styled("─", Style::new().fg(Color::Rgb(60, 70, 85)))
        });
    }
    spans.push(Span::styled(
        format!("  {:>3}%", (v * 100.0).round() as u32),
        Style::new().white().bold(),
    ));
    spans
}

fn toggle_spans(on: bool) -> Vec<Span<'static>> {
    let lit_on = Style::new()
        .fg(Color::Black)
        .bg(Color::Rgb(80, 230, 120))
        .bold();
    let lit_off = Style::new()
        .fg(Color::Black)
        .bg(Color::Rgb(235, 80, 80))
        .bold();
    let dim = Style::new().fg(Color::Rgb(90, 100, 115));
    vec![
        Span::styled("[ON]", if on { lit_on } else { dim }),
        Span::raw(" "),
        Span::styled("[OFF]", if on { dim } else { lit_off }),
    ]
}

fn row_line(r: &Row, selected: bool) -> Line<'static> {
    let mut spans = vec![Span::styled(
        format!("{:<22}", r.label),
        Style::new().white().bold(),
    )];
    match &r.value {
        Value::Slider(v) => spans.extend(slider_spans(*v)),
        Value::Toggle(on) => spans.extend(toggle_spans(*on)),
        Value::Choice { options, index } => {
            let arrow = |c: &'static str| {
                Span::styled(
                    c,
                    Style::new().fg(if selected {
                        Color::Yellow
                    } else {
                        Color::Rgb(90, 100, 115)
                    }),
                )
            };
            spans.push(arrow("◂ "));
            spans.push(Span::styled(
                options[(*index).min(options.len() - 1)],
                Style::new().fg(Color::Cyan).bold(),
            ));
            spans.push(arrow(" ▸"));
        }
    }
    if let Some(n) = r.note {
        spans.push(Span::styled(
            format!("  ({n})"),
            Style::new().fg(Color::DarkGray),
        ));
    }
    Line::from(spans)
}

#[allow(clippy::too_many_arguments)]
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    tab: usize,
    rows: &[Row],
    selected: usize,
    sel_pos: f32,
    time: f32,
) {
    let tab = tab.min(2);
    let accent = ACCENTS[tab];
    let [tabs_area, list_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

    let mut spans = vec![Span::raw(" ")];
    for (i, name) in TABS.iter().enumerate() {
        let style = if i == tab {
            Style::new()
                .fg(Color::Black)
                .bg(fx::color(ACCENTS[i]))
                .bold()
        } else {
            Style::new().fg(Color::Gray)
        };
        spans.push(Span::styled(format!(" {name} "), style));
        spans.push(Span::raw("  "));
    }
    spans.push(Span::styled("Tab ⇥", Style::new().fg(Color::DarkGray)));
    let tabs = Paragraph::new(Line::from(spans)).block(
        Block::bordered()
            .border_style(Style::new().fg(fx::color(accent)))
            .style(Style::default().bg(Color::Rgb(15, 22, 32))),
    );
    frame.render_widget(tabs, tabs_area);

    let lines: Vec<Line<'static>> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| row_line(r, i == selected))
        .collect();
    menu_ui::draw_selector(
        frame,
        list_area,
        &format!(" {} ", TABS[tab]),
        Color::Rgb(accent.0, accent.1, accent.2),
        accent,
        &lines,
        selected,
        sel_pos,
        time,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_counts_match_rows() {
        let cfg = GameConfig::default();
        for tab in 0..3 {
            assert_eq!(
                rows(tab, &cfg, (0.5, 0.5, 0.5, true, true)).len(),
                row_count(tab)
            );
        }
    }

    #[test]
    fn slider_shows_percentage_and_bar() {
        let text: String = slider_spans(0.6)
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(text, "━━━━━━━━●─────   60%");
        let zero: String = slider_spans(0.0)
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(zero.starts_with("●─"));
        let full: String = slider_spans(1.0)
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(full.ends_with("100%"));
    }

    #[test]
    fn shake_index_buckets() {
        let mut c = GameConfig::default();
        for (pct, idx) in [(0, 0), (50, 1), (100, 2)] {
            c.visuals.shake_percent = pct;
            assert_eq!(shake_index(&c), idx);
        }
    }
}

//! Rendu animé du menu: bannière à vague de couleur, sélecteur qui glisse, carte de jeu,
//! poussières d'ambiance. Écrit directement dans le buffer, sans allocation par frame.
use crate::engine::fx::{self, Rgb};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

const GLYPHS: [(char, [&str; 5]); 8] = [
    ('T', ["█████", "  █  ", "  █  ", "  █  ", "  █  "]),
    ('E', ["█████", "█    ", "████ ", "█    ", "█████"]),
    ('R', ["████ ", "█   █", "████ ", "█  █ ", "█   █"]),
    ('M', ["█   █", "██ ██", "█ █ █", "█   █", "█   █"]),
    ('P', ["████ ", "█   █", "████ ", "█    ", "█    "]),
    ('L', ["█    ", "█    ", "█    ", "█    ", "█████"]),
    ('A', [" ███ ", "█   █", "█████", "█   █", "█   █"]),
    ('Y', ["█   █", " █ █ ", "  █  ", "  █  ", "  █  "]),
];
const WORD: &str = "TERMPLAY";
pub const BANNER_W: u16 = 47; // 8 lettres de 5 colonnes + 7 espaces
pub const BANNER_H: u16 = 5;

fn glyph(c: char) -> &'static [&'static str; 5] {
    &GLYPHS.iter().find(|g| g.0 == c).unwrap_or(&GLYPHS[0]).1
}

/// Couleur de la vague à la colonne `x`: va-et-vient lent cyan ↔ magenta, de gauche à droite.
pub fn wave_color(x: f32, row: usize, time: f32) -> Rgb {
    let t = 0.5 + 0.5 * (time * 1.4 - x * 0.13).sin();
    let c = fx::lerp((70, 200, 255), (255, 95, 220), t);
    fx::lerp(c, (20, 30, 60), row as f32 * 0.08)
}

/// Bannière centrée dans `area` (au moins BANNER_W × BANNER_H), lueur discrète derrière.
pub fn draw_banner(buf: &mut Buffer, area: Rect, time: f32) {
    if area.width < BANNER_W || area.height < BANNER_H {
        return;
    }
    let x0 = area.x as i32 + (area.width - BANNER_W) as i32 / 2;
    let y0 = area.y as i32;
    fx::glow(
        buf,
        x0 + BANNER_W as i32 / 2,
        y0 + 2,
        7,
        wave_color(BANNER_W as f32 / 2.0, 0, time),
        0.22,
    );
    let mut x = x0;
    for ch in WORD.chars() {
        for (row, line) in glyph(ch).iter().enumerate() {
            for (i, g) in line.chars().enumerate() {
                if g != ' ' {
                    let col = (x - x0) as f32 + i as f32;
                    fx::put(
                        buf,
                        x + i as i32,
                        y0 + row as i32,
                        g,
                        wave_color(col, row, time),
                    );
                }
            }
        }
        x += 6;
    }
}

const MOTES: usize = 36;

/// Poussières d'étoiles qui montent lentement. Position calculée à partir du temps
/// (pas d'état, pas d'aléa); uniquement sur des cellules vides pour ne jamais gêner le texte.
pub fn draw_motes(buf: &mut Buffer, area: Rect, time: f32) {
    if !fx::fx_enabled() || fx::fx_low() || area.width < 4 || area.height < 4 {
        return;
    }
    for i in 0..MOTES {
        let xf = ((i * 37) % 97) as f32 / 97.0;
        let speed = 0.02 + ((i * 53) % 11) as f32 * 0.004;
        let phase = ((i * 29) % 101) as f32 / 101.0;
        let life = (time * speed + phase).fract(); // 0 → 1 pendant la montée
        let fade = (life * std::f32::consts::PI).sin();
        let x =
            area.x as f32 + xf * (area.width - 1) as f32 + (time * 0.5 + phase * 6.0).sin() * 1.5;
        let y = area.y as f32 + (1.0 - life) * (area.height - 1) as f32;
        let (x, y) = (x.round() as i32, y.round() as i32);
        if let Some(c) = fx::cell(buf, x, y) {
            if c.symbol() == " " {
                let ch = if i % 3 == 0 { '*' } else { '·' };
                c.set_char(ch).set_fg(fx::color(fx::lerp(
                    (22, 30, 42),
                    (110, 135, 175),
                    fade * 0.8,
                )));
            }
        }
    }
}

/// Liste verticale avec barre de sélection qui glisse (`pos` fractionnaire), halo sur les
/// lignes voisines et chevron pulsant.
#[allow(clippy::too_many_arguments)]
pub fn draw_selector(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    border: Color,
    accent: Rgb,
    rows: &[Line<'static>],
    selected: usize,
    pos: f32,
    time: f32,
) {
    let block = Block::bordered()
        .title(title.to_string().white().bold())
        .border_style(Style::new().fg(border))
        .style(Style::default().bg(Color::Rgb(10, 15, 20)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 5 || inner.height == 0 || rows.is_empty() {
        return;
    }
    let h = inner.height as usize;
    let offset = selected.saturating_sub(h - 1);
    let rel = pos - offset as f32;
    let animated = fx::fx_enabled();

    let buf = frame.buffer_mut();
    for r in 0..h {
        let dist = (r as f32 - rel).abs();
        let bar = (1.0 - dist).max(0.0);
        let halo = if animated {
            (1.0 - dist / 3.0).max(0.0) * 0.1
        } else {
            0.0
        };
        let strength = 0.5 * bar + halo;
        if strength <= 0.0 {
            continue;
        }
        for dx in 0..inner.width {
            let k = strength * (1.0 - 0.5 * dx as f32 / inner.width as f32);
            if let Some(c) = fx::cell(buf, (inner.x + dx) as i32, (inner.y + r as u16) as i32) {
                c.set_bg(fx::color(fx::lerp((10, 15, 20), accent, k)));
            }
        }
    }
    let ry = rel.round().clamp(0.0, (h - 1) as f32) as i32;
    let pulse = if animated {
        0.5 + 0.5 * (time * 6.0).sin()
    } else {
        1.0
    };
    let nudge = i32::from(animated && pulse > 0.75);
    fx::put(
        buf,
        inner.x as i32 + 1 + nudge,
        inner.y as i32 + ry,
        '❯',
        fx::lerp(accent, (255, 255, 255), pulse),
    );

    for r in 0..h {
        let idx = offset + r;
        let Some(line) = rows.get(idx) else { break };
        let mut p = Paragraph::new(line.clone());
        if idx == selected {
            p = p.style(Style::new().add_modifier(Modifier::BOLD));
        }
        let rect = Rect::new(inner.x + 3, inner.y + r as u16, inner.width - 3, 1);
        frame.render_widget(p, rect);
    }
}

/// (icône, couleur d'accent, touches) d'un jeu, d'après sa clé (`games::key`).
pub fn game_meta(key: &str) -> (&'static str, Rgb, &'static str) {
    match key {
        "snake" => ("🐍", (80, 230, 120), "←↑↓→ Steer   Q Quit"),
        "tetris" => (
            "🧩",
            (190, 90, 235),
            "←→ Move   ↑ Rotate   ↓ Soft drop   Space Hard drop",
        ),
        "pong" => ("🏓", (90, 200, 255), "W/S Player 1   ↑/↓ Player 2"),
        "2048" => ("🔢", (255, 170, 50), "Arrows / WASD Slide tiles"),
        "minesweeper" => ("💣", (255, 90, 90), "Arrows Move   Space Reveal   F Flag"),
        "breakout" => ("🧱", (240, 200, 60), "←→ Move paddle   Space Launch"),
        "gameoflife" => (
            "🧬",
            (100, 240, 200),
            "Space Toggle   P Play   1-6 Patterns",
        ),
        _ => ("🎮", (150, 170, 255), "Arrow keys"),
    }
}

/// Carte de prévisualisation du jeu survolé; la bordure s'éclaire (`flash` 0→1) à chaque changement.
#[allow(clippy::too_many_arguments)]
pub fn draw_card(
    frame: &mut Frame,
    area: Rect,
    key: &str,
    name: &str,
    description: &str,
    best: Option<u32>,
    flash: f32,
    time: f32,
) {
    let (icon, accent, controls) = game_meta(key);
    let border = fx::color(fx::lerp((255, 255, 255), accent, flash));
    let block = Block::bordered()
        .title(" Preview ".white().bold())
        .border_style(Style::new().fg(border))
        .style(Style::default().bg(Color::Rgb(10, 15, 20)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 8 || inner.height < 3 {
        return;
    }
    let best_line = match best {
        Some(s) => Span::styled(s.to_string(), Style::new().green().bold()),
        None => Span::styled("no score yet", Style::new().dark_gray()),
    };
    let pulse = if fx::fx_enabled() {
        0.5 + 0.5 * (time * 4.0).sin()
    } else {
        1.0
    };
    let lines = vec![
        Line::from(Span::styled(
            format!(" {icon} {} ", name.to_uppercase()),
            Style::new().bg(fx::color(accent)).fg(Color::Black).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            description.to_string(),
            Style::new().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(Span::styled("Controls", Style::new().yellow().bold())),
        Line::from(Span::styled(controls, Style::new().fg(Color::White))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Best score  ", Style::new().yellow().bold()),
            best_line,
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Enter ❯ play",
            Style::new()
                .fg(fx::color(fx::lerp((90, 100, 120), accent, pulse)))
                .bold(),
        )),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_glyphs_are_rectangular() {
        for (c, rows) in GLYPHS {
            assert!(rows.iter().all(|r| r.chars().count() == 5), "{c}");
        }
        assert_eq!(
            WORD.chars().count() as u16 * 5 + (WORD.len() as u16 - 1),
            BANNER_W
        );
    }

    #[test]
    fn wave_moves_with_time_and_position() {
        assert_ne!(wave_color(0.0, 0, 0.0), wave_color(20.0, 0, 0.0));
        assert_ne!(wave_color(5.0, 0, 0.0), wave_color(5.0, 0, 1.0));
    }

    #[test]
    fn motes_only_use_blank_cells() {
        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::filled(area, ratatui::buffer::Cell::new("x"));
        draw_motes(&mut buf, area, 3.0);
        assert!(buf.content().iter().all(|c| c.symbol() == "x"));
        let mut buf = Buffer::empty(area);
        draw_motes(&mut buf, area, 3.0);
        assert!(buf.content().iter().any(|c| c.symbol() != " "));
    }
}

use ratatui::style::{Color, Modifier, Style};

// paperboy palette — mirrors newtab.css CSS variables
pub const BG: Color = Color::Rgb(40, 45, 28);       // --bg
pub const BG2: Color = Color::Rgb(54, 60, 38);      // --bg-2
pub const BG3: Color = Color::Rgb(64, 72, 52);      // --bg-3
pub const SURFACE: Color = Color::Rgb(58, 66, 48);  // --surface

pub const BORDER: Color = Color::Rgb(79, 91, 74);   // --border
pub const BORDER2: Color = Color::Rgb(90, 106, 84); // --border-2 (focused)
pub const BORDER3: Color = Color::Rgb(122, 133, 115);

pub const TX: Color = Color::Rgb(220, 224, 217);    // --tx
pub const TX2: Color = Color::Rgb(168, 176, 159);   // --tx-2
pub const TX3: Color = Color::Rgb(122, 133, 115);   // --tx-3 (muted)
pub const TX4: Color = Color::Rgb(102, 110, 96);    // --tx-4 (very dim)

pub const ACCENT: Color = Color::Rgb(212, 160, 51); // --accent (amber)
pub const GREEN: Color = Color::Rgb(122, 158, 56);  // --green
pub const RED: Color = Color::Rgb(194, 93, 68);     // --red
pub const ORANGE: Color = Color::Rgb(192, 144, 96); // --orange

pub fn border(focused: bool) -> Style {
    if focused {
        Style::default().fg(BORDER2)
    } else {
        Style::default().fg(BORDER)
    }
}

pub fn highlight() -> Style {
    Style::default()
        .fg(BG)
        .bg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

//! Dark theme:
//! #000814
//! #001d3d
//! #003566
//! #ffc300
//! #ffd60a

use iced::{ border::{ Border, Radius }, widget::{button, container}, Background, Color, Theme };

/// #000814
pub const COLOR_DARK_0: Color = Color { r: 0.0, g: 0.031, b: 0.078, a: 1.0 };

/// #001d3d
pub const COLOR_DARK_1: Color = Color { r: 0.0, g: 0.114, b: 0.239, a: 1.0 };

/// #003566
pub const COLOR_DARK_2: Color = Color { r: 0.0, g: 0.208, b: 0.4, a: 1.0 };

/// #ffc300
pub const COLOR_DARK_3: Color = Color { r: 1.0, g: 0.765, b: 0.0, a: 1.0 };

/// #ffd60a
pub const COLOR_DARK_4: Color = Color { r: 1.0, g: 0.839, b: 0.039, a: 1.0 };

pub const ZERO_RADIUS: Radius = Radius {
    top_right: 0.0, bottom_right: 0.0, bottom_left: 0.0, top_left: 0.0,
};

pub const EXPLORER_BG_COLOR: Color = Color {
    r: 0.0, g: 0.063, b: 0.157, a: 1.0
};

pub const EXPLORER_BG_HOVER_COLOR: Color = Color {
    r: 0.0, g: 0.094, b: 0.235, a: 1.0
};

pub struct AppStyles {
    pub editor_bg_color: Color,
    pub editor_bg: container::Style,
    pub editor_border: Border,

    /// Editor selection (highlight) color
    pub editor_sel_color: Color,

    pub explorer_bg_color: Background,

    pub explorer_notes: button::Style,
    pub explorer_notes_hovered: button::Style,
}

pub fn get_app_styles(theme: &Theme) -> AppStyles {
    if *theme == Theme::Dark {
        return AppStyles {
            editor_bg_color: COLOR_DARK_0,
            editor_bg: container::Style {
                background: Some(Background::Color(COLOR_DARK_0)),
                ..container::Style::default()
            },
            editor_border: Border {
                color: COLOR_DARK_0,
                width: 0.0,
                radius: ZERO_RADIUS,
            },
            editor_sel_color: COLOR_DARK_2,
            explorer_bg_color: Background::Color(EXPLORER_BG_COLOR),
            explorer_notes: button::Style {
                background: Some(Background::Color(EXPLORER_BG_COLOR)),
                text_color: Color::WHITE,
                ..button::Style::default()
            },
            explorer_notes_hovered: button::Style {
                background: Some(Background::Color(EXPLORER_BG_HOVER_COLOR)),
                text_color: Color::WHITE,
                ..button::Style::default()
            },
        };
    }

    AppStyles {
        editor_bg_color: Color::BLACK,
        editor_bg: container::Style::default(),
        editor_border: Border {
            color: Color::BLACK,
            width: 0.0,
            radius: ZERO_RADIUS,
        },
        editor_sel_color: Color::BLACK,
        explorer_bg_color: Background::Color(Color::BLACK),
        explorer_notes: button::Style::default(),
        explorer_notes_hovered: button::Style::default(),
    }
}


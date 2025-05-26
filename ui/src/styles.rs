use iced::{ Color, border::{ Border, Radius } };

pub const EDITOR_BACKGROUND: Color = Color {
    r: 0.054,
    g: 0.1,
    b: 0.14,
    a: 1.0,
};

pub const EDITOR_BORDER: Border = Border {
    color: EDITOR_BACKGROUND,
    width: 0.0,
    radius: Radius {
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
        top_left: 0.0,
    },
};


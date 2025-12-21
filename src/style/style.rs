use iced::{
    Background, Color, Theme, border,
    widget::{button, container},
};

pub fn container_active(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.2, 0.7, 0.3))),
        border: border::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn container_inactive(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.85, 0.85, 0.85))),
        border: border::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn cancel_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
            text_color: Color::WHITE,
            ..Default::default()
        },
        _ => button::Style::default(),
    }
}

pub fn validate_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.7, 0.3))),
            text_color: Color::WHITE,
            ..Default::default()
        },
        _ => button::Style::default(),
    }
}

use iced::{
    Background, Border, Color, Theme, border,
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
        text_color: Some(Color::BLACK),
        border: border::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn drop_zone_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.5, 0.7, 1.0, 0.3))),
        border: iced::Border {
            color: Color::from_rgb(0.5, 0.7, 1.0),
            width: 2.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

pub fn transparent_button_style(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: None,
        border: iced::Border::default(),
        ..Default::default()
    }
}
pub fn container_drag(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.85, 0.92, 1.0))),
        text_color: Some(Color::BLACK),
        border: border::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn container_global_deaths(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.15, 0.05, 0.05))),
        border: Border {
            color: Color::from_rgb(0.8, 0.2, 0.2),
            width: 3.0,
            radius: 8.0.into(),
        },
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

pub fn container_global_bosses(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.15, 0.1, 0.05))),
        border: Border {
            color: Color::from_rgb(0.9, 0.6, 0.2),
            width: 3.0,
            radius: 8.0.into(),
        },
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

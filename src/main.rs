#![feature(once_cell)]
#![windows_subsystem = "windows"]

use crate::app::App;
use iced::{Application, Settings};
use std::sync::OnceLock;

mod app;

/// select a md file
/// select a directory which save the images used in the md file
/// find all the images used in the md file
/// remove the images which are not used in the md file
fn main() -> iced::Result {
    App::run(Settings {
        window: iced::window::Settings {
            size: (800, 400),
            ..iced::window::Settings::default()
        },
        default_font: font(),
        ..Settings::default()
    })
}

static FONT: OnceLock<Option<Vec<u8>>> = OnceLock::new();

fn font() -> Option<&'static [u8]> {
    FONT.get_or_init(|| {
        use iced_graphics::font::Family;
        let source = iced_graphics::font::Source::new();
        source
            .load(&[
                Family::Title("PingFang SC".to_owned()),
                Family::Title("Hiragino Sans GB".to_owned()),
                Family::Title("Heiti SC".to_owned()),
                Family::Title("Microsoft YaHei".to_owned()),
                Family::Title("WenQuanYi Micro Hei".to_owned()),
                Family::Title("Microsoft YaHei".to_owned()),
                Family::Title("Helvetica".to_owned()),
                Family::Title("Tahoma".to_owned()),
                Family::Title("Arial".to_owned()),
                Family::SansSerif,
            ])
            .ok()
    })
    .as_ref()
    .map(|f| f.as_slice())
}

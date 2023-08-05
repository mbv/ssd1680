//! SSD1680 ePaper Display Driver
//!
//! Used in the [WeAct 2.13" Tri-Color display](https://www.aliexpress.com/item/1005004644515880.html)
//!
//! For a complete example see [the example](https://github.com/mbv/esp32-ssd1680/blob/main/src/main.rs).
//!
//! This driver is losely modeled after the
//! [epd-waveshare](https://github.com/caemor/epd-waveshare) drivers but built for my needs.
//!
//!
//! ### Usage
//! This driver does not hide that you're working with one buffer for black/white and one for red. To
//! display something you:
//!
//! 1. first create a buffer (either b/w or red) and draw things onto it, preferably
//! with [`embedded_graphics`](https://github.com/jamwaffles/embedded-graphics).
//! 1. then send the frame to the display driver using [`driver::Ssd1680::update_bw_frame`] or
//!    [`driver::Ssd1680::update_red_frame`]
//! 1. then kick off a display update using [`driver::Ssd1680::display_frame`]
//!
//!
#![no_std]
#![deny(missing_docs)]
#![allow(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

pub mod color;
pub mod driver;

#[cfg(feature = "graphics")]
pub mod graphics;
mod cmd;
mod flag;

/// Maximum display height this driver supports
pub const MAX_HEIGHT: u16 = 296;

/// Maximum display width this driver supports
pub const MAX_WIDTH: u16 = 176;

/// Display height
pub const HEIGHT: u8 = 250;

/// Display width
pub const WIDTH: u8 = 122;

pub mod interface;

/// Useful exports
pub mod prelude {
    pub use crate::color::Color;
    pub use crate::driver::Ssd1680;

    #[cfg(feature = "graphics")]
    pub use crate::graphics::{Display, Display2in13, DisplayRotation};
}

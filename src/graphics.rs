//! Graphics Support for EPDs

use crate::color::Color;
use crate::{HEIGHT, WIDTH};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};

/// Displayrotation
#[derive(Clone, Copy)]
pub enum DisplayRotation {
    /// No rotation
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}

impl Default for DisplayRotation {
    fn default() -> Self {
        DisplayRotation::Rotate0
    }
}

/// Necessary traits for all displays to implement for drawing
///
/// Adds support for:
/// - Drawing (With the help of DrawTarget/Embedded Graphics)
/// - Rotations
/// - Clearing
pub trait Display: DrawTarget<BinaryColor> {
    /// Clears the buffer of the display with the chosen background color
    fn clear_buffer(&mut self, background_color: Color) {
        let fill_color = if self.is_inverted() {
            background_color.inverse().get_byte_value()
        } else {
            background_color.get_byte_value()
        };

        for elem in self.get_mut_buffer().iter_mut() {
            *elem = fill_color
        }
    }

    /// Returns the buffer
    fn buffer(&self) -> &[u8];

    /// Returns a mutable buffer
    fn get_mut_buffer(&mut self) -> &mut [u8];

    /// Sets the rotation of the display
    fn set_rotation(&mut self, rotation: DisplayRotation);

    /// Get the current rotation of the display
    fn rotation(&self) -> DisplayRotation;

    /// If the color for this display is inverted
    fn is_inverted(&self) -> bool;

    /// Helperfunction for the Embedded Graphics draw trait
    ///
    /// Becomes uneccesary when `const_generics` become stablised
    fn draw_helper(
        &mut self,
        width: u32,
        height: u32,
        pixel: Pixel<BinaryColor>,
    ) -> Result<(), Self::Error> {
        let rotation = self.rotation();
        let is_inverted = self.is_inverted();
        let buffer = self.get_mut_buffer();

        let Pixel(point, color) = pixel;
        if outside_display(point, width, height, rotation) {
            return Ok(());
        }

        // Give us index inside the buffer and the bit-position in that u8 which needs to be changed
        let (index, bit) = find_position(point.x as u32, point.y as u32, width, height, rotation);
        let index = index as usize;

        // "Draw" the Pixel on that bit
        match color {
            // Black/Red
            BinaryColor::On => {
                if is_inverted {
                    buffer[index] &= !bit;
                } else {
                    buffer[index] |= bit;
                }
            }
            // White
            BinaryColor::Off => {
                if is_inverted {
                    buffer[index] |= bit;
                } else {
                    buffer[index] &= !bit;
                }
            }
        }
        Ok(())
    }
}

/// Display for a 122x250 panel
pub struct Display2in13 {
    buffer: [u8;  buffer_len(WIDTH as usize, HEIGHT as usize)],
    rotation: DisplayRotation,
    is_inverted: bool,
}

impl Display2in13 {
    /// Create a black & white display buffer
    pub fn bw() -> Self {
        Display2in13 {
            buffer: [Color::White.get_byte_value(); buffer_len(WIDTH as usize, HEIGHT as usize)],
            rotation: DisplayRotation::default(),
            is_inverted: true,
        }
    }

    /// Create a red display buffer
    pub fn red() -> Self {
        Display2in13 {
            buffer: [Color::White.inverse().get_byte_value();
                buffer_len(WIDTH as usize, HEIGHT as usize)],
            rotation: DisplayRotation::default(),
            is_inverted: false,
        }
    }
}

impl DrawTarget<BinaryColor> for Display2in13 {
    type Error = core::convert::Infallible;

    fn draw_pixel(&mut self, pixel: Pixel<BinaryColor>) -> Result<(), Self::Error> {
        self.draw_helper(u32::from(WIDTH), u32::from(HEIGHT), pixel)
    }

    fn size(&self) -> Size {
        Size::new(u32::from(WIDTH), u32::from(HEIGHT))
    }
}

impl Display for Display2in13 {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn get_mut_buffer(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.rotation = rotation;
    }

    fn rotation(&self) -> DisplayRotation {
        self.rotation
    }

    fn is_inverted(&self) -> bool {
        self.is_inverted
    }
}

// Checks if a pos is outside the defined display
fn outside_display(p: Point, width: u32, height: u32, rotation: DisplayRotation) -> bool {
    if p.x < 0 || p.y < 0 {
        return true;
    }
    let (x, y) = (p.x as u32, p.y as u32);
    match rotation {
        DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
            if x >= width || y >= height {
                return true;
            }
        }
        DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
            if y >= width || x >= height {
                return true;
            }
        }
    }
    false
}

fn find_rotation(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u32) {
    let nx;
    let ny;
    match rotation {
        DisplayRotation::Rotate0 => {
            nx = x;
            ny = y;
        }
        DisplayRotation::Rotate90 => {
            nx = width - 1 - y;
            ny = x;
        }
        DisplayRotation::Rotate180 => {
            nx = width - 1 - x;
            ny = height - 1 - y;
        }
        DisplayRotation::Rotate270 => {
            nx = y;
            ny = height - 1 - x;
        }
    }
    (nx, ny)
}

#[rustfmt::skip]
//returns index position in the u8-slice and the bit-position inside that u8
fn find_position(x: u32, y: u32, width: u32, height: u32, rotation: DisplayRotation) -> (u32, u8) {
    let (nx, ny) = find_rotation(x, y, width, height, rotation);
    (
        nx / 8 + ((width + 7) / 8) * ny,
        0x80 >> (nx % 8),
    )
}

/// Computes the needed buffer length. Takes care of rounding up in case width
/// is not divisible by 8.
#[must_use]
const fn buffer_len(width: usize, height: usize) -> usize {
    (width + 7) / 8 * height
}

#[cfg(test)]
mod tests {
    use super::{find_position, outside_display, Display, Display2in13, DisplayRotation};
    use crate::color::Black;
    use crate::color::Color;
    use embedded_graphics::{prelude::*, primitives::Line, style::PrimitiveStyle};

    #[test]
    fn buffer_clear() {
        let mut display = Display2in13::bw();

        for &byte in display.buffer.iter() {
            assert_eq!(byte, Color::White.get_byte_value());
        }

        display.clear_buffer(Color::White);

        for &byte in display.buffer.iter() {
            assert_eq!(byte, Color::Black.get_byte_value());
        }
    }

    #[test]
    fn rotation_overflow() {
        use crate::{HEIGHT, WIDTH};
        let width = WIDTH as u32;
        let height = HEIGHT as u32;
        test_rotation_overflow(width, height, DisplayRotation::Rotate0);
        test_rotation_overflow(width, height, DisplayRotation::Rotate90);
        test_rotation_overflow(width, height, DisplayRotation::Rotate180);
        test_rotation_overflow(width, height, DisplayRotation::Rotate270);
    }

    fn test_rotation_overflow(width: u32, height: u32, rotation2: DisplayRotation) {
        let max_value = width / 8 * height;
        for x in 0..(width + height) {
            //limit x because it runs too long
            for y in 0..(u32::max_value()) {
                if outside_display(Point::new(x as i32, y as i32), width, height, rotation2) {
                    break;
                } else {
                    let (idx, _) = find_position(x, y, width, height, rotation2);
                    assert!(idx < max_value);
                }
            }
        }
    }

    #[test]
    fn graphics_rotation_0() {
        let mut display = Display2in13::bw();

        let _ = Line::new(Point::new(0, 0), Point::new(7, 0))
            .into_styled(PrimitiveStyle::with_stroke(Black, 1))
            .draw(&mut display);

        let buffer = display.buffer();

        assert_eq!(buffer[0], Color::Black.get_byte_value());

        for &byte in buffer.iter().skip(1) {
            assert_eq!(byte, Color::White.get_byte_value());
        }
    }
}

//! Driver for interacting with SSD1680 display driver
use core::fmt::Debug;

use embedded_hal::{
    blocking::{delay::DelayMs, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

use crate::interface::DisplayInterface;
use crate::{cmd, color, flag, HEIGHT, WIDTH};

/// A configured display with a hardware interface.
pub struct Ssd1680<SPI, CS, BUSY, DC, RST> {
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST>,
}

impl<SPI, CS, BUSY, DC, RST> Ssd1680<SPI, CS, BUSY, DC, RST>
where
    SPI: Write<u8>,
    CS: OutputPin,
    CS::Error: Debug,
    BUSY: InputPin,
    DC: OutputPin,
    DC::Error: Debug,
    RST: OutputPin,
    RST::Error: Debug,
{
    /// Create and initialize the display driver
    pub fn new<DELAY: DelayMs<u8>>(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error>
    where
        Self: Sized,
    {
        let interface = DisplayInterface::new(cs, busy, dc, rst);
        let mut ssd1680 = Ssd1680 { interface };
        ssd1680.init(spi, delay)?;
        Ok(ssd1680)
    }

    /// Initialise the controller
    pub fn init<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.interface.reset(delay);
        self.interface.cmd(spi, cmd::Cmd::SW_RESET)?;
        self.interface.wait_until_idle(delay);

        self.interface
            .cmd_with_data(spi, cmd::Cmd::DRIVER_CONTROL, &[HEIGHT - 1, 0x00, 0x00])?;

        self.interface
            .cmd_with_data(spi, cmd::Cmd::DATA_ENTRY_MODE, &[flag::Flag::DATA_ENTRY_INCRY_INCRX])?;

        self.interface.cmd_with_data(
            spi,
            cmd::Cmd::BORDER_WAVEFORM_CONTROL,
            &[flag::Flag::BORDER_WAVEFORM_FOLLOW_LUT | flag::Flag::BORDER_WAVEFORM_LUT1],
        )?;

        self.interface
            .cmd_with_data(spi, cmd::Cmd::TEMP_CONTROL, &[flag::Flag::INTERNAL_TEMP_SENSOR])?;

        self.interface
            .cmd_with_data(spi, cmd::Cmd::DISPLAY_UPDATE_CONTROL, &[0x00, 0x80])?;

        self.use_full_frame(spi)?;

        self.interface.wait_until_idle(delay);
        Ok(())
    }

    /// Update the whole BW buffer on the display driver
    pub fn update_bw_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.use_full_frame(spi)?;
        self.interface
            .cmd_with_data(spi, cmd::Cmd::WRITE_BW_DATA, &buffer)
    }

    /// Update the whole Red buffer on the display driver
    pub fn update_red_frame(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        self.use_full_frame(spi)?;
        self.interface
            .cmd_with_data(spi, cmd::Cmd::WRITE_RED_DATA, &buffer)
    }

    /// Start an update of the whole display
    pub fn display_frame<DELAY: DelayMs<u8>>(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.interface
            .cmd_with_data(spi, cmd::Cmd::UPDATE_DISPLAY_CTRL2, &[flag::Flag::DISPLAY_MODE_1])?;
        self.interface.cmd(spi, cmd::Cmd::MASTER_ACTIVATE)?;

        self.interface.wait_until_idle(delay);

        Ok(())
    }

    /// Make the whole black and white frame on the display driver white
    pub fn clear_bw_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.use_full_frame(spi)?;

        // TODO: allow non-white background color
        let color = color::Color::White.get_byte_value();

        self.interface.cmd(spi, cmd::Cmd::WRITE_BW_DATA)?;
        self.interface
            .data_x_times(spi, color, u32::from(WIDTH) / 8 * u32::from(HEIGHT))?;
        Ok(())
    }

    /// Make the whole red frame on the display driver white
    pub fn clear_red_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        self.use_full_frame(spi)?;

        // TODO: allow non-white background color
        let color = color::Color::White.inverse().get_byte_value();

        self.interface.cmd(spi, cmd::Cmd::WRITE_RED_DATA)?;
        self.interface
            .data_x_times(spi, color, u32::from(WIDTH) / 8 * u32::from(HEIGHT))?;
        Ok(())
    }

    fn use_full_frame(&mut self, spi: &mut SPI) -> Result<(), SPI::Error> {
        // choose full frame/ram
        self.set_ram_area(spi, 0, 0, u32::from(WIDTH) - 1, u32::from(HEIGHT) - 1)?;

        // start from the beginning
        self.set_ram_counter(spi, 0, 0)
    }

    fn set_ram_area(
        &mut self,
        spi: &mut SPI,
        start_x: u32,
        start_y: u32,
        end_x: u32,
        end_y: u32,
    ) -> Result<(), SPI::Error> {
        assert!(start_x < end_x);
        assert!(start_y < end_y);

        self.interface.cmd_with_data(
            spi,
            cmd::Cmd::SET_RAMXPOS,
            &[(start_x >> 3) as u8, (end_x >> 3) as u8],
        )?;

        self.interface.cmd_with_data(
            spi,
            cmd::Cmd::SET_RAMYPOS,
            &[
                start_y as u8,
                (start_y >> 8) as u8,
                end_y as u8,
                (end_y >> 8) as u8,
            ],
        )?;
        Ok(())
    }

    fn set_ram_counter(&mut self, spi: &mut SPI, x: u32, y: u32) -> Result<(), SPI::Error> {
        // x is positioned in bytes, so the last 3 bits which show the position inside a byte in the ram
        // aren't relevant
        self.interface
            .cmd_with_data(spi, cmd::Cmd::SET_RAMX_COUNTER, &[(x >> 3) as u8])?;

        // 2 Databytes: A[7:0] & 0..A[8]
        self.interface
            .cmd_with_data(spi, cmd::Cmd::SET_RAMY_COUNTER, &[y as u8, (y >> 8) as u8])?;
        Ok(())
    }

    // pub fn wake_up<DELAY: DelayMs<u8>>(
    //     &mut self,
    //     spi: &mut SPI,
    //     delay: &mut DELAY,
    // ) -> Result<(), SPI::Error> {
    //     todo!()
    // }
}

//! Display interface using SPI

use core::fmt::Debug;
use core::marker::PhantomData;
use embedded_hal::{
    blocking::{delay::DelayMs, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

const RESET_DELAY_MS: u8 = 10;

/// The Connection Interface of all (?) Waveshare EPD-Devices
///
pub(crate) struct DisplayInterface<SPI, CS, BUSY, DC, RST> {
    /// SPI
    _spi: PhantomData<SPI>,
    /// CS for SPI
    cs: CS,
    /// Low for busy, Wait until display is ready!
    busy: BUSY,
    /// Data/Command Control Pin (High for data, Low for command)
    dc: DC,
    /// Pin for Reseting
    rst: RST,
}

impl<SPI, CS, BUSY, DC, RST> DisplayInterface<SPI, CS, BUSY, DC, RST>
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
    /// Create and initialize display
    pub fn new(cs: CS, busy: BUSY, dc: DC, rst: RST) -> Self {
        DisplayInterface {
            _spi: PhantomData::default(),
            cs,
            busy,
            dc,
            rst,
        }
    }

    /// Basic function for sending commands
    pub(crate) fn cmd(&mut self, spi: &mut SPI, command: u8) -> Result<(), SPI::Error> {
        // low for commands
        self.dc.set_low().unwrap();

        // Transfer the command over spi
        self.write(spi, &[command])
    }

    /// Basic function for sending an array of u8-values of data over spi
    pub(crate) fn data(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        // high for data
        self.dc.set_high().unwrap();

        // Transfer data (u8-array) over spi
        self.write(spi, data)
    }

    /// Basic function for sending a command and the data belonging to it.
    pub(crate) fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: u8,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.cmd(spi, command)?;
        self.data(spi, data)
    }

    /// Basic function for sending the same byte of data (one u8) multiple times over spi
    /// Used for setting one color for the whole frame
    pub(crate) fn data_x_times(
        &mut self,
        spi: &mut SPI,
        val: u8,
        repetitions: u32,
    ) -> Result<(), SPI::Error> {
        // high for data
        let _ = self.dc.set_high();
        // Transfer data (u8) over spi
        for _ in 0..repetitions {
            self.write(spi, &[val])?;
        }
        Ok(())
    }

    /// Waits until device isn't busy anymore (busy == HIGH)
    pub(crate) fn wait_until_idle<DELAY: DelayMs<u8>>(&mut self, delay: &mut DELAY) {
        while self.busy.is_high().unwrap_or(true) { delay.delay_ms(1) }
    }

    /// Resets the device.
    pub(crate) fn reset<DELAY: DelayMs<u8>>(&mut self, delay: &mut DELAY) {
        self.rst.set_low().unwrap();
        delay.delay_ms(RESET_DELAY_MS);
        self.rst.set_high().unwrap();
        delay.delay_ms(RESET_DELAY_MS);
    }

    // spi write helper/abstraction function
    fn write(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        // activate spi with cs low
        self.cs.set_low().unwrap();

        for data_chunk in data.chunks(4096) {
            spi.write(data_chunk)?;
        }

        // deativate spi with cs high
        self.cs.set_high().unwrap();

        Ok(())
    }
}

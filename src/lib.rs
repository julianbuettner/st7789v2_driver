#![no_std]
#![no_main]

use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;

pub const HORIZONTAL: u16 = 0;
pub const VERTICAL: u16 = 1;
/// Enumeration of instructions for the ST7789V2 display.
pub enum Instruction {
    Nop = 0x00,      // No Operation
    SwReset = 0x01,  // Software Reset
    RddId = 0x04,    // Read Display Identification Information
    RddSt = 0x09,    // Read Display Status
    SlpIn = 0x10,    // Enter Sleep Mode (SLPIN)
    SlpOut = 0x11,   // Exit Sleep Mode (SLPOUT)
    PtlOn = 0x12,    // Enter Partial Mode (PTLON)
    NorOn = 0x13,    // Enter Normal Mode (NORON)
    InvOff = 0x20,   // Display Inversion Off (INVOFF)
    InvOn = 0x21,    // Display Inversion On (INVON)
    GamSet = 0x26,   // Gamma Set (GAMSET) — Choose Gamma curve
    DispOff = 0x28,  // Display Off (DISPOFF)
    DispOn = 0x29,   // Display On (DISPON)
    CaSet = 0x2A,    // Column Address Set (CASET)
    RaSet = 0x2B,    // Row Address Set (RASET)
    RamWr = 0x2C,    // Memory Write (RAMWR)
    RamRd = 0x2E,    // Memory Read (RAMRD)
    PtlAr = 0x30,    // Partial Area (PTLAR)
    VScrDef = 0x33,  // Vertical Scrolling Definition (VSCRDEF)
    TEOFF = 0x34,    // Tearing Effect Line OFF (TEOFF)
    TEON = 0x35,     // Tearing Effect Line ON (TEON)
    MadCtl = 0x36,   // Memory Access Control (MADCTL)
    ColMod = 0x3A,   // Pixel Format Set (COLMOD)
    WrMemC = 0x3C,   // Write Memory Continue (WRMEMC)
    RdMemC = 0x3E,   // Read Memory Continue (RDMEMC)
    Ste = 0x44,      // Set Tear Scanline (STE)
    GScan = 0x45,    // Get Scanline (GSCAN)
    WrDisBV = 0x51,  // Write Display Brightness (WRDISBV)
    RdDisBV = 0x52,  // Read Display Brightness (RDDISBV)
    WrCtrLD = 0x53,  // Write CTRL Display (WRCTRLD)
    RdCtrLD = 0x54,  // Read CTRL Display (RDCTRLD)
    WrCACE = 0x55,   // Write CABC (WRCACE)
    RdCABC = 0x56,   // Read CABC (RDCABC)
    WrCABCMB = 0x5E, // Write CABC Minimum Brightness (WRCABCMB)
    RdCABCMB = 0x5F, // Read CABC Minimum Brightness (RDCABCMB)
    RdABCSDR = 0x68, // Read Automatic Brightness Control Self-Diagnostic Result (RDABCSDR)
    RdId1 = 0xDA,    // Read ID1 (RDID1)
    RdId2 = 0xDB,    // Read ID2 (RDID2)
    RdId3 = 0xDC,    // Read ID3 (RDID3)
}

/// Structure to represent a region.
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Region {
    pub x: u16,
    pub y: u16,
    pub width: u32,
    pub height: u32,
}

/// Driver for the ST7789V2 display.
pub struct ST7789V2<SPI, DC, CS, RST>
where
    SPI: SpiBus<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
{
    /// SPI interface.
    spi: SPI,

    /// Data/command pin.
    dc: DC,

    /// Chip select pin.
    cs: CS,

    /// Reset pin.
    rst: RST,

    /// Whether the display is RGB (true) or BGR (false).
    _rgb: bool,
    /// Screen Direction Horizontal or vertical
    sd: u16,

    /// Global image offset.
    width: u32,
    height: u32,
    regions: [Option<Region>; 10],
}

impl<SPI, DC, CS, RST> ST7789V2<SPI, DC, CS, RST>
where
    SPI: SpiBus<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
{
    /// Creates a new driver instance that uses hardware SPI.
    ///
    /// # Arguments
    ///
    /// * `spi` - SPI interface.
    /// * `dc` - Data/command pin.
    /// * `rst` - Reset pin.
    /// * `rgb` - Whether the display is RGB (true) or BGR (false).
    /// * `width` - Width of the display.
    /// * `height` - Height of the display.
    pub fn new(
        spi: SPI,
        dc: DC,
        cs: CS,
        rst: RST,
        _rgb: bool,
        sd: u16,
        width: u32,
        height: u32,
    ) -> Self {
        ST7789V2 {
            spi,
            dc,
            cs,
            rst,
            _rgb,
            sd,
            width,
            height,
            regions: [None; 10],
        }
    }

    /// Initializes the display.
    ///
    /// This function initializes the display by sending a sequence of commands and settings
    /// to configure the display properly. It includes a hardware reset and various configuration
    /// commands.
    ///
    /// # Arguments
    ///
    /// * `delay` - Delay provider.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayNs,
    {
        self.hard_reset(delay)?;
        //Set Attributes for Scan Direction
        if self.sd == VERTICAL {
            self.write_command(Instruction::MadCtl as u8, &[0x00])?; // Vertical
        } else {
            self.write_command(Instruction::MadCtl as u8, &[0x78])?; // Horizontal
        }

        //Initalize Display
        //self.write_command(Instruction::MadCtl as u8, &[0x00])?;  //Vertical Screen Direction
        self.write_command(Instruction::ColMod as u8, &[0x05])?;
        self.write_command(0xB2, &[0x0B, 0x0B, 0x00, 0x33, 0x35])?;
        self.write_command(0xB7, &[0x11])?;
        self.write_command(0xBB, &[0x35])?;
        self.write_command(0xC0, &[0x2C])?;
        self.write_command(0xC2, &[0x01])?;
        self.write_command(0xC3, &[0x0D])?;
        self.write_command(0xC4, &[0x20])?;
        self.write_command(0xC6, &[0x13])?;
        self.write_command(0xD0, &[0xA4, 0xA1])?;
        self.write_command(0xD6, &[0xA1])?;
        self.write_command(
            0xE0,
            &[
                0xF0, 0x06, 0x0B, 0x0A, 0x09, 0x26, 0x29, 0x33, 0x41, 0x18, 0x16, 0x15, 0x29, 0x2D,
            ],
        )?;
        self.write_command(
            0xE1,
            &[
                0xF0, 0x04, 0x08, 0x08, 0x07, 0x03, 0x28, 0x32, 0x40, 0x3B, 0x19, 0x18, 0x2A, 0x2E,
            ],
        )?;
        self.write_command(0xE4, &[0x25, 0x00, 0x00])?;
        self.write_command(Instruction::InvOn as u8, &[])?;
        self.write_command(Instruction::SlpOut as u8, &[])?;

        delay.delay_ms(120);

        self.write_command(Instruction::DispOn as u8, &[])?; // Display ON (DISPON)

        Ok(())
    }

    /// Performs a hard reset of the display.
    ///
    /// This function performs a hard reset by toggling the reset pin, ensuring the display
    /// is in a known state before initialization.
    ///
    /// # Arguments
    ///
    /// * `delay` - Delay provider.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub fn hard_reset<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayNs,
    {
        self.rst.set_high().map_err(|_| ())?;
        delay.delay_ms(10);
        self.rst.set_low().map_err(|_| ())?;
        delay.delay_ms(10);
        self.rst.set_high().map_err(|_| ())?;
        delay.delay_ms(10);

        Ok(())
    }

    /// Writes a command to the display.
    ///
    /// This function sends a command followed by optional parameters to the display.
    ///
    /// # Arguments
    ///
    /// * `command` - Command to write.
    /// * `params` - Parameters for the command.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    fn write_command(&mut self, command: u8, params: &[u8]) -> Result<(), ()> {
        self.cs.set_high().map_err(|_| ())?;
        self.dc.set_low().map_err(|_| ())?;
        self.cs.set_low().map_err(|_| ())?;
        self.spi.write(&[command]).map_err(|_| ())?;
        if !params.is_empty() {
            self.start_data()?;
            self.write_data(params)?;
        }
        self.cs.set_high().map_err(|_| ())?;
        Ok(())
    }

    /// Starts data transmission.
    ///
    /// Sets the data/command pin to indicate data mode for subsequent transmissions.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    fn start_data(&mut self) -> Result<(), ()> {
        self.dc.set_high().map_err(|_| ())
    }

    /// Writes data to the display.
    ///
    /// This function writes data to the display through the SPI interface.
    ///
    /// # Arguments
    ///
    /// * `data` - Data to write.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    fn write_data(&mut self, data: &[u8]) -> Result<(), ()> {
        self.cs.set_high().map_err(|_| ())?;
        self.dc.set_high().map_err(|_| ())?;
        self.cs.set_low().map_err(|_| ())?;
        self.spi.write(data).map_err(|_| ())?;
        self.cs.set_high().map_err(|_| ())?;
        Ok(())
    }

    /// Writes a data word to the display.
    ///
    /// This function writes a 16-bit word to the display.
    ///
    /// # Arguments
    ///
    /// * `value` - Data word to write.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    fn write_word(&mut self, value: u16) -> Result<(), ()> {
        self.write_data(&value.to_be_bytes())
    }

    /// Sets the address window for the display.
    ///
    /// This function sets the address window for subsequent drawing commands.
    ///
    /// # Arguments
    ///
    /// * `start_x` - Start x-coordinate.
    /// * `start_y` - Start y-coordinate.
    /// * `end_x` - End x-coordinate.
    /// * `end_y` - End y-coordinate.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub fn set_address_window(
        &mut self,
        start_x: u16,
        start_y: u16,
        end_x: u16,
        end_y: u16,
    ) -> Result<(), ()> {
        if self.sd == VERTICAL {
            self.write_command(Instruction::CaSet as u8, &[])?;
            self.start_data()?;
            // Write start and end x-coordinates
            self.write_data(&start_x.to_be_bytes())?; // Big-endian: splits into two bytes
            self.write_data(&(end_x - 1).to_be_bytes())?;
            self.write_command(Instruction::RaSet as u8, &[])?;
            self.start_data()?;
            // Write start and end y-coordinates (with a 20 pixel offset)
            self.write_data(&(start_y + 20).to_be_bytes())?;
            self.write_data(&(end_y + 20 - 1).to_be_bytes())?;
        } else {
            self.write_command(Instruction::CaSet as u8, &[])?;
            self.start_data()?;
            // Write start and end x-coordinates
            self.write_data(&(start_x + 20).to_be_bytes())?; // Big-endian: splits into two bytes
            self.write_data(&(end_x + 20 - 1).to_be_bytes())?;
            self.write_command(Instruction::RaSet as u8, &[])?;
            self.start_data()?;
            // Write start and end y-coordinates (with a 20 pixel offset)
            self.write_data(&(start_y).to_be_bytes())?;
            self.write_data(&(end_y - 1).to_be_bytes())?;
        }
        self.write_command(0x2C, &[])?;

        Ok(())
    }

    /// Clears the screen by filling it with a single color.
    ///
    /// This function sets the entire display to the specified color by writing data
    /// in chunks, which balances memory efficiency and performance.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to fill the screen with, in RGB565 format.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub fn clear_screen(&mut self, color: u16) -> Result<(), ()> {
        let color_high = (color >> 8) as u8;
        let color_low = (color & 0xff) as u8;

        // Set the address window to cover the entire screen
        self.set_address_window(0, 0, self.width as u16 - 1, self.height as u16 - 1)?;
        self.write_command(Instruction::RamWr as u8, &[])?;
        self.start_data()?;

        // Define a constant for the chunk size
        const CHUNK_SIZE: usize = 512;
        let mut chunk = [0u8; CHUNK_SIZE * 2];

        // Fill the chunk with the color data
        for i in 0..CHUNK_SIZE {
            chunk[i * 2] = color_high;
            chunk[i * 2 + 1] = color_low;
        }

        // Write data in chunks
        let total_pixels = (self.width * self.height) as usize;
        let full_chunks = total_pixels / CHUNK_SIZE;
        let remaining_pixels = total_pixels % CHUNK_SIZE;

        for _ in 0..full_chunks {
            self.write_data(&chunk)?;
        }

        if remaining_pixels > 0 {
            self.write_data(&chunk[0..(remaining_pixels * 2)])?;
        }

        Ok(())
    }

    /// Sets a pixel color at the given coordinates.
    ///
    /// This function sets the color of a single pixel at the specified coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - X-coordinate.
    /// * `y` - Y-coordinate.
    /// * `color` - Color of the pixel.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub fn write_pixel(&mut self, x: u16, y: u16, color: u16) -> Result<(), ()> {
        self.set_address_window(x, y, x, y)?;
        self.write_command(Instruction::RamWr as u8, &[])?;
        self.start_data()?;
        self.write_word(color)
    }

    /// Draws an image from a slice of RGB565 data.
    ///
    /// This function draws an image from a slice of pixel data in RGB565 format.
    /// It assumes the image dimensions match the display dimensions.
    ///
    /// # Arguments
    ///
    /// * `image_data` - Image data to draw.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub fn draw_image(&mut self, image_data: &[u8]) -> Result<(), ()> {
        let width = self.width as u16;
        let height = self.height as u16;

        self.set_address_window(0, 0, width - 1, height - 1)?;
        self.write_command(Instruction::RamWr as u8, &[])?;
        self.start_data()?;

        for chunk in image_data.chunks(32) {
            self.write_data(chunk)?;
        }

        Ok(())
    }

    /// Displays the provided buffer on the screen.
    ///
    /// This function writes the entire buffer to the display, assuming the buffer
    /// contains pixel data for the full display area.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer to display.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub fn show(&mut self, buffer: &[u8]) -> Result<(), ()> {
        self.write_command(Instruction::CaSet as u8, &[])?;
        self.write_data(&[0x00, 0x00, 0x00, 0xEF])?;

        self.write_command(Instruction::RaSet as u8, &[])?;
        self.write_data(&[0x00, 0x00, 0x00, 0xEF])?;

        self.write_command(Instruction::RamWr as u8, &[])?;

        self.cs.set_high().map_err(|_| ())?;
        self.dc.set_high().map_err(|_| ())?;
        self.cs.set_low().map_err(|_| ())?;
        self.spi.write(buffer).map_err(|_| ())?;
        self.cs.set_high().map_err(|_| ())?;

        Ok(())
    }

    /// Updates only the specified region of the display with the provided buffer.
    ///
    /// This function updates a specified rectangular region of the display with the pixel data
    /// provided in the buffer. It calculates the necessary offsets and addresses to update only
    /// the designated area, ensuring efficient display refresh.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A slice of bytes representing the pixel data in RGB565 format.
    /// * `top_left_x` - The x-coordinate of the top-left corner of the region to update.
    /// * `top_left_y` - The y-coordinate of the top-left corner of the region to update.
    /// * `width` - The width of the region to update.
    /// * `height` - The height of the region to update.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success (`Ok`) or failure (`Err`).
    pub fn show_region(
        &mut self,
        buffer: &[u8],
        top_left_x: u16,
        top_left_y: u16,
        width: u32,
        height: u32,
    ) -> Result<(), ()> {
        let start_x = top_left_x as u16; // Start x-coordinate
        let start_y = top_left_y as u16; // Start y-coordinate
        let end_x = (top_left_x as u32 + width - 1) as u16; // End x-coordinate
        let end_y = (top_left_y as u32 + height - 1) as u16; // End y-coordinate

        // Calculate the buffer offset for the region
        let buffer_width = self.width as usize; // Width of the buffer
        let bytes_per_pixel = 2; // Number of bytes per pixel in RGB565 format

        // Set the address window for the region to be updated
        self.set_address_window(start_x, start_y, end_x, end_y)?;

        // Send the command to write to RAM
        self.write_command(Instruction::RamWr as u8, &[])?;

        // Start data transmission
        self.start_data()?;

        // Iterate over each row in the region
        for y in start_y..=end_y {
            let start_index = ((y as usize) * buffer_width + (start_x as usize)) * bytes_per_pixel;
            let end_index = start_index + (width as usize) * bytes_per_pixel;

            // Write data to the display in chunks of 32 bytes
            for chunk in buffer[start_index..end_index].chunks(32) {
                self.write_data(chunk)?;
            }
        }

        Ok(())
    }

    pub fn store_region(&mut self, region: Region) -> Result<(), ()> {
        for i in 0..self.regions.len() {
            if self.regions[i].is_none() {
                self.regions[i] = Some(region);
                return Ok(());
            }
        }
        Err(())
    }

    pub fn store_region_from_params(
        &mut self,
        x: u16,
        y: u16,
        width: u32,
        height: u32,
    ) -> Result<(), ()> {
        let region = Region {
            x,
            y,
            width,
            height,
        };

        self.store_region(region)
    }

    pub fn get_regions(&self) -> &[Option<Region>] {
        &self.regions
    }

    pub fn clear_regions(&mut self) {
        self.regions = [None; 10];
    }

    pub fn show_regions(&mut self, buffer: &[u8]) -> Result<(), ()> {
        for i in 0..self.regions.len() {
            if self.regions[i].is_some() {
                if let Some(region_data) = self.regions[i] {
                    self.show_region(
                        buffer,
                        region_data.x,
                        region_data.y,
                        region_data.width,
                        region_data.height,
                    )?;
                }
            }
        }

        Ok(())
    }

    // Additional function with default parameter
    pub fn show_regions_and_clear(&mut self, buffer: &[u8]) -> Result<(), ()> {
        if let Err(e) = self.show_regions(buffer) {
            // Handle the error, e.g., log it or return a different error
            return Err(e);
        }
        self.clear_regions();
        Ok(())
    }
}

// Implementing the DrawTarget trait for the ST7789V2 display driver
impl<SPI, DC, CS, RST> DrawTarget for ST7789V2<SPI, DC, CS, RST>
where
    SPI: SpiBus<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
{
    type Color = Rgb565;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            let color_value = color.into_storage();
            // Only draw pixels that would be on screen
            if coord.x >= 0
                && coord.y >= 0
                && coord.x < self.width as i32
                && coord.y < self.height as i32
            {
                self.write_pixel(coord.x as u16, coord.y as u16, color_value)?;
            }
        }
        Ok(())
    }
}

// Implementing the OriginDimensions trait for the ST7789V2 display driver
impl<SPI, DC, CS, RST> OriginDimensions for ST7789V2<SPI, DC, CS, RST>
where
    SPI: SpiBus<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

/// A structure representing a frame buffer.
pub struct FrameBuffer<'a> {
    buffer: &'a mut [u8],
    width: u32,
    height: u32,
}

impl<'a> FrameBuffer<'a> {
    /// Creates a new frame buffer.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A mutable slice representing the pixel data.
    /// * `width` - The width of the frame buffer.
    /// * `height` - The height of the frame buffer.
    pub fn new(buffer: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    /// Returns a reference to the buffer.
    ///
    /// # Returns
    ///
    /// A reference to the buffer.
    pub fn get_buffer(&self) -> &[u8] {
        self.buffer
    }

    /// Clears the frame buffer with the specified color.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to clear the buffer with.
    pub fn clear(&mut self, color: Rgb565) {
        let raw_color = color.into_storage();
        for chunk in self.buffer.chunks_exact_mut(2) {
            chunk[0] = (raw_color >> 8) as u8;
            chunk[1] = raw_color as u8;
        }
    }

    /// Copies a region from another buffer into this buffer.
    ///
    /// # Arguments
    ///
    /// * `src_buffer` - The source buffer.
    /// * `src_x` - The x-coordinate of the top-left corner of the source region.
    /// * `src_y` - The y-coordinate of the top-left corner of the source region.
    /// * `src_width` - The width of the source region.
    /// * `src_height` - The height of the source region.
    /// * `dest_x` - The x-coordinate of the top-left corner of the destination region.
    /// * `dest_y` - The y-coordinate of the top-left corner of the destination region.
    pub fn copy_region(
        &mut self,
        src_buffer: &[u8],
        src_x: u16,
        src_y: u16,
        src_width: u32,
        src_height: u32,
        dest_x: u16,
        dest_y: u16,
    ) {
        for row in 0..src_height as usize {
            let src_row_start =
                (src_y as usize + row) * self.width as usize * 2 + src_x as usize * 2;
            let src_row_end = src_row_start + src_width as usize * 2;

            let dest_row_start =
                (dest_y as usize + row) * self.width as usize * 2 + dest_x as usize * 2;
            let dest_row_end = dest_row_start + src_width as usize * 2;

            self.buffer[dest_row_start..dest_row_end]
                .copy_from_slice(&src_buffer[src_row_start..src_row_end]);
        }
    }

    /// Restores regions from a source buffer into the frame buffer.
    ///
    /// # Arguments
    ///
    /// * `src_buffer` - The source buffer.
    /// * `regions` - An array of regions to restore.
    pub fn copy_regions(&mut self, src_buffer: &[u8], regions: &[Option<Region>]) {
        for region in regions.iter().flatten() {
            self.copy_region(
                src_buffer,
                region.x,
                region.y,
                region.width,
                region.height,
                region.x,
                region.y,
            );
        }
    }

    /// Compares the current frame buffer with another frame buffer and returns an iterator
    /// of `Pixel` that can be drawn to update the display.
    ///
    /// # Arguments
    ///
    /// * `other` - The other frame buffer to compare against.
    ///
    /// # Returns
    ///
    /// An iterator of `Pixel<Rgb565>`.
    pub fn diff_with<'b>(
        &'b self,
        other: &'b FrameBuffer<'a>,
    ) -> impl Iterator<Item = Pixel<Rgb565>> + 'b {
        self.buffer
            .chunks_exact(2)
            .enumerate()
            .filter_map(move |(i, chunk)| {
                let other_chunk = &other.buffer[i * 2..i * 2 + 2];
                if chunk != other_chunk {
                    let x = (i as u32 % self.width) as i32;
                    let y = (i as u32 / self.width) as i32;
                    let raw_color = u16::from_be_bytes([chunk[0], chunk[1]]);
                    let color = Rgb565::from(RawU16::new(raw_color));
                    Some(Pixel(Point::new(x, y), color))
                } else {
                    None
                }
            })
    }
}

impl<'a> DrawTarget for FrameBuffer<'a> {
    type Color = Rgb565;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0
                && coord.x < self.width as i32
                && coord.y >= 0
                && coord.y < self.height as i32
            {
                let index = ((coord.y as u32 * self.width + coord.x as u32) * 2) as usize;
                let raw_color = color.into_storage();
                self.buffer[index] = (raw_color >> 8) as u8;
                self.buffer[index + 1] = raw_color as u8;
            }
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for FrameBuffer<'a> {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

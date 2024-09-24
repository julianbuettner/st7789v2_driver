extern crate image;
use image::GenericImageView;
use std::fs::File;
use std::io::Write;

fn save_rgb565_data_as_raw_file(filename: &str, rgb888_data: &[u8], big_endian: bool) -> std::io::Result<()> {
    let mut raw_data = Vec::new();

    for chunk in rgb888_data.chunks(3) {
        let r_8bit = chunk[0];
        let g_8bit = chunk[1];
        let b_8bit = chunk[2];

        // Convert to RGB565 format
        let r = (r_8bit >> 3) as u16 & 0x1F;
        let g = (g_8bit >> 2) as u16 & 0x3F;
        let b = (b_8bit >> 3) as u16 & 0x1F;

        // Combine to RGB565
        let rgb565 = (r << 11) | (g << 5) | b;

        // Append the two bytes of RGB565 data with proper endianness
        if big_endian {
            raw_data.push((rgb565 >> 8) as u8);  // MSB
            raw_data.push((rgb565 & 0xFF) as u8);  // LSB
        } else {
            raw_data.push((rgb565 & 0xFF) as u8);  // LSB
            raw_data.push((rgb565 >> 8) as u8);  // MSB
        }
    }

    // Save the raw data as a binary file
    let mut file = File::create(filename)?;
    file.write_all(&raw_data)?;

    Ok(())
}
/*
Please note that because of the .cargo/config.toml in the parent directory this code will not build or compile unless the
target is commented out.

This is an example of how to convert images I included it for completness

The PNG image in this example is 24-bit depth.  The original image of 32-bit depth does not convert to raw.

*/
fn main() -> std::io::Result<()> {
    let img_path = "assets/rust-logo-240x240.png";

    // Load the image
    let img = image::open(&img_path).unwrap();
    let rgb888_data = img.to_rgb8().into_raw();

    // Save the raw data in big-endian format
    let output_path = "assets/rust-logo-240x240.raw";
    save_rgb565_data_as_raw_file(&output_path, &rgb888_data, true)?;

    Ok(())
}

use it8951::Config;
use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};
use linux_embedded_hal::spidev::{SpiModeFlags, SpidevOptions};
use linux_embedded_hal::{CdevPin, Delay, SpidevDevice};
use std::error::Error;
use embedded_graphics::text::renderer::TextRenderer;
use std::io;
use std::io::Read;
use std::cell::RefCell;

use embedded_graphics::{
    pixelcolor::Gray4,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use u8g2_fonts::{
    fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Raspi SPI0.0
    // MISO: 9
    // MOSI: 10
    // SCK: 11
    // CS: 8
    let mut spi = SpidevDevice::open("/dev/spidev0.0")?;
    let spi_options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(12_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&spi_options)?;

    let mut chip = Chip::new("/dev/gpiochip0")?;
    // RST: 17
    let rst_output = chip.get_line(17)?;
    let rst_output_handle = rst_output.request(LineRequestFlags::OUTPUT, 0, "meeting-room")?;
    let rst = CdevPin::new(rst_output_handle)?;
    // BUSY / HDRY: 24
    let busy_input = chip.get_line(24)?;
    let busy_input_handle = busy_input.request(LineRequestFlags::INPUT, 0, "meeting-room")?;
    let busy = CdevPin::new(busy_input_handle)?;

    let driver = it8951::interface::IT8951SPIInterface::new(spi, busy, rst, Delay);
    let mut epd = it8951::IT8951::new(driver, Config::default())
        .init(1670)
        .unwrap();

    println!(
        "Reset and initalized E-Ink Display: \n\r {:?}",
        epd.get_dev_info()
    );

    // Rectangle::new(Point::new(0, 0), Size::new(1872, 1404))
    //     .into_styled(PrimitiveStyle::with_fill(Gray4::new(255)))
    //     .draw(&mut epd)
    //     .unwrap();


    let text_style = embedded_graphics::mono_font::MonoTextStyle::new(
        &embedded_graphics::mono_font::iso_8859_1::FONT_9X18_BOLD,
        Gray4::new(0),
    );

    let mut position = text_style.draw_string(
        "ready",
        Point::new(0, 16),
        embedded_graphics::text::Baseline::Bottom,
        &mut UpScale(&mut epd)
    ).unwrap();

    loop {
        for b in "all work and no play makes jack dull boy\n".bytes() {
            let mut us = UpScale(&mut epd);

            if b.clone() == 10 {
                position = Point::new(0, position.y + 16);
                continue;
            }

            position = text_style.draw_string(
                &format!("{}", char::from(b)),
                position,
                embedded_graphics::text::Baseline::Bottom,
                &mut us
            ).unwrap();
            epd.display(it8951::WaveformMode::A2).unwrap();
        }
    }

    epd.sleep().unwrap();

    Ok(())
}

struct UpScale<'a, T>(&'a mut T) where T: DrawTarget;

impl <'a, T>Dimensions for UpScale<'a, T> where T: DrawTarget {
    fn bounding_box(&self) -> Rectangle {
        Rectangle {
            top_left: self.0.bounding_box().top_left,
            size:  embedded_graphics_core::geometry::Size {
                height: self.0.bounding_box().size.height >> 2,
                width: self.0.bounding_box().size.width >> 2,
            }
        }
    }
}

impl <'a, T>DrawTarget for UpScale<'a, T> where T: DrawTarget {
    type Color = T::Color;
    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error> where I: IntoIterator<Item = Pixel<Self::Color>> {
        for Pixel(point, color) in pixels {
            self.0.fill_solid(
                &Rectangle::new(Point::new(point.x << 2, point.y << 2), Size::new(4, 4)),
                color
            )?
        }
        Ok(())
    }
}

use embedded_graphics::{
    mono_font::{ascii::* , MonoTextStyle},
    pixelcolor::{BinaryColor},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio;
use esp_idf_svc::hal::prelude::Peripherals;

use esp32s2_common_lib::set_sh1106_display;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    log::info!("Initializing OLED");

    // 1. ピン定義
    let rst_pin = peripherals.pins.gpio38;
    let dc_pin = peripherals.pins.gpio37;
    let sclk_pin = peripherals.pins.gpio36;
    let sda_pin = peripherals.pins.gpio35;
    let cs_pin = peripherals.pins.gpio34;
    let spi_peripheral = peripherals.spi2;

    let (mut display, _rst_driver) = set_sh1106_display(
        rst_pin,
        dc_pin,
        sclk_pin, 
        sda_pin,
        cs_pin, 
        spi_peripheral
    )?;

    // 8. 描画処理
    // `clear`は引数を取らず、エラーも返さない
    display.clear();

   let style = PrimitiveStyleBuilder::new()
       .stroke_color(BinaryColor::On)
       .stroke_width(1)
       .build();

    Rectangle::new(Point::new(2, 2), Size::new(126, 60))
        .into_styled(style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Draw rectangle error: {:?}", e))?;

    let text_style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);
    Text::new("Hello OLED!", Point::new(10, 25), text_style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Draw text error: {:?}", e))?;

    display.flush().map_err(|e| anyhow::anyhow!("Display flush error: {:?}", e))?;

    let mut sec = 0;
    loop {
        display.clear();

        let style = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(1)
        .build();

        Rectangle::new(Point::new(2, 2), Size::new(3, 3))
        .into_styled(style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Draw rectangle error: {:?}", e))?;

        let text_style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);
        Text::new(format!("SEC {}", sec).as_str(), Point::new(10, 25), text_style)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Draw text error: {:?}", e))?;

        display.flush().map_err(|e| anyhow::anyhow!("Display flush error: {:?}", e))?;
        sec += 1;
        sec = sec % 60;
        FreeRtos::delay_ms(1000);
    }
}

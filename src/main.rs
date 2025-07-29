// 修正点: 古いAPIに合わせてSPIInterfaceNoCSをインポート
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
//use embedded_hal::spi::MODE_3;
use esp_idf_svc::hal::{
    delay::Ets,
    gpio::AnyIOPin,
    spi::{config::{Config as SpiConfig, MODE_3}, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
    units::FromValueType,
};
// 修正点: embedded_halから直接MODE_3をインポート
use mipidsi::{Builder, Orientation};

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::prelude::Peripherals;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // --- LCD初期化コード ---
    log::info!("Initializing LCD");

    let rst_pin = peripherals.pins.gpio38;
    let dc_pin = peripherals.pins.gpio37;
    let mut backlight = PinDriver::output(peripherals.pins.gpio33)?;
    let sclk_pin = peripherals.pins.gpio36;
    let sda_pin = peripherals.pins.gpio35;
    let cs_pin = peripherals.pins.gpio34;
    let spi_peripheral = peripherals.spi2;

    let spi_driver = SpiDriver::new(
        spi_peripheral,
        sclk_pin,
        sda_pin,
        None::<AnyIOPin>,
        &SpiDriverConfig::new(),
    )?;

    let spi_config = SpiConfig::new()
       .baudrate(40.MHz().into())
       .data_mode(MODE_3); // 修正点: インポートしたMODE_3を直接使用

    let spi_device = SpiDeviceDriver::new(spi_driver, Some(cs_pin), &spi_config)?;

    let dc_driver = PinDriver::output(dc_pin)?;
    // 修正点: 古いAPIのSPIInterfaceNoCSを使用
    let di = SPIInterfaceNoCS::new(spi_device, dc_driver);

    let rst_driver = PinDriver::output(rst_pin)?;
    let mut delay = Ets;

    let mut display = Builder::st7789(di)
       .with_display_size(240, 240)
       .with_orientation(Orientation::Portrait(false))
       .init(&mut delay, Some(rst_driver)) // `?`演算子が使えるようにエラー型を修正
       .map_err(|e| anyhow::anyhow!("Display init error: {:?}", e))?;

    backlight.set_high()?;
    display.clear(Rgb565::BLACK).map_err(|e| anyhow::anyhow!("Display clear error: {:?}", e))?;

    log::info!("LCD Initialized");

    let style = PrimitiveStyleBuilder::new()
       .stroke_color(Rgb565::YELLOW)
       .fill_color(Rgb565::BLUE)
       .stroke_width(2)
       .build();

    Rectangle::new(Point::new(20, 20), Size::new(200, 50))
       .into_styled(style)
       .draw(&mut display)
       .map_err(|e| anyhow::anyhow!("Draw rectangle error: {:?}", e))?;

    let text_style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    Text::new("Hello, Rust!", Point::new(60, 45), text_style)
       .draw(&mut display)
       .map_err(|e| anyhow::anyhow!("Draw text error: {:?}", e))?;

    let mut led = PinDriver::output(peripherals.pins.gpio2)?;
    loop {
        log::info!("Blinking LED...");
        led.set_high()?;
        FreeRtos::delay_ms(1000);
        led.set_low()?;
        FreeRtos::delay_ms(1000);
    }
}

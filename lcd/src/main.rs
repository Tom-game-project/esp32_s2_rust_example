use embedded_graphics::{
    mono_font::{ascii::* , MonoTextStyle},
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use esp_idf_svc::hal::{
    gpio::AnyIOPin,
    spi::{
        config::Config as SpiConfig,
        config::MODE_3,
        SpiDeviceDriver,
        SpiDriver,
        SpiDriverConfig
    },
    units::FromValueType,
};
use sh1106::{prelude::*, Builder};

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::prelude::Peripherals;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    log::info!("Initializing OLED");

    // 1. ピン定義
    let rst_pin = peripherals.pins.gpio38;
    let dc_pin = peripherals.pins.gpio37;
    let mut backlight = PinDriver::output(peripherals.pins.gpio33)?;
    let sclk_pin = peripherals.pins.gpio36;
    let sda_pin = peripherals.pins.gpio35;
    let cs_pin = peripherals.pins.gpio34;
    let spi_peripheral = peripherals.spi2;

    // 2. SPIドライバ初期化
    let spi_driver = SpiDriver::new(
        spi_peripheral,
        sclk_pin,
        sda_pin,
        None::<AnyIOPin>,
        &SpiDriverConfig::new(),
    )?;

    // 3. SPI通信設定
    let spi_config = SpiConfig::new()
   .baudrate(40.MHz().into())
   .data_mode(MODE_3); // `esp-idf-hal`が提供する`SpiMode`を使用

    // 4. SPIデバイスドライバ作成
    let spi_device = SpiDeviceDriver::new(spi_driver, None::<AnyIOPin>, &spi_config)?;

    // 5. 制御ピンのドライバを作成
    let dc_driver = PinDriver::output(dc_pin)?;
    let cs_driver = PinDriver::output(cs_pin)?;
    let mut rst_driver = PinDriver::output(rst_pin)?;

    // 6. ディスプレイドライバ初期化
    // ハードウェアリセットを実行
    rst_driver.set_low()?;
    FreeRtos::delay_ms(50);
    rst_driver.set_high()?;
    FreeRtos::delay_ms(50);

    // sh1106のBuilderに、生のSPIデバイスと制御ピンを直接渡す
    let mut display: GraphicsMode<_> = Builder::new()
   .with_size(DisplaySize::Display128x64)
   .with_rotation(DisplayRotation::Rotate0)
   .connect_spi(spi_device, dc_driver, cs_driver)
   .into();

    // `init`は引数を取らない
    display.init().map_err(|e| anyhow::anyhow!("Display init error: {:?}", e))?;
    log::info!("OLED Initialized");

    backlight.set_high()?;

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

    // let mut led = PinDriver::output(peripherals.pins.gpio2)?;
    let mut sec = 0;
    loop {
        //log::info!("Blinking LED...");
        //led.set_high()?;
        //FreeRtos::delay_ms(1000);
        //led.set_low()?;
        //FreeRtos::delay_ms(1000);

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

use embedded_graphics::{
    mono_font::{ascii::* , MonoTextStyle},
    pixelcolor::{BinaryColor},
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
use esp_idf_svc::hal::gpio;
use esp_idf_svc::hal::prelude::Peripherals;
use crate::gpio::OutputPin;
use crate::gpio::Output;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::spi;

/// 任意のGPIOピンとSPIペリフェラルを受け取り、ディスプレイの初期化を行う
fn set_display<'d, RST, DC, SCLK, SDA, CS, SPI>(
    rst_pin: RST,
    dc_pin: DC,
    sclk_pin: SCLK,
    sda_pin: SDA,
    cs_pin: CS,
    spi_peripheral: SPI,
) -> anyhow::Result<(
    GraphicsMode<
        SpiInterface<
            SpiDeviceDriver<'d, SpiDriver<'d>>,
            // PinDriverにOutputモードを指定
            PinDriver<'d, DC, Output>,
            PinDriver<'d, CS, Output>
        >
    >,
    PinDriver<'d, RST, Output> // rst_driverの型 (DropされてしまうとLCDがうまく表示されない)
)>
where
    // コンパイラの指示に従い、必要なトレイト境界を追加
    RST: Peripheral<P = RST> + OutputPin,
    DC: Peripheral<P = DC> + OutputPin,
    CS: Peripheral<P = CS> + OutputPin,
    SCLK: Peripheral<P = SCLK> + OutputPin,
    SDA: Peripheral<P = SDA> + OutputPin,
    SPI: Peripheral<P = SPI> + spi::Spi + esp_idf_svc::hal::spi::SpiAnyPins + 'd,
{
    // ...関数の実装部分は変更なし...

    let spi_driver = SpiDriver::new(
        spi_peripheral,
        sclk_pin,
        sda_pin,
        None::<AnyIOPin>,
        &SpiDriverConfig::new(),
    )?;

    let spi_config = SpiConfig::new()
        .baudrate(40.MHz().into())
        .data_mode(MODE_3);

    let spi_device = SpiDeviceDriver::new(spi_driver, None::<AnyIOPin>, &spi_config)?;

    let dc_driver = PinDriver::output(dc_pin)?;
    let cs_driver = PinDriver::output(cs_pin)?;
    let mut rst_driver = PinDriver::output(rst_pin)?;

    rst_driver.set_low()?;
    FreeRtos::delay_ms(50);
    rst_driver.set_high()?;
    FreeRtos::delay_ms(50);

    let mut display :GraphicsMode<_>= Builder::new()
        .with_size(DisplaySize::Display128x64)
        .with_rotation(DisplayRotation::Rotate0)
        .connect_spi(spi_device, dc_driver, cs_driver)
        .into();

    display.init().map_err(|e| anyhow::anyhow!("Display init error: {:?}", e))?;
    log::info!("OLED Initialized");

    Ok((display, rst_driver))
}

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

    let (mut display, _rst_driver) = set_display(
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
        log::info!("hello world    !!!!!");
    }
}

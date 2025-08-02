
use embedded_graphics::{
    mono_font::{ascii::* , MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use embedded_hal::spi::{Mode, Phase, Polarity, MODE_0};
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

mod adxl367;
use adxl367::Adxl367;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    log::info!("Initializing OLED");

    // 1. ピン定義
    let miso_pin = peripherals.pins.gpio37; // -> miso pin 
    let mut backlight = PinDriver::output(peripherals.pins.gpio33)?;
    let sclk_pin = peripherals.pins.gpio36;
    let mosi_pin = peripherals.pins.gpio35; // -> mosi pin
                                                    
    // OLED
    let pled_rst_pin = peripherals.pins.gpio38; // 
    let oled_cs_pin = peripherals.pins.gpio34;
    let oled_dc_pin = peripherals.pins.gpio4;
    let spi_peripheral = peripherals.spi2;
 
    let cs_adxl = peripherals.pins.gpio8;

    // 2. 共有SPIドライバ初期化
    let spi_driver = SpiDriver::new(
        spi_peripheral,
        sclk_pin,
        mosi_pin,
        Some(miso_pin),
        //None::<AnyIOPin>,
        &SpiDriverConfig::new(),
    )?;

    // 3. SPI通信設定
    let spi_config_display = SpiConfig::new()
       .baudrate(2.MHz().into())
       .data_mode(MODE_3);
       //.data_mode(Mode {
       //    polarity: Polarity::IdleHigh,
       //    phase: Phase::CaptureOnSecondTransition
       //}); // `esp-idf-hal`が提供する`SpiMode`を使用

    let spi_config_adlx = SpiConfig::new()
       .baudrate(1.MHz().into())
       .data_mode(MODE_0);
       //.data_mode(Mode {
       //    polarity: Polarity::IdleLow,
       //    phase: Phase::CaptureOnFirstTransition
       //}); // `esp-idf-hal`が提供する`SpiMode`を使用

    // 4. SPIデバイスドライバ作成
    let oled_spi_device = SpiDeviceDriver::new(
        &spi_driver, 
        None::<AnyIOPin>,
        &spi_config_display
    )?;

    // ADXL367用のCS（GPIO32）をドライバ化して SpiDeviceDriver を構築
    // let mut cs_adxl_driver = PinDriver::output(cs_adxl)?;

    let adxl_spi_device = SpiDeviceDriver::new(
        &spi_driver,
        Some::<AnyIOPin>(cs_adxl.into()), 
        &spi_config_adlx
    )?;

    // 3. ドライバに渡す
    let mut adxl = Adxl367::new(adxl_spi_device);
    // ★★★★★ ここが最も重要な修正点 ★★★★★
    // センサーを測定モードにするためにinitを呼び出す
    adxl.init().expect("Failed to initialize ADXL367");

    // 5. 制御ピンのドライバを作成
    let dc_driver = PinDriver::output(oled_dc_pin)?;
    let cs_driver = PinDriver::output(oled_cs_pin)?;
    let mut rst_driver = PinDriver::output(pled_rst_pin)?;

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
   .connect_spi(oled_spi_device, dc_driver, cs_driver)
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

    FreeRtos::delay_ms(2000);

    let mut sec = 0;
    loop {
        log::info!("Hello ESP32 S2...");

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

        if let Ok((ax, ay, az)) = adxl.acceleration_raw(){
            //
            log::info!("ax {}, ay {}, az {}", ax, ay, az);
        }
        else
        {
            log::info!("Error !");
        }
        //Text::new(format!("ax {}, ay {}, az {}", ax, ay, az).as_str(), Point::new(10, 40), text_style)
        //.draw(&mut display)
        //.map_err(|e| anyhow::anyhow!("Draw text error: {:?}", e))?;
        //

        display.flush().map_err(|e| anyhow::anyhow!("Display flush error: {:?}", e))?;
        sec += 1;
        sec = sec % 60;
        FreeRtos::delay_ms(1000);
    }
}


/*
use embedded_hal::spi::{Mode, Phase, Polarity};
use esp_idf_svc::hal::{
    delay::FreeRtos,
    gpio::AnyIOPin,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
    units::FromValueType,
    prelude::Peripherals,
};

mod adxl367;
use adxl367::Adxl367;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    log::info!("Starting ADXL367 Diagnosis...");

    // 1. SPIバスのピン定義
    let sclk_pin = peripherals.pins.gpio36;
    let mosi_pin = peripherals.pins.gpio35;
    let miso_pin = peripherals.pins.gpio37;
    let adxl_cs_pin = peripherals.pins.gpio8;
    let spi_peripheral = peripherals.spi2;

    // 2. 共有SPIドライバ初期化
    let spi_driver = SpiDriver::new(
        spi_peripheral,
        sclk_pin,
        mosi_pin,
        Some(miso_pin),
        &SpiDriverConfig::new(),
    )?;

    // 3. ADXL367用のSPI通信設定 (MODE 0)
    let adxl_spi_config = SpiConfig::new().baudrate(1.MHz().into()).data_mode(Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    });

    // 4. ADXL367用のSPIデバイスドライバを作成
    let adxl_spi_device =
        SpiDeviceDriver::new(&spi_driver, Some::<AnyIOPin>(adxl_cs_pin.into()), &adxl_spi_config)?;

    // 5. ドライバのインスタンス化
    let mut adxl = Adxl367::new(adxl_spi_device);

    // 6. メインループでデバイスIDを読み続ける
    loop {
        match adxl.device_id() {
            Ok((ad, mst, part)) => {
                log::info!("Successfully read Chip ID -> AD: 0x{:02X}, MST: 0x{:02X}, PART: 0x{:02X}", ad, mst, part);
                if ad == 0xAD && mst == 0x1D && part == 0xF7 {
                    log::info!("SUCCESS! Correct Chip ID verified.");
                } else {
                    log::error!("FAIL! Chip ID does not match expected values.");
                }
            }
            Err(_) => {
                log::error!("Failed to communicate with ADXL367. Check wiring.");
            }
        }
        FreeRtos::delay_ms(1000);
    }
}

*/

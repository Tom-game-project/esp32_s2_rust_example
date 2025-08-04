use embedded_graphics::primitives::Circle;
use embedded_svc::sys_time::SystemTime;
use esp_idf_hal::sys::{gettimeofday, timeval, tzset};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::rmt::{
    config::TransmitConfig, 
    FixedLengthSignal, 
    PinState,
    Pulse, 
    TxRmtDriver
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
use esp_idf_svc::{
    wifi::EspWifi,
    wifi::ClientConfiguration,
    wifi::Configuration,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
    systime::EspSystemTime,
    sntp::{EspSntp, SyncStatus},
};

use esp_idf_svc::hal::delay::FreeRtos;

use heapless::String;
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::time::Duration;

use sh1106::{prelude::*, Builder};
use embedded_graphics::{
    mono_font::{ascii::* , MonoTextStyle},
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

const SSID_STR: &'static str = env!("SSID");
const SSID_PASSWORD_STR: &'static str = env!("SSID_PASSWORD");

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let ssid = String::<32>::try_from(SSID_STR).unwrap();
    let password = String::<64>::try_from(SSID_PASSWORD_STR).unwrap();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();


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
    display.init().map_err(|e| anyhow::anyhow!("Display init error: {:?}", e))?;
    log::info!("OLED Initialized");

    backlight.set_high()?;
    display.clear();


    let mut wifi_driver = EspWifi::new(
        peripherals.modem,
        sys_loop,
        Some(nvs)
    ).unwrap();

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration{
        ssid,
        password,
        ..Default::default()
    })).unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();

    // NeoPixel (WS2812B) on GPIO18
    let led_pin = peripherals.pins.gpio18;
    let channel = peripherals.rmt.channel0;
    let config = TransmitConfig::new().clock_divider(1);
    let mut neopixel_tx = TxRmtDriver::new(channel, led_pin, &config)?;

    esp_idf_svc::log::EspLogger::initialize_default();

    while !wifi_driver.is_connected().unwrap(){
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }

    let ip_info = wifi_driver.sta_netif().get_ip_info().unwrap();
    log::info!("Wi-Fi connected, IP: {:?}", ip_info.ip);

    let listener = TcpListener::bind("0.0.0.0:8080")?;
    log::info!("TCP server listening on 0.0.0.0:8080");


    // --- 2. SNTPサービスによる時刻同期 ---
    log::info!("Initializing SNTP...");
    let sntp = EspSntp::new_default()?;

    log::info!("Waiting for time synchronization...");
    while sntp.get_sync_status() != SyncStatus::Completed {
        //FreeRtos::delay_ms(5);
        sleep(Duration::from_millis(100));
    }
    log::info!("Time synchronized successfully!");

    loop {
            // タイムゾーンを日本標準時 (JST) に設定
            // POSIX TZフォーマットでは、UTCからのオフセットの符号が逆になることに注意
            // JSTはUTC+9だが、"JST-9"と指定する
            env::set_var("TZ", "JST+9");
            unsafe { tzset() }
            // gettimeofdayを呼び出してUNIXタイムスタンプを取得
            let mut tv = timeval{
                tv_sec: 0,
                tv_usec: 0,
            };
            unsafe {
                gettimeofday(&mut tv, std::ptr::null_mut());
            }
            // chronoを使用して人間が可読な形式に変換
            let dt = chrono::DateTime::from_timestamp(tv.tv_sec, tv.tv_usec as u32 * 1000)
               .expect("Invalid timestamp");
            // フォーマットして表示

            display.clear();
            let style = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(1)
            .build();

            //Rectangle::new(Point::new(2, 2), Size::new(3, 3))
            //.into_styled(style)
            //.draw(&mut display)
            //.map_err(|e| anyhow::anyhow!("Draw rectangle error: {:?}", e))?;

            //Circle::new(Point::new(0, 0), diameter).into_styled(style).draw(display).map_err(|e| anyhow::anyhow!("Draw rectangle error: {:?}", e))?;

            let text_style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);
            Text::new(format!("{}", dt.format("%Y-%m-%d")).as_str(), Point::new(10, 25), text_style)
            .draw(&mut display)
            .map_err(|e| anyhow::anyhow!("Draw text error: {:?}", e))?;
            Text::new(format!("{}", dt.format("%H:%M:%S %Z")).as_str(), Point::new(10, 39), text_style)
            .draw(&mut display)
            .map_err(|e| anyhow::anyhow!("Draw text error: {:?}", e))?;

            display.flush().map_err(|e| anyhow::anyhow!("Display flush error: {:?}", e))?;
            FreeRtos::delay_ms(500);
    }
}

fn neopixel(rgb: Rgb, tx: &mut TxRmtDriver) -> anyhow::Result<()> {
    let color: u32 = rgb.into();
    let ticks_hz = tx.counter_clock()?;

    let (t0h, t0l, t1h, t1l) = (
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
    );

    let mut signal = FixedLengthSignal::<24>::new();

    for i in (0..24).rev() {
        let p = 2_u32.pow(i);
        let bit: bool = p & color != 0;

        let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };

        signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }

    tx.start_blocking(&signal)?;

    Ok(())
}

struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb> for u32 {
    /// Convert RGB to u32 color value
    ///
    /// e.g. rgb: (1,2,4)
    /// G R B
    /// 7 0 7 0 7 0
    /// 00000010 00000001 00000100
    fn from(rgb: Rgb) -> Self {
        ((rgb.g as u32) << 16) | ((rgb.r as u32) << 8) | rgb.b as u32
    }
}


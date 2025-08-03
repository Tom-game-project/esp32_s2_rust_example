//use std::thread;
//use std::time::Duration;

//esp_idf_svc::hal::prelude::Periferal;
//use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;

// esp idf hal
use esp_idf_hal::gpio;
use esp_idf_hal::delay::{BLOCK};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::{config, UartDriver};

fn main() -> anyhow::Result<()>  {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    //let mut led = PinDriver::output(peripherals.pins.gpio2)?;
    // 2. UARTピンの指定
    // UART1を使用 (UART0はデバッグ/書き込み用)
    let tx = peripherals.pins.gpio17; // U1TXD
    let rx = peripherals.pins.gpio18; // U1RXD

    // 3. UARTのコンフィグレーション設定
    // ボーレートを115200 bpsに設定。PC側のターミナルソフトも同じ設定にする必要があります。
    let config = config::Config::new().baudrate(Hertz(115_200));

    // 4. UartDriverのインスタンス化
    // UART1ペリフェラル、TX/RXピン、コンフィグを渡してドライバを作成します。
    // CTS/RTS (フロー制御)は使用しないためNoneを指定します。
    let uart = UartDriver::new(
        peripherals.uart1,
        tx,
        rx,
        Option::<gpio::Gpio16>::None, // RTS
        Option::<gpio::Gpio15>::None, // CTS
        &config,
    )?;

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    loop {
        //led.set_high()?;
        //FreeRtos::delay_ms(1000);
        //led.set_low()?;
        //FreeRtos::delay_ms(1000);
        // 1バイト分のバッファを用意
        let mut buf = [0_u8; 1];

        // データが受信されるまでブロック(待機)して読み込む
        uart.read(&mut buf, BLOCK)?;

        // 読み込んだデータをコンソール(UART0)に出力して確認
        //println!("Received: 0x{:02x} ('{:?}')", buf, buf);

        // 修正後
        println!("Received: ('{:?}')", buf);

        // 読み込んだデータをそのまま書き戻す(エコー)
        //uart.write(&buf)?;

    }
}

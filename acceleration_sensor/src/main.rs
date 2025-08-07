//! MMA7660FC I2Cドライバを使用したESP32でのサンプルコード
//!
//! このコードは、I2C経由でMMA7660FC加速度センサーを初期化し、
//! 1秒ごとにX, Y, Z軸の加速度を読み取ってコンソールに出力します。

// 作成したドライバをモジュールとして読み込みます
mod mma7660fc;

//use std::thread;
//use std::time::Duration;

//esp_idf_svc::hal::prelude::Periferal;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::units::FromValueType;


use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use mma7660fc::{Mma7660fc, Mode, DEFAULT_I2C_ADDRESS};

fn main() -> anyhow::Result<()> {
    // ランタイムのパッチをリンクします
    esp_idf_svc::sys::link_patches();
    // ログ機能を初期化します
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("MMA7660FCセンサーのサンプルを開始します");

    // ペリフェラルを取得します
    let peripherals = Peripherals::take()?;

    // --- I2Cの初期化 ---
    let i2c = peripherals.i2c0;

    let sda = peripherals.pins.gpio8;
    let scl = peripherals.pins.gpio9;

    // I2Cドライバを設定します
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;

    // --- センサーの初期化 ---
    // 作成したI2Cドライバを使って、MMA7660FCドライバを初期化します
    let mut sensor = Mma7660fc::new(i2c_driver, DEFAULT_I2C_ADDRESS);

    // センサーをアクティブモードに設定します
    log::info!("センサーをアクティブモードに設定します...");
    match sensor.set_mode(Mode::Active) {
        Ok(_) => log::info!("センサーはアクティブです"),
        Err(e) => {
            // anyhow::Errorに変換するために具体的なエラー型を文字列にする
            let error_str = format!("センサーのモード設定に失敗しました: {:?}", e);
            return Err(anyhow::anyhow!(error_str));
        }
    }
    
    FreeRtos::delay_ms(100); // モード変更が安定するまで少し待機

    // --- メインループ ---
    loop {
        // 加速度データを取得します
        match sensor.get_acceleration() {
            Ok(accel) => {
                // 取得した値をログに出力します
                log::info!("加速度: x={}, y={}, z={}", accel.x, accel.y, accel.z);
            }
            Err(e) => {
                log::error!("加速度の読み取りに失敗しました: {:?}", e);
            }
        }

        // 1秒待機
        FreeRtos::delay_ms(1000);
    }
}


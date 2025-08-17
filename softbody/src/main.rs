use embedded_graphics::{
    mono_font::{ascii::* , MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::prelude::Peripherals;

use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};

use esp_idf_svc::hal::units::FromValueType;

use softbody::core::{Simulation, SimulationConfig, SoftBodyConfig, Vec2};

use esp32s2_common_lib::sh1106_display::set_sh1106_display;
use esp32s2_common_lib::mma7660fc::{Mode, DEFAULT_I2C_ADDRESS, Mma7660fc};

fn create_simulation02_small() -> Simulation {
    let sim_width = 126.0;
    let sim_height = 60.0;

    let sim_config = SimulationConfig {
        bounds: Some((Vec2::new(0.0, 0.0), Vec2::new(sim_width, sim_height))),
        gravity: Vec2::new(0.0, 0.0),
        solver_iterations: 6,
        damping: 0.99,
        use_volumetric_collisions: true,
        ..Default::default()
    };
    
    let mut sim = Simulation::new(sim_config);

    let grid_cols = 2;
    let grid_rows = 2;
    let cube_size = 9.0;
    let spacing = 15.0;
    let start_x = (sim_width - (grid_cols - 1) as f64 * spacing) / 2.0;
    let start_y = (sim_height - (grid_rows - 1) as f64 * spacing) / 2.0;
    
    for i in 0..grid_rows {
        for j in 0..grid_cols {
            let x = start_x + j as f64 * spacing;
            let y = start_y + i as f64 * spacing;
            let cube_config = SoftBodyConfig {
                center: Vec2::new(x, y),
                size: Vec2::new(cube_size, cube_size),
                rows: 3, cols: 3, 
                stiffness: 0.6, 
                shape_stiffness: 0.5, 
                particle_radius: 2.5,
                ..Default::default()
            };
            sim.add_soft_body(&cube_config);
        }
    }
    
    sim
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

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

    // 加速度センサの起動
    log::info!("MMA7660FCセンサーのサンプルを開始します");

    // ペリフェラルを取得します
    // --- I2Cの初期化 ---
    let i2c = peripherals.i2c0;

    let sda = peripherals.pins.gpio8;
    let scl = peripherals.pins.gpio9;

    log::info!("i2cの初期化とペリフェラルの取得完了");
    // I2Cドライバを設定します
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;

    log::info!("i2cの設定完了");
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

    // ここから、softbodyの設定

    
    let mut sim = create_simulation02_small();
    //sim.add_soft_body(&fixed_anchor);

    loop {
        display.clear();
        match sensor.get_acceleration() {
            Ok(accel) => {
                // 取得した値をログに出力します
                log::info!("加速度: x={}, y={}, z={}", accel.x, accel.y, accel.z);
                let new_gravity = Vec2::new(-accel.y as f64, -accel.z as f64) * 50.0;
                sim.config_mut().gravity = new_gravity;
                let style = PrimitiveStyleBuilder::new()
                    .stroke_color(BinaryColor::On)
                    .stroke_width(1)
                    .build();

                sim.step(0.025);

                for p in &sim.particles {
                    Circle::new(Point::new(p.pos.x as i32, p.pos.y as i32), 4)
                    .into_styled(style)
                    .draw(&mut display)
                    .map_err(|e| anyhow::anyhow!("Draw text error: {:?}", e))?;
                }

                display
                    .flush()
                    .map_err(|e| anyhow::anyhow!("Display flush error: {:?}", e))?;
            }
            Err(e) => {
                log::error!("加速度の読み取りに失敗しました: {:?}", e);
            }
        }
        FreeRtos::delay_ms(5);
    }
}

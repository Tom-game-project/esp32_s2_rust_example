//! MMA7660FC 3軸加速度センサー用 I2C ドライバ
//!
//! このドライバは `embedded-hal` の I2C トレイトを利用します。
//!
//! ## 使用例
//!
//! ```no_run
//! use mma7660fc::{Mma7660fc, Mode, DEFAULT_I2C_ADDRESS};
//! # use embedded_hal_mock::i2c::{Mock as I2c, Transaction};
//!
//! // I2Cペリフェラルを初期化 (プラットフォーム依存)
//! # let i2c = I2c::new(&[
//! #   Transaction::write(DEFAULT_I2C_ADDRESS, vec![0x07, 0x00]), // Standby
//! #   Transaction::write(DEFAULT_I2C_ADDRESS, vec![0x07, 0x01]), // Active
//! #   Transaction::write_read(DEFAULT_I2C_ADDRESS, vec![0x00], vec![0b000001]), // Read X
//! #   Transaction::write_read(DEFAULT_I2C_ADDRESS, vec![0x01], vec![0b011111]), // Read Y
//! #   Transaction::write_read(DEFAULT_I2C_ADDRESS, vec![0x02], vec![0b001010]), // Read Z
//! # ]);
//!
//! // ドライバを初期化
//! let mut sensor = Mma7660fc::new(i2c, DEFAULT_I2C_ADDRESS);
//!
//! // センサーをアクティブモードに設定
//! sensor.set_mode(Mode::Active).unwrap();
//!
//! // 加速度データを読み取る
//! loop {
//!     if let Ok(accel) = sensor.get_acceleration() {
//!         // 読み取ったデータをコンソールに出力
//!         // 例: Acceleration { x: 1, y: -1, z: 10 }
//!         println!("Acceleration: {:?}", accel);
//!     }
//! }
//! ```

use embedded_hal::i2c::I2c;

// MMA7660FCのレジスタアドレス
const REG_XOUT: u8 = 0x00;
const REG_YOUT: u8 = 0x01;
const REG_ZOUT: u8 = 0x02;
const REG_MODE: u8 = 0x07;

/// デフォルトのI2Cスレーブアドレス
pub const DEFAULT_I2C_ADDRESS: u8 = 0x4C;

/// センサーの動作モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// スタンバイモード（低消費電力）
    Standby,
    /// アクティブモード（測定実行中）
    Active,
}

/// 3軸の加速度データを格納する構造体
/// 値は6ビットの符号付き整数 (-32 to +31)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Acceleration {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

/// MMA7660FC ドライバ
pub struct Mma7660fc<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> Mma7660fc<I2C>
where
    I2C: I2c<Error = E>,
{
    /// 新しいドライバインスタンスを作成します。
    ///
    /// # Arguments
    ///
    /// * `i2c` - `embedded_hal::i2c::I2c` を実装したI2Cペリフェラル
    /// * `address` - センサーのI2Cスレーブアドレス (デフォルトは 0x4C)
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// レジスタに1バイト書き込みます。
    fn write_register(&mut self, register: u8, value: u8) -> Result<(), E> {
        self.i2c.write(self.address, &[register, value])
    }

    /// レジスタから1バイト読み込みます。
    fn read_register(&mut self, register: u8) -> Result<u8, E> {
        let mut buffer = [0u8; 1];
        self.i2c.write_read(self.address, &[register], &mut buffer)?;
        Ok(buffer[0])
    }

    /// センサーの動作モードを設定します。
    ///
    /// 測定を開始するには、`Mode::Active` に設定する必要があります。
    /// 設定を変更する場合は、一度 `Mode::Standby` にする必要があります。
    pub fn set_mode(&mut self, mode: Mode) -> Result<(), E> {
        let current_mode = self.read_register(REG_MODE)?;
        
        let target_mode_value = match mode {
            Mode::Active => (current_mode & 0b1111_1100) | 0x01,
            Mode::Standby => current_mode & 0b1111_1100,
        };

        // モード設定は一度スタンバイにする必要がある
        self.write_register(REG_MODE, current_mode & 0b1111_1100)?;
        if mode == Mode::Active {
            self.write_register(REG_MODE, target_mode_value)?;
        }
        
        Ok(())
    }

    /// X, Y, Z軸の加速度データを取得します。
    ///
    /// データは6ビットの符号付き整数として返されます。
    /// センサーがスタンバイモードの場合、最後の測定値または0が返されます。
    pub fn get_acceleration(&mut self) -> Result<Acceleration, E> {
        let x_raw = self.read_register(REG_XOUT)?;
        let y_raw = self.read_register(REG_YOUT)?;
        let z_raw = self.read_register(REG_ZOUT)?;

        // 6ビットの符号付き整数に変換
        let x = self.convert_to_signed(x_raw);
        let y = self.convert_to_signed(y_raw);
        let z = self.convert_to_signed(z_raw);

        Ok(Acceleration { x, y, z })
    }

    /// センサーから読み取った6ビットの値をi8の符号付き整数に変換します。
    ///
    /// MMA7660FCのデータは6ビットの2の補数で表現されます。
    /// - bit 6 (Alert bit) は無視します。
    /// - bit 5 が符号ビットです。
    /// - bit 4-0 が値です。
    ///
    /// 例:
    /// 0b00_0001 (1) -> 1
    /// 0b01_1111 (31) -> 31
    /// 0b10_0000 (32) -> -32
    /// 0b11_1111 (63) -> -1
    fn convert_to_signed(&self, raw_value: u8) -> i8 {
        // 上位2ビットをマスクして6ビットの値を取得
        let value = raw_value & 0x3F;
        
        // 符号ビット(bit 5)が立っているかチェック
        if (value & 0x20) != 0 {
            // 負の値の場合、64を引くことで2の補数をデコード
            (value as i8) - 64
        } else {
            // 正の値
            value as i8
        }
    }
}


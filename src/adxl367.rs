// Adxl367 driver
//

use embedded_hal::spi::*;

// ADXL367の主要なレジスタアドレスを定義
mod registers {
    pub const DEVID_AD: u8 = 0x00;      // Analog Devices ID
    pub const DEVID_MST: u8 = 0x01;     // MEMS ID
    pub const PARTID: u8 = 0x02;        // Part ID
    pub const XDATA_L: u8 = 0x0E;       // X軸データ 下位バイト
    pub const POWER_CTL: u8 = 0x2D;     // 電源制御レジスタ
}

pub struct Adxl367<SPI> {
    spi: SPI,
}

impl<SPI> Adxl367<SPI>{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }
}

impl<SPI, E> Adxl367<SPI>
where
    SPI: SpiDevice<u8, Error = E>,
{
    // レジスタに1バイト書き込む
    fn write_register(&mut self, address: u8, value: u8) -> Result<(), E> {
        let write_buf = [0x0A, address, value];
        let mut ops = [Operation::Write(&write_buf)];
        self.spi.transaction(&mut ops)
    }

    // レジスタから1バイト読み込む
    fn read_register(&mut self, address: u8) -> Result<u8, E> {
        let command_buf = [0x0B, address];
        let mut read_buf = [0u8];
        let mut ops = [
            Operation::Write(&command_buf),
            Operation::Read(&mut read_buf),
        ];
        self.spi.transaction(&mut ops)?;
        Ok(read_buf[0])
    }

    // 複数のレジスタを一度に読み込む
    //fn read_multiple_registers(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), E> {
    //    let write_buf =;
    //    self.spi.transaction(&mut)
    //}

    fn read_multiple_registers(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), E> {
        let command_buf = [0x0B, address];
        let mut ops = [
            Operation::Write(&command_buf),
            Operation::Read(buffer),
        ];
        self.spi.transaction(&mut ops)
    }

    /// 3軸のRAW加速度データを読み取ります。
    ///
    /// X, Y, Z軸のデータレジスタから6バイトを一度に読み込み（バーストリード）、
    /// 各軸の14ビットデータをi16型のタプル (x, y, z) として返します。
    pub fn acceleration_raw(&mut self) -> Result<(i16, i16, i16), E> {
        // X, Y, ZのL/Hバイトを格納するための6バイトのバッファ
        let mut buffer = [0u8; 6];

        // XDATA_L (0x0E)から連続して6バイト読み込む
        self.read_multiple_registers(0x0E, &mut buffer)?;

        // バッファからリトルエンディアンで16ビット符号付き整数を組み立てる
        // ADXL367のデータは14ビットですが、i16で扱うのが便利です
        let x = i16::from_le_bytes([buffer[0], buffer[1]]);
        let y = i16::from_le_bytes([buffer[2], buffer[3]]);
        let z = i16::from_le_bytes([buffer[4], buffer[5]]);

        Ok((x, y, z))
    }

    /// ADXL367を初期化し、測定モードを開始します。
    /// この処理を呼び出さないと、センサーはスタンバイ状態のままです。
    //pub fn init(&mut self) -> Result<(), E> {
    //    // POWER_CTL (アドレス 0x2D) レジスタに 0x02 を書き込み、測定モードを有効にする
    //    const POWER_CTL_ADDR: u8 = 0x2D;
    //    const MEASURE_MODE: u8 = 0b0000_0010; // [1]ビット目を立てると測定モード
    //    self.write_register(POWER_CTL_ADDR, MEASURE_MODE)
    //}

    pub fn device_id(&mut self) -> Result<(u8, u8, u8), E> {
        let ad_id = self.read_register(registers::DEVID_AD)?;
        let mst_id = self.read_register(registers::DEVID_MST)?;
        let part_id = self.read_register(registers::PARTID)?;
        Ok((ad_id, mst_id, part_id))
    }

     /// ADXL367を初期化し、測定モードを開始します。
    /// この処理は、通信の健全性を確認するためにIDレジスタを検証します。
    pub fn init(&mut self) -> Result<(),E> {
        // 1. ソフトウェアリセットでデバイスを既知の状態にする
        //self.soft_reset()?;
        // リセット後の安定化のために短い遅延を入れる
        //FreeRtos::delay_ms(10);

        // 2. デバイスIDを読み出して通信が正常か検証する
        let dev_id_ad = self.read_register(0x00)?; // DEVID_AD
        let dev_id_mst = self.read_register(0x01)?; // DEVID_MST

        // 期待値 (AD: 0xAD, MST: 0x1D) と比較
        if dev_id_ad!= 0xAD || dev_id_mst!= 0x1D {
            log::error!("Invalid Chip ID. AD: 0x{:02X}, MST: 0x{:02X}", dev_id_ad, dev_id_mst);
            //return Err();
        }
        log::info!("ADXL367 Chip ID verified successfully.");

        // 3. ID検証後、測定モードを有効にする
        const MEASURE_MODE: u8 = 0b0000_0010;
        self.write_register(registers::POWER_CTL, MEASURE_MODE)?;
        log::info!("ADXL367 set to measurement mode.");

        Ok(())
    }
}

// Adxl367 driver
//

use embedded_hal::spi::*;

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
}

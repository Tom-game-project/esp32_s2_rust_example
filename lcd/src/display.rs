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
use crate::gpio::OutputPin;
use crate::gpio::Output;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::spi;

/// 任意のGPIOピンとSPIペリフェラルを受け取り、ディスプレイの初期化を行う
pub fn set_display<'d, RST, DC, SCLK, SDA, CS, SPI>(
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
    RST: Peripheral<P = RST> + OutputPin,
    DC: Peripheral<P = DC> + OutputPin,
    CS: Peripheral<P = CS> + OutputPin,
    SCLK: Peripheral<P = SCLK> + OutputPin,
    SDA: Peripheral<P = SDA> + OutputPin,
    SPI: Peripheral<P = SPI> + spi::Spi + esp_idf_svc::hal::spi::SpiAnyPins + 'd,
{
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

    let spi_device = SpiDeviceDriver::new(
        spi_driver,
        None::<AnyIOPin>,
        &spi_config)?;

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

use arduino_nano33iot as bsp;
use bsp::hal;
use hal::clock::GenericClockController;
use hal::gpio::v2::pin;
use hal::time::KiloHertz;

use lsm6ds33::{AccelerometerOutput, GyroscopeOutput, Lsm6ds33};

type I2C = hal::sercom::v1::I2CMaster4<
    pin::Pin<pin::PB08, pin::Alternate<pin::D>>,
    pin::Pin<pin::PB09, pin::Alternate<pin::D>>,
>;

pub fn setup_gyro(
    clocks: &mut GenericClockController,
    sercom: bsp::pac::SERCOM4,
    pm: &mut bsp::pac::PM,
    sda: impl Into<bsp::Sda>,
    scl: impl Into<bsp::Scl>,
) -> Result<Lsm6ds33<I2C>, &'static str> {
    let i2c = bsp::i2c_master(clocks, KiloHertz(100), sercom, pm, sda, scl);
    match Lsm6ds33::new(i2c, 0x6A) {
        Ok(mut gyro) => {
            if gyro.set_gyroscope_output(GyroscopeOutput::Rate104).is_err() {
                return Err("Failed to setup gyroscope output rate");
            }
            if gyro
                .set_accelerometer_output(AccelerometerOutput::Rate104)
                .is_err()
            {
                return Err("Failed to setup accelerometer output rate");
            }
            Ok(gyro)
        }
        _ => Err("Gyroscope setup failed"),
    }
}

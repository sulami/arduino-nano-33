use core::time::Duration;

use arduino_nano33iot as bsp;
use bsp::hal;
use bsp::hal::gpio::v2::{pin, Alternate, Input, Output, Pin, Reset};

use hal::sercom::v2::spi;
use wifi_nina::transport;

pub struct WifiPins {
    pub miso: Pin<pin::PA13, Reset>,
    pub mosi: Pin<pin::PA12, Reset>,
    pub sck: Pin<pin::PA15, Reset>,
    pub ack: Pin<pin::PA28, Reset>,
    pub rst: Pin<pin::PA08, Reset>,
    pub cs: Pin<pin::PA14, Reset>,
}

type Spi<DELAY> = transport::SpiTransport<
    spi::Spi<
        spi::Config<
            spi::Pads<
                hal::pac::SERCOM2,
                Pin<pin::PA13, Alternate<hal::gpio::v2::C>>,
                Pin<pin::PA12, Alternate<hal::gpio::v2::C>>,
                Pin<pin::PA15, Alternate<hal::gpio::v2::C>>,
            >,
        >,
        spi::Duplex,
    >,
    Pin<pin::PA28, Input<hal::gpio::v2::Floating>>,
    Pin<pin::PA08, Output<hal::gpio::v2::PushPull>>,
    Pin<pin::PA14, Output<hal::gpio::v2::PushPull>>,
    DELAY,
>;

pub fn setup_wifi<DELAY: FnMut(Duration)>(
    clock: &hal::thumbv6m::clock::Sercom2CoreClock,
    pm: &hal::pac::PM,
    sercom: hal::pac::SERCOM2,
    pins: WifiPins,
    sleep: DELAY,
) -> wifi_nina::Wifi<Spi<DELAY>> {
    let pads = spi::Pads::default()
        .data_in(pins.miso)
        .data_out(pins.mosi)
        .sclk(pins.sck);
    let spi_master = spi::Config::new(pm, sercom, pads, clock.freq())
        .baud(hal::time::MegaHertz(8))
        .spi_mode(spi::MODE_0)
        .enable();
    let transport = transport::SpiTransport::start(
        spi_master,
        pins.ack.into_floating_input(),
        pins.rst.into_push_pull_output(),
        pins.cs.into_push_pull_output(),
        sleep,
    )
    .unwrap();
    wifi_nina::Wifi::new(transport)
}

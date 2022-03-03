use core::time::Duration;

use arduino_nano33iot as bsp;
use bsp::hal;
use bsp::hal::gpio::v2::pin;

use hal::sercom::v2::spi;
use wifi_nina::transport;

pub struct WifiPins {
    pub miso: pin::Pin<pin::PA13, hal::gpio::v2::Reset>,
    pub mosi: pin::Pin<pin::PA12, hal::gpio::v2::Reset>,
    pub sck: pin::Pin<pin::PA15, hal::gpio::v2::Reset>,
    pub ack: pin::Pin<pin::PA28, hal::gpio::v2::Reset>,
    pub rst: pin::Pin<pin::PA08, hal::gpio::v2::Reset>,
    pub cs: pin::Pin<pin::PA14, hal::gpio::v2::Reset>,
}

type Spi<DELAY> = wifi_nina::transport::SpiTransport<
    hal::sercom::v2::spi::Spi<
        hal::sercom::v2::spi::Config<
            hal::sercom::v2::spi::Pads<
                hal::pac::SERCOM2,
                hal::gpio::v2::Pin<pin::PA13, hal::gpio::v2::Alternate<hal::gpio::v2::C>>,
                hal::gpio::v2::Pin<pin::PA12, hal::gpio::v2::Alternate<hal::gpio::v2::C>>,
                hal::gpio::v2::Pin<pin::PA15, hal::gpio::v2::Alternate<hal::gpio::v2::C>>,
            >,
        >,
        hal::sercom::v2::spi::Duplex,
    >,
    hal::gpio::v2::Pin<pin::PA28, hal::gpio::v2::Input<hal::gpio::v2::Floating>>,
    hal::gpio::v2::Pin<pin::PA08, hal::gpio::v2::Output<hal::gpio::v2::PushPull>>,
    hal::gpio::v2::Pin<pin::PA14, hal::gpio::v2::Output<hal::gpio::v2::PushPull>>,
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

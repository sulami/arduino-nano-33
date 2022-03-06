# Arduino Nano 33 IoT

This project is a starting platform to build applications on the
Arduino Nano 33 IoT. It comes preconfigured with WiFi, Gyroscope, and
serial via USB.

## Building

### Install dependencies:

```sh
arduino-cli core install arduino:samd
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

### Build & flash:

```sh
cargo build --release
rust-objcopy -O binary target/thumbv6m-none-eabi/release/arduino-nano-33-iot target/arduino.bin
arduino-cli upload -i target/arduino.bin -b arduino:samd:nano_33_iot -p /dev/tty.usbmodem144401
```

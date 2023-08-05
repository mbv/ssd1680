# SSD1680 ePaper Display Driver

Rust driver for the [Solomon Systech SSD1680][SSD1680] e-Paper display (EPD)
controller, for use with [embedded-hal].

[![crates.io](https://img.shields.io/crates/v/ssd1680.svg)](https://crates.io/crates/ssd1680)
[![Documentation](https://docs.rs/ssd1680/badge.svg)](https://docs.rs/ssd1680/)


## Description

This driver is written for a [WeAct Studio Epaper Module 2.13" Tri-Color][tricolor] display.
It will probably work for other displays with the same chip.

It is built using [embedded-hal] and optionally
[embedded-graphics]. 

## Examples
* [ESP32-C3](https://github.com/mbv/esp32-ssd1680)

## Partial updates
Partial updates is not supported. There was support initially but
the driver refreshes the whole screen so there is no point.

## Credits

* [Arduino Display Library for SPI E-Paper Displays](https://github.com/ZinggJM/GxEPD2)
* [SSD1681 EPD driver](https://github.com/afajl/ssd1681)

## License

`ssd1680` is dual licenced under:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) **or**
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

[embedded-hal]: https://crates.io/crates/embedded-hal
[embedded-graphics]: https://github.com/embedded-graphics/embedded-graphics
[LICENSE-APACHE]: https://github.com/mbv/ssd1675/blob/master/LICENSE-APACHE
[LICENSE-MIT]: https://github.com/mbv/ssd1675/blob/master/LICENSE-MIT
[SSD1680]: https://www.solomon-systech.com/product/ssd1680
[hardware problem]: https://forums.adafruit.com/viewtopic.php?f=47&t=146252&p=722909&hilit=partial+update#p722957.
[tricolor]: https://www.aliexpress.com/item/1005004644515880.html


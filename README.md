# SSD1327 I2C Driver

`no_std` I2C Driver for SSD1327 Oled screens.

The `graphics` feature implements the [embedded-graphics](https://crates.io/crates/embedded-graphics) `DrawTarget` trait for the SSD1327 Oled screen.

Tested on an ESP32.

## Examples

### Without Graphics
Following code shows how to flash a SSD1327 screen using the ESP HAL I2C Peripheral Driver.

```rust
// Create a new peripheral object with the described wiring
// and standard I2C clock speed
let i2c = I2C::new(
    peripherals.I2C0,
    sda,
    scl,
    100u32.kHz(),
    &clocks,
);

// Create a new SSD1327I2C object with slave address 0x3C, width 127 and height 127
let mut driver = ssd1327_i2c::SSD1327I2C::new(i2c);

driver.init();

loop {
    driver.send_cmd(ssd1327_i2c::Commands::DisplayModeAllON);
    delay.delay_ms(1000u32);
    driver.send_cmd(ssd1327_i2c::Commands::DisplayModeAllOFF);
    delay.delay_ms(1000u32);
}
```

### With Graphics
Following code shows how to write `Hello rust!` to a SSD1327 screen using the ESP HAL I2C Peripheral Driver.

```rust
// Create a new peripheral object with the described wiring
// and standard I2C clock speed
let i2c = I2C::new(
    peripherals.I2C0,
    sda,
    scl,
    100u32.kHz(),
    &clocks,
);

// Create a new SSD1327I2C object with slave address 0x3C, width 127 and height 127
let mut driver = ssd1327_i2c::SSD1327I2C::new(i2c);

driver.init();

// Create a new character style
let style = MonoTextStyle::new(&FONT_6X10, Gray4::WHITE);

// Create a text at position (10, 10) and draw it using the previously defined style
Text::new("Hello rust!", Point::new(10, 10), style).draw(&mut driver).unwrap();

loop {}
```
//! # SSD1327 I2C driver
//!
//! `no_std` I2C Driver for SSD1327 Oled screens.
//!
//! ## Example
//! Following code shows how to flash a SSD1327 screen using the ESP HAL I2C Peripheral Driver.
//!
//! ```ignore
//! // Create a new peripheral object with the described wiring
//! // and standard I2C clock speed
//! let i2c = I2C::new(
//!     peripherals.I2C0,
//!     sda,
//!     scl,
//!     100u32.kHz(),
//!     &clocks,
//! );
//!
//! // Create a new SSD1327I2C object with slave address 0x3C, width 128 and height 128
//! let mut driver = ssd1327_i2c::SSD1327I2C::new(i2c);
//!
//! driver.init();
//!
//! loop {
//!     driver.send_cmd(ssd1327_i2c::Commands::DisplayModeAllON);
//!     delay.delay_ms(1000u32);
//!     driver.send_cmd(ssd1327_i2c::Commands::DisplayModeAllOFF);
//!     delay.delay_ms(1000u32);
//! }
//! ```

#![no_std]
use core::{result::Result, usize};
use embedded_hal::blocking::i2c::Write as I2CWrite;

#[cfg(feature = "graphics")]
use embedded_graphics_core::{
    draw_target::DrawTarget, geometry::OriginDimensions, geometry::Size, pixelcolor::Gray4,
    pixelcolor::GrayColor, Pixel,
};

/// Default slave address for SSD1327
const DEFAULT_SLAVE_ADDRESS: u8 = 0x3C;
/// Command control byte for SSD1327
const CMD_CONTROL_BYTE: u8 = 0x00; // Don't know why it's not 0x80
/// Data control byte for SSD1327
const DATA_CONTROL_BYTE: u8 = 0x40;

/// Calculates the buffer size for a given screen width and height
///
/// ```
/// # use ssd1327_i2c::buffer_size;
/// let size = buffer_size(128, 128);
/// assert_eq!(size, 8192);
/// ```
#[must_use]
#[inline]
pub const fn buffer_size(width: u8, height: u8) -> usize {
    let halfwidth = width as usize / 2; // Two pixels per byte
    halfwidth * height as usize
}

/// SSD1327 I2C driver container
pub struct SSD1327I2C<I2C, #[cfg(feature = "graphics")] const N: usize>
where
    I2C: I2CWrite,
{
    i2c: I2C,
    slave_address: u8,
    halfwidth: u8,
    width: u8,
    height: u8,
    #[cfg(feature = "graphics")]
    framebuffer: [u8; N],
}

/// Create a new SSD1327I2C object with custom width and height
#[cfg(feature = "graphics")]
#[macro_export]
macro_rules! build_ssd1327_i2c {
    ($i2c:expr, $addr:expr, $w:expr, $h:expr) => {{
        const SSD1327_I2C_BUFFER_SIZE: usize = $crate::buffer_size($w as u8, $h as u8);
        $crate::SSD1327I2C::<_, SSD1327_I2C_BUFFER_SIZE>::with_addr_wh($i2c, $addr, $w, $h)
    }};
    ($i2c:expr, $addr:expr) => {{
        const SSD1327_I2C_BUFFER_SIZE: usize = $crate::buffer_size(128 as u8, 128 as u8);
        $crate::SSD1327I2C::<_, SSD1327_I2C_BUFFER_SIZE>::with_addr($i2c, $addr)
    }};
    ($i2c:expr, $w:expr, $h:expr) => {{
        const SSD1327_I2C_BUFFER_SIZE: usize = $crate::buffer_size($w as u8, $h as u8);
        $crate::SSD1327I2C::<_, SSD1327_I2C_BUFFER_SIZE>::with_wh($i2c, $w, $h)
    }};
    ($i2c:expr) => {{
        const SSD1327_I2C_BUFFER_SIZE: usize = $crate::buffer_size(128 as u8, 128 as u8);
        $crate::SSD1327I2C::<_, SSD1327_I2C_BUFFER_SIZE>::new($i2c)
    }};
}

#[cfg(feature = "graphics")]
macro_rules! impl_ssd1327_i2c {
    () => {
        SSD1327I2C::<I2C, N>
    };
}

#[cfg(not(feature = "graphics"))]
macro_rules! impl_ssd1327_i2c {
    () => {
        SSD1327I2C::<I2C>
    };
}

impl<I2C, #[cfg(feature = "graphics")] const N: usize> impl_ssd1327_i2c!()
where
    I2C: I2CWrite,
{
    /// Create a new SSD1327I2C object with custom slave adress, width and height
    #[must_use]
    pub fn with_addr_wh(i2c: I2C, slave_address: u8, width: u8, height: u8) -> Self {
        #[cfg(feature = "graphics")]
        let framebuffer = [0u8; N];
        let halfwidth = (width - 1) / 2; // Two pixels per byte
        Self {
            i2c,
            slave_address,
            halfwidth,
            width: width - 1,
            height: height - 1,
            #[cfg(feature = "graphics")]
            framebuffer,
        }
    }

    /// Create a new SSD1327I2C object with custom slave address, width 128 and height 128
    #[must_use]
    pub fn with_addr(i2c: I2C, slave_address: u8) -> Self {
        Self::with_addr_wh(i2c, slave_address, 128, 128)
    }

    /// Create a new SSD1327I2C object with slave address 0x3C, and custom width and height
    #[must_use]
    pub fn with_wh(i2c: I2C, width: u8, height: u8) -> Self {
        Self::with_addr_wh(i2c, DEFAULT_SLAVE_ADDRESS, width, height)
    }

    /// Create a new SSD1327I2C object with slave address 0x3C, width 128 and height 128
    #[must_use]
    pub fn new(i2c: I2C) -> Self {
        Self::with_addr_wh(i2c, DEFAULT_SLAVE_ADDRESS, 128, 128)
    }

    /// Initialize the SSD1327
    pub fn init(&mut self) {
        self.send_cmd(Commands::CommandUnlock).ok();
        self.send_cmd(Commands::DisplayOFF).ok();
        self.send_cmd(Commands::ColumnAddress {
            start: 0x00,
            end: self.halfwidth,
        })
        .ok();
        self.send_cmd(Commands::RowAddress {
            start: 0x00,
            end: self.height,
        })
        .ok();
        self.send_cmd(Commands::ContrastControl(0x7f)).ok(); //50% (128/255) RESET 0x7f
        self.send_cmd(Commands::Remap(0x51)).ok();
        self.send_cmd(Commands::DisplayStartLine(0x00)).ok();
        self.send_cmd(Commands::DisplayOffset(0x00)).ok();
        self.send_cmd(Commands::DisplayModeNormal).ok();
        self.send_cmd(Commands::MUXRatio(0x7e)).ok(); // RESET 0x7f
        self.send_cmd(Commands::PhaseLength(0x51)).ok(); // RESET 0x71
        self.send_cmd(Commands::LinearLUT).ok();
        self.send_cmd(Commands::FrontClockDividerOscFreq(0x00)).ok();
        self.send_cmd(Commands::SelectInternalVDD).ok();
        self.send_cmd(Commands::SecondPreChargePeriod(0x04)).ok();
        self.send_cmd(Commands::VCOMH(0x05)).ok();
        self.send_cmd(Commands::PreChargeVoltage(0x05)).ok();
        self.send_cmd(Commands::FunctionSelectionB(0x60)).ok();
        self.send_cmd(Commands::DisplayON).ok();
    }

    /// Write command to the SSD1327
    pub fn send_cmd(&mut self, cmd: Commands) -> Result<(), I2C::Error> {
        let (data, len) = match cmd {
            Commands::ColumnAddress { start, end } => ([CMD_CONTROL_BYTE, 0x15, start, end], 4),
            Commands::RowAddress { start, end } => ([CMD_CONTROL_BYTE, 0x75, start, end], 4),
            Commands::ContrastControl(value) => ([CMD_CONTROL_BYTE, 0x81, value, 0], 3),
            Commands::Remap(value) => ([CMD_CONTROL_BYTE, 0xA0, value, 0], 3),
            Commands::DisplayStartLine(value) => ([CMD_CONTROL_BYTE, 0xA1, value, 0], 3),
            Commands::DisplayOffset(value) => ([CMD_CONTROL_BYTE, 0xA2, value, 0], 3),
            Commands::DisplayModeNormal => ([CMD_CONTROL_BYTE, 0xA4, 0, 0], 2),
            Commands::DisplayModeAllON => ([CMD_CONTROL_BYTE, 0xA5, 0, 0], 2),
            Commands::DisplayModeAllOFF => ([CMD_CONTROL_BYTE, 0xA6, 0, 0], 2),
            Commands::DisplayModeInverseDisplay => ([CMD_CONTROL_BYTE, 0xA7, 0, 0], 2),
            Commands::MUXRatio(value) => ([CMD_CONTROL_BYTE, 0xA8, value, 0], 3),
            Commands::FunctionSelectionA(value) => ([CMD_CONTROL_BYTE, 0xAB, value, 0], 3),
            Commands::SelectExternalVDD => ([CMD_CONTROL_BYTE, 0xAB, 0x00, 0], 3),
            Commands::SelectInternalVDD => ([CMD_CONTROL_BYTE, 0xAB, 0x01, 0], 3),
            Commands::DisplayON => ([CMD_CONTROL_BYTE, 0xAF, 0, 0], 2),
            Commands::DisplayOFF => ([CMD_CONTROL_BYTE, 0xAE, 0, 0], 2),
            Commands::PhaseLength(value) => ([CMD_CONTROL_BYTE, 0xB1, value, 0], 3),
            Commands::FrontClockDividerOscFreq(value) => ([CMD_CONTROL_BYTE, 0xB3, value, 0], 3),
            Commands::GPIO(value) => ([CMD_CONTROL_BYTE, 0xB5, value, 0], 3),
            Commands::SecondPreChargePeriod(value) => ([CMD_CONTROL_BYTE, 0xB6, value, 0], 3),
            Commands::LinearLUT => ([CMD_CONTROL_BYTE, 0xB9, 0, 0], 2),
            Commands::PreChargeVoltage(value) => ([CMD_CONTROL_BYTE, 0xBC, value, 0], 3),
            Commands::VCOMH(value) => ([CMD_CONTROL_BYTE, 0xBE, value, 0], 3),
            Commands::FunctionSelectionB(value) => ([CMD_CONTROL_BYTE, 0xD5, value, 0], 3),
            Commands::SetCommandLock(value) => ([CMD_CONTROL_BYTE, 0xFD, value, 0], 3),
            Commands::CommandUnlock => ([CMD_CONTROL_BYTE, 0xFD, 0x00, 0x12], 4),
            Commands::CommandLock => ([CMD_CONTROL_BYTE, 0xFD, 0x00, 0x16], 4),
        };
        self.send_bytes(&data[0..len])
    }

    /// Write bytes to the SSD1327
    #[inline]
    fn send_bytes(&mut self, bytes: &[u8]) -> Result<(), I2C::Error> {
        self.i2c.write(self.slave_address, bytes)
    }

    /// Write 8 bytes of data to the SSD1327
    pub fn send_data(&mut self, data: &[u8]) -> Result<(), I2C::Error> {
        let (data, len) = (
            [
                DATA_CONTROL_BYTE,
                data[0],
                data[1],
                data[2],
                data[3],
                data[4],
                data[5],
                data[6],
                data[7],
            ],
            9,
        );
        self.send_bytes(&data[0..len])
    }

    /// Write 8 bytes of data to the SSD1327
    #[cfg(feature = "graphics")]
    #[inline]
    fn send_buffer_data(&mut self, index: usize) -> Result<(), I2C::Error> {
        let bytes = [
            DATA_CONTROL_BYTE,
            self.framebuffer[index],
            self.framebuffer[index + 1],
            self.framebuffer[index + 2],
            self.framebuffer[index + 3],
            self.framebuffer[index + 4],
            self.framebuffer[index + 5],
            self.framebuffer[index + 6],
            self.framebuffer[index + 7],
        ];
        self.send_bytes(&bytes)
    }

    /// Update the display with the current framebuffer
    #[cfg(feature = "graphics")]
    pub fn flush(&mut self) -> Result<(), I2C::Error> {
        // Reset display address pointers
        self.send_cmd(Commands::ColumnAddress {
            start: 0x00,
            end: self.halfwidth,
        })
        .ok();
        self.send_cmd(Commands::RowAddress {
            start: 0x00,
            end: self.height,
        })
        .ok();

        // Send buffer data
        let mut res: Result<(), I2C::Error> = Ok(());
        for y in 0..=(usize::from(self.height)) {
            for x in (0..=(usize::from(self.halfwidth))).step_by(8) {
                let start_index = x + y * (usize::from(self.halfwidth) + 1);
                if let Err(e) = self.send_buffer_data(start_index) {
                    res = Err(e);
                }
            }
        }
        res
    }
}


/// Commands to be sent to the SSD1327
#[derive(Clone, Copy)]
pub enum Commands {
    /// Setup Column start and end address (0x15)
    ColumnAddress {
        /// Start address 00-7f (RESET = 00)
        start: u8,
        /// End address 00-3f (RESET = 3F)
        end: u8,
    },
    /// Setup Row start and end address (0x75)
    RowAddress {
        /// Start address 00-7f (RESET = 00)
        start: u8,
        /// End address 00-7f (RESET = 7F)
        end: u8,
    },
    /// Double byte command to select 1 out of 256 contrast steps.
    /// Contrast increases as the value increases (RESET = 7F ) (0x81)
    ContrastControl(u8),
    /// **Re-map setting in Graphic Display Data RAM (GDDRAM)** (0xA0)\
    /// A\[0\] = 0, Disable Column Address Re-map (RESET)\
    /// A\[0\] = 1, Enable Column Address Re-map\
    /// A\[1\] = 0, Disable Nibble Re-map (RESET)\
    /// A\[1\] = 1, Enable Nibble Re-map\
    /// A\[2\] = 0, Enable Horizontal Address Increment (RESET)\
    /// A\[2\] = 1, Enable Vertical Address Increment\
    /// A\[3\] = 0, Reserved (RESET)\
    /// A\[4\] = 0, Disable COM Re-map (RESET)\
    /// A\[4\] = 1, Enable COM Re-map\
    /// A\[5\] = 0, Reserved (RESET)\
    /// A\[6\] = 0, Disable COM Split Odd Even (RESET)\
    /// A\[6\] = 1, Enable COM Split Odd Even\
    /// A\[7\] = 0, Reserved (RESET)
    Remap(u8),
    /// Vertical shift by setting the starting address of display RAM
    /// from 0 ~ 127 (RESET = 00) (0xA1)
    DisplayStartLine(u8),
    /// Set vertical offset by COM from 0 ~ 127 (RESET = 00) (0xA2)
    DisplayOffset(u8),
    /// Normal Display Mode (0xA4)
    DisplayModeNormal,
    /// All ON Display Mode (0xA5)
    DisplayModeAllON,
    /// All OFF Display Mode (0xA6)
    DisplayModeAllOFF,
    /// Inverse Display Display Mode (0xA7)
    DisplayModeInverseDisplay,
    /// Set MUX ratio from 16MUX ~ 128MUX (0xA8)
    MUXRatio(u8),
    /// Function Selection A (0 = external VDD; 1 = internal VDD (RESET)) (0xAB)
    FunctionSelectionA(u8),
    /// External VDD (0xAB 0x00)
    SelectExternalVDD,
    /// Internal VDD (RESET) (0xAB 0x01)
    SelectInternalVDD,
    /// Turn display ON (0xAF)
    DisplayON,
    /// Turn display OFF (0xAE)
    DisplayOFF,
    /// Phase Length (0xB1)
    PhaseLength(u8),
    /// Front Clock Divider / Oscillator Frequency (0xB3)
    FrontClockDividerOscFreq(u8),
    /// GPIO : 00 represents GPIO pin HiZ, input disable (always read as low) ;
    /// 01 represents GPIO pin HiZ, input enable ; 10 represents GPIO pin output Low (RESET) ;
    /// 11 represents GPIO pin output High ; (0xB5)
    GPIO(u8),
    /// Second Pre-charge period of 1~15 DCLK’s (RESET = 0100) (0xB6)
    SecondPreChargePeriod(u8),
    /// The default Lineear Gray Scale table (0xB9)
    LinearLUT,
    /// Set pre-charge voltage level (0xBC)
    PreChargeVoltage(u8),
    /// Set COM deselect voltage level (0xBE)
    VCOMH(u8),
    /// Function Selection B : 0.: Disable second precharge (RESET) ; 1.: Enable second precharge ;
    /// .0: Internal VSL (RESET) ; .1: Enable external VSL ; (0xD5)
    FunctionSelectionB(u8),
    /// MCU protection status 0x16 = Lock ; 0x12 Unlock (RESET) ; (0xFD)
    SetCommandLock(u8),
    /// Unlock OLED driver IC MCU interface from entering commands (RESET) (0xFD 0x12)
    CommandUnlock,
    /// Lock OLED driver IC MCU interface from entering commands (0xFD 0x16)
    CommandLock,
}

#[cfg(feature = "graphics")]
impl<I2C, const N: usize> DrawTarget for SSD1327I2C<I2C, N>
where
    I2C: I2CWrite,
{
    type Color = Gray4;

    type Error = I2C::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            let (x_i32, y_i32): (i32, i32) = coord.into();
            // Check if the pixel coordinates are out of bounds
            if let Ok(x) = usize::try_from(x_i32) {
                if let Ok(y) = usize::try_from(y_i32) {
                    if (x <= usize::from(self.width)) && (y <= usize::from(self.height)) {
                        // Calculate the index in the framebuffer.
                        let index = x / 2 + y * (usize::from(self.halfwidth) + 1);
                        let mut new_byte = color.luma();
                        // 1 byte for 2 pixels so we need to shift the byte by 4 bits if
                        // the x coordinate is even
                        if x % 2 == 0 {
                            new_byte <<= 4;
                            self.framebuffer[index] &= 0x0F;
                        } else {
                            self.framebuffer[index] &= 0xF0;
                        }
                        self.framebuffer[index] |= new_byte;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<I2C, const N: usize> OriginDimensions for SSD1327I2C<I2C, N>
where
    I2C: I2CWrite,
{
    #[inline]
    fn size(&self) -> Size {
        Size::new(u32::from(self.width), u32::from(self.height))
    }
}

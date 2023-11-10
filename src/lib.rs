#![no_std]
use embedded_hal;
use core::result::Result;

/// SSD1327 I2C driver
pub struct SSD1327I2C<I2C>
where 
    I2C: embedded_hal::blocking::i2c::Write
{
    i2c: I2C,
    slave_address : u8,
    width: u8,
    height: u8,
}

impl<I2C> SSD1327I2C<I2C>
where 
    I2C: embedded_hal::blocking::i2c::Write,
{
    /// Create a new SSD1327I2C object with custom slave adress, width and height
    pub fn new_add_wh(i2c : I2C, slave_address : u8, width : u8, height : u8) -> Self {
        SSD1327I2C {
            i2c,
            slave_address,
            width,
            height,
        }
    }

    /// Create a new SSD1327I2C object with custom slave address, width 127 and height 127
    pub fn new_add(i2c : I2C, slave_address : u8) -> Self {
        SSD1327I2C::new_add_wh(i2c, slave_address, 0x7F, 0x7F)
    }

    /// Create a new SSD1327I2C object with slave address 0x3C, and custom width and height
    pub fn new_wh(i2c : I2C, width : u8, height : u8) -> Self {
        SSD1327I2C::new_add_wh(i2c, 0x3C, width, height)
    }

    /// Create a new SSD1327I2C object with slave address 0x3C, width 127 and height 127
    pub fn new(i2c : I2C) -> Self {
        SSD1327I2C::new_add_wh(i2c, 0x3C, 0x7F, 0x7F)
    }

    /// Initialize the SSD1327
    pub fn init(&mut self) {
        self.send_cmd(Commands::DisplayOFF).ok();
        self.send_cmd(Commands::ColumnAddress { start: 0x00, end: self.width }).ok(); //0-128 / should be 3f aka 63 ?
        self.send_cmd(Commands::RowAddress { start: 0x00, end: self.height }).ok(); //0-128 / 0x7f
        self.send_cmd(Commands::ContrastControl(0x80)).ok(); //50% (128/255)
        self.send_cmd(Commands::Remap(0x51)).ok();
        self.send_cmd(Commands::DisplayStartLine(0x00)).ok();
        self.send_cmd(Commands::DisplayOffset(0x00)).ok();
        self.send_cmd(Commands::DisplayModeNormal).ok();
        self.send_cmd(Commands::MUXRatio(0x7f)).ok();
        self.send_cmd(Commands::PhaseLength(0x11)).ok();
        self.send_cmd(Commands::LinearLUT).ok();
        self.send_cmd(Commands::FrontClockDividerOscillatorFrequency(0x01)).ok();
        self.send_cmd(Commands::SelectInternalVDD).ok();
        self.send_cmd(Commands::SecondPreChargePeriod(0x04)).ok();
        self.send_cmd(Commands::VCOMH(0x0F)).ok();
        self.send_cmd(Commands::PreChargeVoltage(0x08)).ok();
        self.send_cmd(Commands::FunctionSelectionB(0x62)).ok();
        self.send_cmd(Commands::CommandUnlock).ok();
        self.send_cmd(Commands::DisplayON).ok();
    }

    /// Write bytes to the SSD1327
    fn send_bytes(&mut self, bytes: &[u8]) -> Result<(), I2C::Error> {
        self.i2c.write(self.slave_address, bytes)
        // match res {
        //     Ok(_v) => println!("Command written successfully"), 
        //     Err(_e) => println!("Error writing command"),
        // }
    }

    /// Write command to the SSD1327
    pub fn send_cmd(&mut self, cmd: Commands) -> Result<(), I2C::Error> {
        // 0x80 = Command
        let (data, len) = match cmd {
            Commands::ColumnAddress { start, end } => ([0x80, 0x15, start, end], 4),
            Commands::RowAddress { start, end } => ([0x80, 0x75, start, end], 4),
            Commands::ContrastControl(value) => ([0x80, 0x81, value, 0], 3),
            Commands::Remap(value) => ([0x80, 0xA0, value, 0], 3),
            Commands::DisplayStartLine(value) => ([0x80, 0xA1, value, 0], 3),
            Commands::DisplayOffset(value) => ([0x80, 0xA2, value, 0], 3),
            Commands::DisplayModeNormal => ([0x80, 0xA4, 0, 0], 2),
            Commands::DisplayModeAllON => ([0x80, 0xA5, 0, 0], 2),
            Commands::DisplayModeAllOFF => ([0x80, 0xA6, 0, 0], 2),
            Commands::DisplayModeInverseDisplay => ([0x80, 0xA7, 0, 0], 2),
            Commands::MUXRatio(value) => ([0x80, 0xA8, value, 0], 3),
            Commands::FunctionSelectionA(value) => ([0x80, 0xAB, value, 0], 3),
            Commands::SelectExternalVDD => ([0x80, 0xAB, 0x00, 0], 3),
            Commands::SelectInternalVDD => ([0x80, 0xAB, 0x01, 0], 3),
            Commands::DisplayON => ([0x80, 0xAF, 0, 0], 2),
            Commands::DisplayOFF => ([0x80, 0xAE, 0, 0], 2),
            Commands::PhaseLength(value) => ([0x80, 0xB1, value, 0], 3),
            Commands::FrontClockDividerOscillatorFrequency(value) => ([0x80, 0xB3, value, 0], 3),
            Commands::GPIO(value) => ([0x80, 0xB5, value, 0], 3),
            Commands::SecondPreChargePeriod(value) => ([0x80, 0xB6, value, 0], 3),
            Commands::LinearLUT => ([0x80, 0xB9, 0, 0], 2),
            Commands::PreChargeVoltage(value) => ([0x80, 0xBC, value, 0], 3),
            Commands::VCOMH(value) => ([0x80, 0xBE, value, 0], 3),
            Commands::FunctionSelectionB(value) => ([0x80, 0xD5, value, 0], 3),
            Commands::SetCommandLock(value) => ([0x80, 0xFD, value, 0], 3),
            Commands::CommandUnlock => ([0x80, 0xFD, 0x80, 0x12], 4),
            Commands::CommandLock => ([0x80, 0xFD, 0x80, 0x16], 4),
        };
        self.send_bytes(&data[0..len])
    }

    /// Write 8 bytes of data to the SSD1327
    pub fn send_data(&mut self, data: &[u8; 8]) -> Result<(), I2C::Error> {
        // 0x40 = Data
        let (data, len) = (
            [0x40, data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]],
            9
        );
        self.send_bytes(&data[0..len])
    }
    
}

/// Commands to be sent to the SSD1327
pub enum Commands {
    /// Setup Column start and end address (0x15)
    ColumnAddress {
        /// Start address 00-7f (RESET = 00) 
        start: u8,
        /// End address 00-7f (RESET = 3F) 
        end: u8,
    },
    /// Setup Row start and end address (0x75)
    RowAddress {
        /// Start address range:00~7f (RESET = 00) 
        start: u8,
        /// End address range:00~7f (RESET = 7F)
        end: u8,
    },
    /// Double byte command to select 1 out of 256 contrast steps. Contrast increases as the value increases (RESET = 7F ) (0x81)
    ContrastControl(u8),
    /// **Re-map setting in Graphic Display Data RAM (GDDRAM)** (0xA0)\
    /// A[0] = 0, Disable Column Address Re-map (RESET)\
    /// A[0] = 1, Enable Column Address Re-map\
    /// A[1] = 0, Disable Nibble Re-map (RESET)\
    /// A[1] = 1, Enable Nibble Re-map\
    /// A[2] = 0, Enable Horizontal Address Increment (RESET)\
    /// A[2] = 1, Enable Vertical Address Increment\
    /// A[3] = 0, Reserved (RESET)\
    /// A[4] = 0, Disable COM Re-map (RESET)\
    /// A[4] = 1, Enable COM Re-map\
    /// A[5] = 0, Reserved (RESET)\
    /// A[6] = 0, Disable COM Split Odd Even (RESET)\
    /// A[6] = 1, Enable COM Split Odd Even\
    /// A[7] = 0, Reserved (RESET) 
    Remap(u8),
    /// Vertical shift by setting the starting address of display RAM from 0 ~ 127 (RESET = 00) (0xA1)
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
    FrontClockDividerOscillatorFrequency(u8),
    /// GPIO : 00 represents GPIO pin HiZ, input disable (always read as low) ; 01 represents GPIO pin HiZ, input enable ; 10 represents GPIO pin output Low (RESET) ; 11 represents GPIO pin output High ; (0xB5)
    GPIO(u8),
    /// Second Pre-charge period of 1~15 DCLK’s (RESET = 0100) (0xB6)
    SecondPreChargePeriod(u8),
    /// The default Lineear Gray Scale table (0xB9)
    LinearLUT,
    /// Set pre-charge voltage level (0xBC)
    PreChargeVoltage(u8),
    /// Set COM deselect voltage level (0xBE)
    VCOMH(u8),
    /// Function Selection B : 0.: Disable second precharge (RESET) ; 1.: Enable second precharge ; .0: Internal VSL (RESET) ; .1: Enable external VSL ; (0xD5)
    FunctionSelectionB(u8),
    /// MCU protection status 0x16 = Lock ; 0x12 Unlock (RESET) ; (0xFD)
    SetCommandLock(u8),
    /// Unlock OLED driver IC MCU interface from entering commands (RESET) (0xFD 0x12)
    CommandUnlock,
    /// Lock OLED driver IC MCU interface from entering commands (0xFD 0x16)
    CommandLock,
}
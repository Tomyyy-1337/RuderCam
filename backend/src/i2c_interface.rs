use std::{mem::MaybeUninit, sync::Mutex};

#[cfg(target_os = "linux")] 
use embedded_hal::i2c::I2c;
#[cfg(target_os = "linux")] 
use linux_embedded_hal::{I2cdev, I2CError};

use crate::{CONFIG, I2C_INTERFACE, shared::Config};

const I2C_BUS_PATH: &str = "/dev/i2c-1";

const MAX17048_I2C_ADDRESS: u8 = 0b0110110;
const ACCELEROMETER_I2C_ADDRESS: u8 = 0b0011000; 
const EEPROM_I2C_ADDRESS_MEMORY: u8 = 0b1010000;

pub struct BatteryStatus {
    pub battery_level: u8,
    pub voltage: f32,
}

pub struct I2CInterface {
    i2c: Mutex<MaybeUninit<I2cDevice>>,
}

impl I2CInterface {
    /// Lock the I2C interface to prevent writes to the EEPROM while performing critical operations like shutdown or reboot.
    pub fn lock(&self) -> std::sync::MutexGuard<'_, MaybeUninit<I2cDevice>> {
        self.i2c.lock().unwrap()
    }

    /// Creates a new I2CInterface with an uninitialized I2C device. 
    /// The I2C has to be initialized by calling `initialize()` before using read or write methods, otherwise they will panic. 
    pub const fn new_uninitialized() -> Self {
        I2CInterface {
            i2c: Mutex::new(MaybeUninit::uninit()),
        }
    }

    /// Initializes the I2C device and configures the sensors. 
    /// This has to be called before using any read or write methods, otherwise they will panic.
    pub fn initialize(&self) {
        let i2c = loop {
            match Self::try_init_i2c() {
                Ok(i2c) => break i2c,
                Err(_) => std::thread::sleep(std::time::Duration::from_secs(1))
            }
        };
        *self.i2c.lock().unwrap() = MaybeUninit::new(i2c);
    }
    
    fn try_init_i2c() -> Result<I2cDevice, I2CError> {
        let mut i2c = I2cDevice::new(I2C_BUS_PATH)?;        
        
        // Init Accelerometer (LIS3DH)
        // CTRL_REG1 (0x20): Enable all axes, 50 Hz data rate
        // 0x4F = 0b01001111: ODR=0100 (50Hz), Z=Y=X=1 (all axes enabled)
        i2c.write(ACCELEROMETER_I2C_ADDRESS, &[0x20, 0x4F])?;
        // CTRL_REG4 (0x23): 4g range, high resolution mode
        // 0x98 = 0b10011000: BDU=1, FS=01 (4g range), HR=1 (high resolution)
        i2c.write(ACCELEROMETER_I2C_ADDRESS, &[0x23, 0x98])?;

        Ok(i2c)
    }

    pub async fn read_battery_status(&self) -> BatteryStatus {
        let bytes = Self::read_async::<2>(
            MAX17048_I2C_ADDRESS,
            0x02,
        ).await;

        let raw_voltage = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
        let voltage = (raw_voltage as f32) * 78.125e-6;

        let battery_level = Self::read_async::<1>(
            MAX17048_I2C_ADDRESS,
            0x04,
        ).await[0];

        BatteryStatus {
            battery_level,
            voltage,
        }
    }

    pub async fn read_accelerometer_data(&self) -> (i16, i16, i16) {
        let bytes = Self::read_async::<6>(ACCELEROMETER_I2C_ADDRESS, 0x28 | 0x80).await;
        let x = i16::from_le_bytes([bytes[0], bytes[1]]) >> 4;
        let y = i16::from_le_bytes([bytes[2], bytes[3]]) >> 4;
        let z = i16::from_le_bytes([bytes[4], bytes[5]]) >> 4;
        (x, y, z)
    }
    
    /// Read N bytes from the specified I2C address and register asynchronously 
    async fn read_async<const N: usize>(
        address: u8,
        register: u8,
    ) -> [u8; N] {
        let buf = tokio::task::spawn_blocking({
            move || {
                let mut buf = [0u8; N];
                let mut device = I2C_INTERFACE.i2c.lock().unwrap();
                // SAFETY: Safe once the device has been initialized by calling `initialize()`.
                let device = unsafe { &mut *device.as_mut_ptr() };
                let _ = device.write_read(address, &[register], &mut buf);
                buf
            }}).await.unwrap_or_else(|_| [0u8; N]);
        buf
    }

    /// Read a block of data from the EEPROM starting at the specified register.
    pub fn read_eeprom_blocking(&self, register: u16, length: usize) -> Vec<u8> {
        let mut read_buffer = vec![0u8; length];
        let mut i2c = self.i2c.lock().unwrap();
        // SAFETY: Safe once the device has been initialized by calling `initialize()`.
        let i2c = unsafe { &mut *i2c.as_mut_ptr() };
        Self::read_eeprom_internal(i2c, register, &mut read_buffer);
        read_buffer
    }

    fn read_eeprom_internal(
        i2c: &mut I2cDevice,
        register: u16,
        read_buffer: &mut [u8], 
    ) {
        let address_bytes = [
            ((register) >> 8) as u8,  
            register as u8,           
        ];
        
        let _ = i2c.write_read(EEPROM_I2C_ADDRESS_MEMORY, &address_bytes, read_buffer);
    }

    pub fn eeprom_initialized_blocking(&self) -> bool {
        let initialized_byte = self.read_eeprom_blocking(0x0000, 1)[0];
        initialized_byte == 41
    }

    pub async fn write_config_to_eeprom(&self) {
        let config = CONFIG.clone();
        tokio::task::spawn_blocking(
            move || I2C_INTERFACE.write_config_to_eeprom_blocking(&config)
        );
    }
    
    /// Write the current configuration Stored in the global CONFIG variable to the EEPROM.
    pub fn write_config_to_eeprom_blocking(&self, config: &Config) {
        let config_data = match serde_json::to_vec(config) {
            Ok(data) => data,
            Err(e) => unreachable!("Failed to serialize config: {}", e),
        };

        let formated_config_data = Self::format_config_data(&config_data);

        let mut max_tries = 10;
        // Try to write the config to EEPROM, retrying on failure
        while self.write_eeprom_blocking(0x0000, &formated_config_data).is_err() {
            eprintln!("Failed to write config to EEPROM. Retrying...");
            max_tries -= 1;
            if max_tries == 0 {
                eprintln!("Failed to write config to EEPROM after multiple attempts.");
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    fn format_config_data(config: &[u8]) -> Vec<u8> {
        let config_len = (config.len() as u16).to_le_bytes();
        let headder = [
            41,                                          // Initialization marker
            config_len[0], config_len[1],                // Config length (little-endian)
        ];
        [headder.as_slice(), config].concat()
    }

    /// Read the configuration from the EEPROM and deserialize it into a Config struct.
    pub fn read_config_from_eeprom_blocking(&self) -> Option<Config> {
        let config_length_bytes = self.read_eeprom_blocking(0x0001, 2);
        let config_length = u16::from_le_bytes([config_length_bytes[0], config_length_bytes[1]]) as usize;
        let config_data = self.read_eeprom_blocking(0x0003, config_length);
        serde_json::from_slice(&config_data).ok()
    }

    fn write_eeprom_blocking(
        &self,
        register: u16,
        data: &[u8],
    ) -> Result<(), I2CError> {
        // M24C64 page write (max 32 bytes per page)
        // Format: [Address_High, Address_Low, Data_Byte_1, Data_Byte_2, ...]
        const PAGE_SIZE: usize = 32;
        
        // Lock the I2C device for the entire write operation to prevent race conditions with other writes
        // As writes are infrequent (only config updates by the user) long blocking of the i2c is acceptable 
        let mut device = self.i2c.lock().unwrap();

        // SAFETY: Safe once the device has been initialized by calling `initialize()`. 
        let device = unsafe { &mut *device.as_mut_ptr() };

        let mut offset = 0;
        while offset < data.len() {
            let addr = register + offset as u16;
            let page_offset = (addr as usize) % PAGE_SIZE;
            let page_remaining = PAGE_SIZE - page_offset;
            let chunk_size = std::cmp::min(page_remaining, data.len() - offset);
            let mut write_buffer = Vec::with_capacity(2 + chunk_size);

            write_buffer.push((addr >> 8) as u8);
            write_buffer.push(addr as u8);
            write_buffer.extend_from_slice(&data[offset..offset + chunk_size]);
            
            device.write(EEPROM_I2C_ADDRESS_MEMORY, &write_buffer)?;

            std::thread::sleep(std::time::Duration::from_millis(10));

            offset += chunk_size;

        }
        Ok(())
    }
}    

#[cfg(not(target_os = "linux"))]
pub struct I2CError;


#[cfg(target_os = "linux")]
type I2cDevice = I2cdev;

#[cfg(not(target_os = "linux"))]
type I2cDevice = MockI2cdev;

// Mock I2C device for non-Linux platforms (Windows development)
#[cfg(not(target_os = "linux"))]
pub struct MockI2cdev;

#[cfg(not(target_os = "linux"))]
impl MockI2cdev {
    pub fn new(_path: &str) -> Result<Self, I2CError> {
        Ok(MockI2cdev)
    }
    
    pub fn write(&mut self, _addr: u8, _data: &[u8]) -> Result<(), I2CError> {
        Ok(())
    }
    
    pub fn write_read(&mut self, _addr: u8, _write_data: &[u8], _read_data: &mut [u8]) -> Result<(), I2CError> {
        Ok(())
    }
}

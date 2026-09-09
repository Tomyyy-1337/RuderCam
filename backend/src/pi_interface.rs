use crate::I2C_INTERFACE;

/// Shutdown the device by setting a GPIO pin high to trigger the shutdown circuit.
pub fn shutdown() {
    
    // Block the I2C interface to ensure no write to the EEPROM happens while we're shutting down
    let _i2c = I2C_INTERFACE.lock(); 
    
    #[cfg(target_os = "linux")]
    {
        use gpiocdev::line::Value;
        use gpiocdev::request::Request;
        let gpio_pin = 17;
        let chip = "/dev/gpiochip0";

        let _request = Request::builder()
            .on_chip(chip)
            .with_line(gpio_pin)
            .as_output(Value::Active)
            .request()
            .expect("Failed to request GPIO line for shutdown");
    }

    std::thread::sleep(std::time::Duration::from_secs(5));

    std::process::exit(0);
}

/// Reboot the device by executing the reboot command. 
pub fn reboot() {
    // Block the I2C interface to ensure no write to the EEPROM happens while we're rebooting
    let _i2c = I2C_INTERFACE.lock(); 

    #[cfg(target_os = "linux")]
    loop {
        match std::process::Command::new("reboot").arg("now").status() {
            Ok(_) => break,
            Err(e) => {
                eprintln!("Failed to execute reboot command: {}. Retrying...", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }

    std::process::exit(0);
}
use mpu6050::*;
use linux_embedded_hal::{I2cdev, Delay};
use std::error::Error;

pub struct GyroSensor {
    mpu: Mpu6050<I2cdev>,
}

impl GyroSensor {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let i2c = I2cdev::new("/dev/i2c-1")?;
        let mut mpu = Mpu6050::new(i2c);
        mpu.init(&mut Delay)?;
        Ok(GyroSensor { mpu })
    }

    pub fn read_angle(&mut self) -> Result<f32, Box<dyn Error>> {
        let gyro = self.mpu.get_gyro()?;
        // Assuming we're only interested in rotation around the z-axis for left/right movement
        Ok(gyro.z)
    }
}
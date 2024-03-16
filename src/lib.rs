use anyhow::Result;
use std::marker::PhantomData;
use tokio::time::Duration;

// Define the different state types
struct Configure;
struct Operate;
struct Standby;

// Define the Radio struct
pub struct Radio<State> {
    data: RadioData,
    state: PhantomData<State>,
}

pub struct RadioData {
    pub number: u32,
    pub other: f64,
}

pub struct RadioError<T> {
    error: anyhow::Error,
    pub radio: Radio<T>,
}

impl<T> std::fmt::Display for RadioError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
impl<T> std::fmt::Debug for RadioError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

// allow RadioError<T> to work with the ? operator because of the error the trait `std::error::Error` is not implemented for `RadioError<Radio<Configure>>`
impl<T> std::error::Error for RadioError<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

// Implement methods for the Radio struct based on the different modes
impl Radio<Standby> {
    pub fn new() -> Self {
        Radio {
            data: RadioData {
                number: 3,
                other: 0.14159,
            },
            state: PhantomData,
        }
    }

    pub async fn configure(self) -> Result<Radio<Configure>, RadioError<Self>> {
        //println!("Radio is in Configure mode");
        // Perform configuration actions here
        tokio::time::sleep(Duration::from_secs(1)).await;
        //println!("Configuration complete");
        Ok(Radio {
            data: self.data,
            state: PhantomData,
        })
    }
}

impl Radio<Configure> {
    pub async fn operate(self) -> Result<Radio<Operate>, RadioError<Self>> {
        //println!("Radio is in Operate mode");
        // Perform operate actions here
        tokio::time::sleep(Duration::from_secs(1)).await;
        //println!("Operate mode enabled");
        Ok(Radio {
            data: self.data,
            state: PhantomData,
        })
    }
    // and back to standby
    pub async fn enter_standby(self) -> Radio<Standby> {
        //println!("Entering Standby mode");
        // Perform standby actions here
        Radio {
            data: self.data,
            state: PhantomData,
        }
    }
}

impl Radio<Operate> {
    pub async fn send_data(&self, _data: &[u8]) {
        //println!("Sending data in operate mode");
        // Perform operate actions here
    }
    pub async fn enter_standby(self) -> Radio<Standby> {
        //println!("Entering Standby mode");
        // Perform standby actions here
        Radio {
            data: self.data,
            state: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_radio() -> anyhow::Result<()> {
        let radio = Radio::<Standby>::new();
        // Must transition to configure first
        let radio = radio.configure().await?;
        // From configure, we can transition to operate
        let radio = radio.operate().await?;
        radio.send_data(&[1, 2, 3]).await;
        let _radio = radio.enter_standby().await;
        Ok(())
    }
}

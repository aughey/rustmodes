use anyhow::Result;
use std::marker::PhantomData;

// Define the different state types
struct Configure;
struct Operate;
struct Standby;
struct Uninitialized;

// The generic radio struct that will self-transition to different states
pub struct Radio<State> {
    /// Data internal to the radio, this is moved from state to state
    data: RadioData,
    state: PhantomData<State>,
}

/// Data relevant to the radio, this might be sockets or other resources
struct RadioData {
    // notional data
    init_count: u32,
    _number: u32,
    _other: f64,
}

/// An error struct that allows an error message to be reported along with the radio that caused it.
/// This is necessary because mode transitions take ownership of the radio and return a new one, so
/// if the transition fails, the original radio must be returned.
pub struct RadioError<T> {
    error: anyhow::Error,
    pub radio: T,
}

// Implement the Display and Debug traits for RadioError<T> so that the error message can be printed
// and proprogated up with ? in the fail condition.
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
impl<T> std::error::Error for RadioError<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl Radio<Uninitialized> {
    pub fn new() -> Self {
        Radio {
            data: RadioData {
                init_count: 0,
                _number: 3,
                _other: 0.14159,
            },
            state: PhantomData,
        }
    }
    pub fn new_init(count: u32) -> Self {
        Radio {
            data: RadioData {
                init_count: count,
                _number: 3,
                _other: 0.14159,
            },
            state: PhantomData,
        }
    }

    /// Attempt to establish contact with the radio and entire standby if
    /// successful.
    pub async fn standby(mut self) -> Result<Radio<Standby>, RadioError<Self>> {
        if self.data.init_count > 0 {
            self.data.init_count -= 1;
            return Err(RadioError {
                error: anyhow::anyhow!("Radio not ready to configure"),
                radio: self,
            });
        }
        //println!("Radio is in Configure mode");
        // Perform configuration actions here
        //tokio::time::sleep(Duration::from_secs(1)).await;
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
        //tokio::time::sleep(Duration::from_secs(1)).await;
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

impl Radio<Standby> {
    pub async fn configure(self) -> Result<Radio<Configure>, RadioError<Self>> {
        //println!("Radio is in Configure mode");
        // Perform configuration actions here
        //tokio::time::sleep(Duration::from_secs(1)).await;
        //println!("Configuration complete");
        Ok(Radio {
            data: self.data,
            state: PhantomData,
        })
    }
}

impl Radio<Operate> {
    pub async fn send_data(&self, _data: &[u8]) -> Result<()> {
        //println!("Sending data in operate mode");
        // Perform operate actions here
        Ok(())
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
        let radio = Radio::<Uninitialized>::new();
        // Must transition to standby first
        let radio = radio.standby().await?;
        // Must transition to configure
        let radio = radio.configure().await?;
        // From configure, we can transition to operate
        let radio = radio.operate().await?;
        radio.send_data(&[1, 2, 3]).await?;
        let _radio = radio.enter_standby().await;
        Ok(())
    }

    #[tokio::test]
    async fn show_looping_init() -> anyhow::Result<()> {
        let radio = Radio::<Uninitialized>::new_init(2);

        // Loop as many times as needed to get into standby mode (it might not be ready)
        let radio = {
            // keep track of how many times we try to init
            let mut init_count = 0;
            // the radio variable needs to be mut to be updated in the loop
            let mut radio = radio;
            loop {
                // Try to go into standby
                match radio.standby().await {
                    Ok(radio) => {
                        // Successful, break out of the loop with the radio in standby
                        assert_eq!(init_count, 2);
                        break radio;
                    }
                    Err(e) => {
                        // Bad day, try again
                        println!("Error configuring radio: {}", e);
                        //tokio::time::sleep(Duration::from_secs(1)).await;
                        // The prior radio is in the error struct, so pull it out and try again
                        radio = e.radio;
                        init_count += 1;
                    }
                }
            }
        };
        // in standby, go to configure
        let radio = radio.configure().await?;
        let radio = radio.operate().await?;
        radio.send_data(&[1, 2, 3]).await?;
        let _radio = radio.enter_standby().await;
        Ok(())
    }
}

use anyhow::Result;
use std::marker::PhantomData;
pub mod future_helper;

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
impl RadioData {
    pub fn new(count: u32) -> Self {
        RadioData {
            init_count: count,
            _number: 3,
            _other: 0.14159,
        }
    }
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
            data: RadioData::new(0),
            state: PhantomData,
        }
    }
    pub fn new_init(count: u32) -> Self {
        Radio {
            data: RadioData::new(count),
            state: PhantomData,
        }
    }

    /// Attempt to establish contact with the radio and entire standby if
    /// successful.
    pub async fn standby(mut self) -> Result<Radio<Standby>, RadioError<Self>> {
        // We use this init_count to simulate a radio that might not be ready
        if self.data.init_count > 0 {
            self.data.init_count -= 1;
            return Err(RadioError {
                error: anyhow::anyhow!("Radio not ready to configure"),
                radio: self,
            });
        }

        // Perform configuration actions here

        Ok(Radio {
            data: self.data,
            state: PhantomData,
        })
    }
}

impl Radio<Configure> {
    pub async fn operate(self) -> Result<Radio<Operate>, RadioError<Self>> {
        // Perform operate actions here
        //tokio::time::sleep(Duration::from_secs(1)).await;

        Ok(Radio {
            data: self.data,
            state: PhantomData,
        })
    }
    // and back to standby
    pub async fn enter_standby(self) -> Radio<Standby> {
        // Perform standby actions here

        Radio {
            data: self.data,
            state: PhantomData,
        }
    }
}

#[derive(Default)]
struct ConfigureData;

impl Radio<Standby> {
    pub async fn configure(
        self,
        _configdata: ConfigureData,
    ) -> Result<Radio<Configure>, RadioError<Self>> {
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
    use std::{future::Future, time::Duration};

    use tokio::task::yield_now;

    use self::future_helper::*;
    use super::*;

    /// Our radio, using the type system and ownership transfer, will transition from
    ///
    /// Uninitialized -> Standby -> Configure -> Operate
    ///
    /// It can also transition to Standby from both Configure and Operate.
    ///
    /// All transitions can fail and return the radio to the prior state.
    /// (except going to standby, that always works)
    #[tokio::test]
    async fn test_radio() -> anyhow::Result<()> {
        // Start in uninitialized.  This is the only way we can create a radio.
        let radio = Radio::<Uninitialized>::new();
        // Must transition to standby first, this sets up the initial communication with the radio.
        let radio = radio.standby().await?;
        // Transition to configure given the supplied configuration data.
        let radio = radio.configure(ConfigureData::default()).await?;
        // From configure, we can transition to operate (or back to standby)
        let radio = radio.operate().await?;
        // Try sending data in operate mode
        radio.send_data(&[1, 2, 3]).await?;
        // Done, go back to standby
        let _radio = radio.enter_standby().await;
        Ok(())
    }

    // Show how we might loop continuously to get into standby.
    #[tokio::test]
    async fn test_looping_init() -> anyhow::Result<()> {
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
        let radio = radio.configure(ConfigureData::default()).await?;
        let radio = radio.operate().await?;
        radio.send_data(&[1, 2, 3]).await?;
        let _radio = radio.enter_standby().await;
        Ok(())
    }

    /// Try to enter standby mode continuously until successful.
    /// We write this as a function, we don't pollute the Radio implementation.
    async fn try_enter_standby_forever(mut radio: Radio<Uninitialized>) -> Radio<Standby> {
        // Loop as many times as needed to get into standby mode (it might not be ready)
        loop {
            // Try to go into standby
            match radio.standby().await {
                Ok(radio) => break radio,
                Err(e) => {
                    // Bad day, try again
                    // yield because we don't ever actually await for anything.
                    // Necessary because we need to allow other tasks to run.
                    yield_now().await;
                    // The prior radio is in the error struct, so pull it out and try again
                    radio = e.radio;
                }
            }
        }
    }

    #[tokio::test]
    async fn test_looping_standby() -> Result<()> {
        let radio = Radio::<Uninitialized>::new_init(2);
        let _radio = try_enter_standby_forever(radio).await;
        Ok(())
    }

    /// Try to enter standby mode continuously unless a timeout future completes first.
    async fn try_enter_standby_until<E>(
        radio: Radio<Uninitialized>,
        timeout: impl Future<Output = E>,
    ) -> Result<Radio<Standby>, E> {
        match wait_for_one_to_complete(try_enter_standby_forever(radio), timeout).await {
            FirstOrSecond::First(r) => Ok(r),
            FirstOrSecond::Second(e) => Err(e),
        }
    }

    #[tokio::test]
    async fn test_timeout_standby() -> Result<()> {
        let radio = Radio::<Uninitialized>::new_init(2);
        // Create our timeout future, we try for 5 seconds then give up
        let timeout = async {
            tokio::time::sleep(Duration::from_secs(5)).await;
            anyhow::anyhow!("Timeout waiting for standby")
        };
        let _radio = try_enter_standby_until(radio, timeout).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_timeout_from_other_event() -> Result<()> {
        // Create a oneshot channel that represents some sort of cancellation button having been pressed.
        let (_tx, rx) = tokio::sync::oneshot::channel::<()>();
        let cancel = async {
            // The button press message indiciates that a timeout was requested
            _ = rx.await;
            anyhow::anyhow!("Cancel requested from the user")
        };

        let radio = Radio::<Uninitialized>::new_init(2);
        let _radio = try_enter_standby_until(radio, cancel).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_timeout_from_other_event_pressed() -> Result<()> {
        // Create a oneshot channel that represents some sort of cancellation button having been pressed.
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let cancel = async {
            // The button press message indiciates that a timeout was requested
            _ = rx.await;
            anyhow::anyhow!("Cancel requested from the user")
        };
        // "Press" the button
        tx.send(()).unwrap();

        let radio = Radio::<Uninitialized>::new_init(2);
        let expect_error_not_radio = try_enter_standby_until(radio, cancel).await;

        assert!(expect_error_not_radio.is_err());
        // message should be "Cancel requested from the user"
        assert_eq!(
            expect_error_not_radio.err().unwrap().to_string(),
            "Cancel requested from the user"
        );
        Ok(())
    }
}

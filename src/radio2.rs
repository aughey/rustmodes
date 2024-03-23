use anyhow::Result;
use std::marker::PhantomData;

struct Uninitialized;
struct Configured;
struct Operate;

#[derive(Default, Debug)]
pub struct Data {
    _value: u32,
}

#[derive(Debug)]
pub struct Radio<State> {
    state: PhantomData<State>,
    data: Data,
}

impl Default for Radio<Uninitialized> {
    fn default() -> Self {
        Radio {
            state: PhantomData,
            data: Data::default(),
        }
    }
}

impl Radio<Uninitialized> {
    pub fn get_data(&self) -> &Data {
        &self.data
    }
    pub async fn configure(self) -> Result<Radio<Configured>> {
        Ok(Radio {
            state: PhantomData,
            data: self.data,
        })
    }
}

impl Radio<Configured> {
    pub async fn operate(self) -> Result<Radio<Operate>> {
        // Here is where you mess with registers or comm to kick the radio into operate mode.
        Ok(Radio {
            state: PhantomData,
            data: self.data,
        })
    }

    pub async fn standby(self) -> Result<Radio<Uninitialized>> {
        // Here is where you mess with registers or comm to kick the radio into standby mode.
        Ok(Radio {
            state: PhantomData,
            data: self.data,
        })
    }
}

impl Radio<Operate> {
    pub async fn standby(self) -> Result<Radio<Uninitialized>> {
        // Here is where you mess with registers or comm to kick the radio into standby mode.
        Ok(Radio {
            state: PhantomData,
            data: self.data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_radio() {
        let mut _radio = Radio::<Uninitialized>::default();
    }

    #[tokio::test]
    async fn test_configure() -> Result<()> {
        let _configured_radio = loop {
            let radio = Radio::<Uninitialized>::default();

            match radio.configure().await {
                Ok(radio) => break radio,
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        };

        Ok(())
    }

    #[tokio::test]
    async fn test_operate() -> Result<()> {
        let radio = Radio::<Uninitialized>::default();
        let radio = radio.configure().await?;
        let radio = radio.standby().await?;

        let radio = radio.configure().await?;
        let radio = radio.operate().await?;
        let _radio = radio.standby().await?;

        Ok(())
    }
}

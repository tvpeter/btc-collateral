use config::{Config, ConfigError, File};

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub application_port: u16
}


impl Settings {
    pub fn get_configuration() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(File::with_name("config"))
            .build()?;

        settings.try_deserialize()
    }
}


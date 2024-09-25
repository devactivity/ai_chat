use std::error::Error as StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Application {
    #[error("Configuration error")]
    Config(#[from] crate::config::ConfigError),

    #[error("UI error")]
    Ui(#[from] color_eyre::Report),

    #[error("Network error")]
    Network(#[from] reqwest::Error),

    #[error("JSON parsing error")]
    JsonParse(#[from] serde_json::Error),

    #[error("Logging error: {0}")]
    Logging(String),

    #[error("Unexpected error")]
    Unexpected(String),
}

impl From<Box<dyn StdError>> for Application {
    fn from(error: Box<dyn StdError>) -> Self {
        Application::Logging(error.to_string())
    }
}

impl Application {
    pub fn user_friendly_message(&self) -> String {
        match self {
            Application::Config(_) => {
                "There was an issue with the application configuration".to_string()
            }
            Application::Ui(_) => "An error occured in the user interface".to_string(),
            Application::Network(_) => "There was a problem connecting to the server".to_string(),
            Application::JsonParse(_) => {
                "There was an issue processing the server response".to_string()
            }
            Application::Logging(_) => {
                "There was a problem setting up the application logs".to_string()
            }
            Application::Unexpected(_) => "An unexpected error occured".to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, Application>;

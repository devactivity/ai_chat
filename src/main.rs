mod config;
mod error;
mod logging;
mod ui;

use config::Config;
use error::{AppResult, Application};
use log::error;
use serde_json::json;
use std::time::Duration;
use tokio::sync::mpsc;
use ui::ChatUI;

#[tokio::main]
async fn main() -> AppResult<()> {
    color_eyre::install()?;
    logging::setup_logging()?;

    let config = Config::load()?;
    let mut chat_ui = ChatUI::new()?;
    let client = reqwest::Client::new();

    while let Some(message) = chat_ui.run()? {
        let (tx, mut rx) = mpsc::channel(1);
        let client_clone = client.clone();
        let config_clone = config.clone();

        tokio::spawn(async move {
            let result = async {
                let response = client_clone
                    .post(&config_clone.api_endpoint)
                    .json(&json!({
                        "model": config_clone.model,
                        "messages": [{
                            "role": "user",
                            "content": message
                        }],
                        "stream": false
                        // "temperature": config_clone.temperature
                    }))
                    .send()
                    .await?;

                let json_response: serde_json::Value = response.json().await?;

                json_response["message"]["content"]
                    .as_str()
                    .map(|content| content.to_string())
                    .ok_or_else(|| {
                        Application::Unexpected("failed to parse response content".to_string())
                    })
            }
            .await;

            tx.send(result).await.unwrap();
        });

        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Some(Ok(content)) => {
                            chat_ui.add_response(content);
                            break;
                        }
                        Some(Err(err)) => {
                            error!("error occurred: {:?}", err);
                            chat_ui.add_response(format!("error: {}",
                            err.user_friendly_message()));
                            chat_ui.add_response(format!("for more details, please check the log file at: {}
            ", logging::get_log_file_path().display()));
                            break;
                        }
                        None => {
                            error!("unexpected error: channel closed unexpectedly");
                            chat_ui.add_response("unexpected error occured. Please check the log file for more details".to_string());
                            break;
                        }
                    }
                }
                    () = tokio::time::sleep(Duration::from_millis(100)) => {
                        if let Some(action) = chat_ui.update()? {
                            if action == ui::Action::CancelRequest {
                                chat_ui.add_response("request cancelled".to_string());
                                break;
                            }
                        }
                    }
            }
        }
    }

    Ok(())
}

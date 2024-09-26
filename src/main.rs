mod config;
mod error;
mod logging;
mod ui;

use config::Config;
use error::{AppResult, Application};
use futures_util::StreamExt;
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
        let (tx, mut rx) = mpsc::channel(100);
        let client_clone = client.clone();
        let config_clone = config.clone();

        tokio::spawn(async move {
            let result: Result<(), Application> = async {
                let response = client_clone
                    .post(&config_clone.api_endpoint)
                    .json(&json!({
                        "model": config_clone.model,
                        "messages": [{
                            "role": "user",
                            "content": message
                        }],
                        "stream": config_clone.stream,
                        "temperature": config_clone.temperature
                    }))
                    .send()
                    .await?;

                let mut stream = response.bytes_stream();

                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    let chunk_str = String::from_utf8_lossy(&chunk);

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&chunk_str) {
                        if let Some(content) = json["message"]["content"].as_str() {
                            tx.send(Ok(content.to_string())).await.unwrap();
                        }

                        if json["done"].as_bool().unwrap_or(false) {
                            break;
                        }
                    }
                }

                Ok(())
            }
            .await;

            if let Err(err) = result {
                tx.send(Err(err)).await.unwrap();
            }
        });

        chat_ui.start_new_response();
        let mut full_response = String::new();

        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Some(Ok(content)) => {
                            full_response.push_str(&content);
                            chat_ui.update_response(&content);
                        }
                        Some(Err(err)) => {
                            error!("error occurred: {:?}", err);
                            chat_ui.add_response(
                                format!("error: {}",
                                err.user_friendly_message())
                            );
                            chat_ui.add_response(
                                format!(
                                    "for more details, please check the log file at: {}",
                                    logging::get_log_file_path().display())
                            );
                            break;
                        }
                        None => {
                            if !full_response.is_empty() {
                                chat_ui.add_response(full_response);
                            } else {
                                error!("unexpected error: channel closed unexpectedly");
                                chat_ui.add_response("unexpected error occured. Please check the log file for more details".to_string());
                            }

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

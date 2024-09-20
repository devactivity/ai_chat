mod ui;

use color_eyre::Result;
use serde_json::json;
use std::time::Duration;
use tokio::sync::mpsc;
use ui::ChatUI;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut chat_ui = ChatUI::new()?;
    let client = reqwest::Client::new();

    while let Some(message) = chat_ui.run()? {
        let (tx, mut rx) = mpsc::channel(1);
        let client_clone = client.clone();

        tokio::spawn(async move {
            let response = client_clone
                .post("http://localhost:8080/v1/chat/completions")
                .json(&json!({
                    "model": "gpt-4",
                    "messages": [{
                        "role": "user",
                        "content": message
                    }],
                    "temperature": 0.7
                }))
                .send()
                .await;

            let result = match response {
                Ok(response) => match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        if let Some(content) =
                            json_response["choices"][0]["message"]["content"].as_str()
                        {
                            Ok(content.to_string())
                        } else {
                            Err("failed to parse response.".to_string())
                        }
                    }
                    Err(_) => Err("failed to parse JSON response".to_string()),
                },
                Err(_) => Err("failed to send request".to_string()),
            };

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
                            chat_ui.add_response(err);
                            break;
                        }
                        None => {
                            chat_ui.add_response("unexpected error occured".to_string());
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
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

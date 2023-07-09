use std::time::Duration;

use color_eyre::{eyre::Context, Result};
use reqwest::Client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
struct CheckWebsiteStatusRequest {
    url: String,
    tx: oneshot::Sender<WebsiteStatus>,
}

#[derive(Debug)]
pub enum WebsiteStatus {
    Valid,
    Redirected(String),
    Dead,
}

#[derive(Debug, Clone)]
pub struct WebsiteChecker {
    tx: mpsc::Sender<CheckWebsiteStatusRequest>,
}

impl WebsiteChecker {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<CheckWebsiteStatusRequest>(100);

        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .wrap_err("Failed to build request client")
                .unwrap();

            while let Some(req) = rx.recv().await {
                let response = Self::check_website(&client, &req.url).await;
                let _ = req.tx.send(response.unwrap_or(WebsiteStatus::Dead));
            }
        });

        Self { tx }
    }

    pub async fn request_check(&self, url: String) -> Result<WebsiteStatus> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(CheckWebsiteStatusRequest { url, tx })
            .await
            .wrap_err("Failed to send check request")?;
        rx.await.wrap_err("Failed to receive check status")
    }

    async fn check_website(client: &Client, url: &str) -> Result<WebsiteStatus> {
        let response = client
            .get(url)
            .send()
            .await
            .wrap_err("Failed to send request")?;

        match response.status() {
            s if s.is_success() => Ok(WebsiteStatus::Valid),
            s if s.is_redirection() => {
                let location = response
                    .headers()
                    .get("Location")
                    .and_then(|l| l.to_str().ok())
                    .unwrap_or_default()
                    .to_string();
                Ok(WebsiteStatus::Redirected(location))
            }
            _ => Ok(WebsiteStatus::Dead),
        }
    }
}

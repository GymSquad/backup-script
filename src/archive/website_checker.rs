use std::time::Duration;

use color_eyre::{eyre::Context, Report, Result};
use reqwest::Client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
struct CheckWebsiteStatusRequest {
    url: String,
    tx: oneshot::Sender<WebsiteStatus>,
}

#[derive(Debug)]
pub enum WebsiteStatus {
    Valid(reqwest::Url),
    Redirected(reqwest::Url),
    Dead(reqwest::Url),
    Failed(Report),
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
                tracing::info!("Checking validity of {}...", req.url);
                let response = Self::check_website(&client, &req.url).await;
                let _ = req.tx.send(response);
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

    async fn check_website(client: &Client, url: &str) -> WebsiteStatus {
        let response = match client
            .get(url)
            .send()
            .await
            .wrap_err("Failed to send request")
        {
            Ok(response) => response,
            Err(e) => return WebsiteStatus::Failed(e),
        };

        match response.status() {
            s if s.is_success() => WebsiteStatus::Valid(response.url().clone()),
            s if s.is_redirection() => WebsiteStatus::Redirected(response.url().clone()),
            _ => WebsiteStatus::Dead(response.url().clone()),
        }
    }
}

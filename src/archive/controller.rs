use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{fs, process::Command, task::JoinHandle};

use crate::db::{Database, Website};

use super::website_checker::{WebsiteChecker, WebsiteStatus};

pub struct ArchiveController {
    db: Database,
    checker: WebsiteChecker,
    program: Arc<str>,
    args: Arc<[&'static str]>,
    output_path: Arc<Path>,
    handles: Vec<JoinHandle<()>>,
}

impl ArchiveController {
    pub fn new(
        db: Database,
        checker: WebsiteChecker,
        program: Arc<str>,
        args: Arc<[&'static str]>,
        output_path: Arc<Path>,
    ) -> Self {
        Self {
            db,
            checker,
            program,
            args,
            output_path,
            handles: Vec::new(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn archive(&mut self, website: Website) {
        let db = self.db.clone();
        let checker = self.checker.clone();
        let program = self.program.clone();
        let args = self.args.clone();
        let output_path = self.output_path.clone();
        let today = chrono::Local::now().format("%Y-%m-%d");

        let handle = tokio::spawn(async move {
            let (url, is_valid) = match checker.request_check(website.url.clone()).await {
                Ok(WebsiteStatus::Valid(url)) => (url, true),
                Ok(WebsiteStatus::Redirected(url)) => (url, true),
                Ok(WebsiteStatus::Dead(url)) => (url, false),
                Ok(WebsiteStatus::Failed(e)) => {
                    tracing::warn!(
                        "Failed to send request to {}. (Cause: {}) Skipping...",
                        &website.url,
                        e
                    );
                    return;
                }
                Err(e) => {
                    tracing::warn!("Something went wrong: {}", e);
                    return;
                }
            };

            if website.is_stale(is_valid) {
                db.update_website_status(&website.id, is_valid)
                    .await
                    .unwrap();
            }

            if !is_valid {
                tracing::info!("Invalid website: {}, skipping...", &url);
                return;
            }

            let Some(archived_path) = get_archive_path(&url) else {
                tracing::warn!("Cannot get archive path, skipping...");
                return;
            };
            let mut output_path = output_path.join(&website.id);
            output_path.push(today.to_string());

            tracing::info!("Start to archive {}...", &url);
            let mut child = Command::new(&*program)
                .args(args.iter().map(|s| {
                    if s.eq_ignore_ascii_case("{url}") {
                        url.as_str()
                    } else {
                        *s
                    }
                }))
                .spawn()
                .unwrap();

            match child.wait().await {
                Ok(s) => {
                    tracing::info!("Archive complete!");
                    if !s.success() {
                        tracing::warn!("Archive exited with non-zero exit code");
                    }
                    if let Err(e) = fs::create_dir_all(&output_path).await {
                        tracing::error!("Failed to create output directory: {}", e);
                        return;
                    }
                    if let Err(e) = fs::rename(archived_path, &output_path).await {
                        tracing::error!("Failed to move archive: {}", e);
                        return;
                    }
                    tracing::info!("Archive moved to {}", output_path.display());
                }
                Err(e) => {
                    tracing::error!("Archive failed with error: {}", e);
                }
            }
        });

        self.handles.push(handle);
    }

    pub async fn wait(self) {
        for handle in self.handles {
            handle.await.unwrap();
        }
    }
}

fn get_archive_path(url: &reqwest::Url) -> Option<PathBuf> {
    let mut path = PathBuf::from(url.host_str()?);
    if let Some(path_segments) = url.path_segments() {
        for segment in path_segments {
            path.push(segment);
        }
    }
    Some(path)
}

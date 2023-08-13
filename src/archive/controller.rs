use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use tokio::{fs, process::Command, sync::Semaphore, task::JoinHandle};

use crate::db::{Database, Website};

use super::website_checker::{WebsiteChecker, WebsiteStatus};

static TODAY_STR: OnceLock<String> = OnceLock::new();

pub struct ArchiveController {
    /// Database connection
    db: Database,
    /// Website checker to check if a website is still valid
    checker: WebsiteChecker,
    /// Program name to use for archiving
    program: Arc<str>,
    /// Arguments to pass to the program
    args: Arc<[&'static str]>,
    /// Output path to store the archived website
    output_path: Arc<Path>,
    /// Semaphore to limit the number of concurrent workers
    semaphore: Arc<Semaphore>,
    /// Join handles for all the workers
    handles: Vec<JoinHandle<()>>,
}

impl ArchiveController {
    pub fn new(
        db: Database,
        checker: WebsiteChecker,
        program: Arc<str>,
        args: Arc<[&'static str]>,
        output_path: Arc<Path>,
        num_workers: usize,
    ) -> Self {
        TODAY_STR.get_or_init(|| chrono::Local::now().format("%Y-%m-%d").to_string());
        let semaphore = Arc::new(Semaphore::new(num_workers));

        Self {
            db,
            checker,
            program,
            args,
            output_path,
            semaphore,
            handles: Vec::new(),
        }
    }

    pub async fn archive(&mut self, website: Website) {
        let config = ArchiveWebsiteConfig {
            db: self.db.clone(),
            checker: self.checker.clone(),
            program: self.program.clone(),
            args: self.args.clone(),
            output_path: self.output_path.clone(),
        };
        let semaphore = self.semaphore.clone();
        let handle = tokio::spawn(Self::archive_website(website, config, semaphore));

        self.handles.push(handle);
    }

    #[tracing::instrument(name="archive", skip(website, config, semaphore), fields(url=%website.url))]
    async fn archive_website(
        website: Website,
        config: ArchiveWebsiteConfig,
        semaphore: Arc<Semaphore>,
    ) {
        let ArchiveWebsiteConfig {
            db,
            checker,
            program,
            args,
            output_path,
        } = config;
        let today = TODAY_STR.get().unwrap();

        let (url, is_valid) = match checker.request_check(website.url.clone()).await {
            Ok(WebsiteStatus::Valid(url)) | Ok(WebsiteStatus::Redirected(url)) => (url, true),
            Ok(WebsiteStatus::Dead(url)) => (url, false),
            Ok(WebsiteStatus::Failed(e)) => {
                tracing::warn!("Failed to send request: {}", e);
                tracing::warn!("Skipping...");
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
            tracing::info!("Skipping invalid website...");
            return;
        }

        let Some(archived_path) = url.host_str() else {
            tracing::warn!("Cannot get archive path, skipping...");
            return;
        };
        let archived_path = PathBuf::from(archived_path);
        let mut output_path = output_path.join(&website.id);
        output_path.push(today.to_string());

        let _permit = semaphore.acquire_owned().await.unwrap();

        tracing::info!("Start archiving...");
        let mut child = Command::new(&*program)
            .args(args.iter().map(|s| {
                if s.eq_ignore_ascii_case("{url}") {
                    url.as_str()
                } else {
                    *s
                }
            }))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
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
    }

    pub async fn wait(self) {
        for handle in self.handles {
            handle.await.unwrap();
        }
    }
}

struct ArchiveWebsiteConfig {
    db: Database,
    checker: WebsiteChecker,
    program: Arc<str>,
    args: Arc<[&'static str]>,
    output_path: Arc<Path>,
}

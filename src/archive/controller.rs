use std::sync::Arc;

use tokio::{process::Command, task::JoinHandle};

use crate::db::{Database, Website};

use super::website_checker::{WebsiteChecker, WebsiteStatus};

pub struct ArchiveController {
    db: Database,
    checker: WebsiteChecker,
    program: Arc<str>,
    args: Arc<[&'static str]>,
    handles: Vec<JoinHandle<()>>,
}

impl ArchiveController {
    pub fn new(
        db: Database,
        checker: WebsiteChecker,
        program: Arc<str>,
        args: Arc<[&'static str]>,
    ) -> Self {
        Self {
            db,
            checker,
            program,
            args,
            handles: Vec::new(),
        }
    }

    pub fn add_website(&mut self, website: Website) {
        let db = self.db.clone();
        let checker = self.checker.clone();
        let program = self.program.clone();
        let args = self.args.clone();

        let handle = tokio::spawn(async move {
            let (url, is_valid) = match checker.request_check(website.url.clone()).await {
                Ok(WebsiteStatus::Valid) => (website.url, true),
                Ok(WebsiteStatus::Redirected(url)) => (url, true),
                Ok(WebsiteStatus::Dead) => (website.url, false),
                Err(_) => return,
            };

            if website.is_valid != is_valid {
                db.update_website_status(&website.id, is_valid)
                    .await
                    .unwrap();
            }

            if !is_valid {
                return;
            }

            let mut child = Command::new(&*program)
                .args(args.into_iter().map(|s| {
                    if s.eq_ignore_ascii_case("{url}") {
                        url.as_str()
                    } else {
                        *s
                    }
                }))
                .spawn()
                .unwrap();

            match child.wait().await {
                Ok(s) if s.success() => {
                    todo!("move to correct location");
                }
                Ok(_) => {
                    todo!("warn about non-successful exit codes");
                }
                Err(_) => {
                    todo!("warn about errors");
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

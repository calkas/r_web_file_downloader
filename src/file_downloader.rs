use futures::future::join_all;
use futures::Future;
use reqwest::Url;
use std::error::Error;
use std::fmt;
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;
#[derive(Debug)]
struct DownloadError;

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error with download files.")
    }
}

impl Error for DownloadError {}

#[derive(PartialEq)]
enum DownloadStatus {
    InProgress,
    Completed,
}

struct JobStatus {
    download_status: DownloadStatus,
}
impl Default for JobStatus {
    fn default() -> Self {
        Self {
            download_status: DownloadStatus::InProgress,
        }
    }
}

struct DownloadProgressIndicator {
    status_of_tasks: Vec<Arc<Mutex<JobStatus>>>,
    information_log: String,
}

impl Future for DownloadProgressIndicator {
    type Output = bool;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut is_lock_fail = false;
        let number_of_jobs = self.status_of_tasks.len();
        let mut jobs_completed: usize = 0;

        for job_status in self.status_of_tasks.iter() {
            match job_status.try_lock() {
                Ok(guard) => {
                    if guard.download_status == DownloadStatus::Completed {
                        jobs_completed += 1;
                    }
                }
                Err(_) => {
                    is_lock_fail = true;
                    continue;
                }
            };
        }

        let percent_of_done = (jobs_completed as f32 / number_of_jobs as f32) * 100.0;

        let new_information_log = format!("Download {}%", percent_of_done);
        if new_information_log != self.information_log {
            println!("{}", new_information_log);
            self.information_log = new_information_log;
        }

        if is_lock_fail || percent_of_done < 100.0 {
            cx.waker().wake_by_ref();
            return std::task::Poll::Pending;
        }
        std::task::Poll::Ready(true)
    }
}

pub async fn download_listed_files(
    links: &[String],
    download_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut download_tasks = vec![];
    let mut download_status_of_tasks = vec![];

    for link in links.iter() {
        download_status_of_tasks.push(Arc::new(Mutex::new(JobStatus::default())));

        download_tasks.push(download_file(
            link,
            download_path,
            download_status_of_tasks.last().unwrap().clone(),
        ));
    }

    let download_indicator = DownloadProgressIndicator {
        status_of_tasks: download_status_of_tasks,
        information_log: String::new(),
    };
    let download_indicator_handler = tokio::spawn(download_indicator);

    let res_output: Vec<Result<(), Box<dyn std::error::Error>>> = join_all(download_tasks).await;

    for res in res_output.iter() {
        if res.is_err() {
            let error: Box<dyn Error> = Box::new(DownloadError);
            download_indicator_handler.abort();
            println!("\x1b[91mError downloading files.\x1b[0m");
            return Err(error);
        }
    }
    download_indicator_handler.await.unwrap();
    println!("\n\x1b[92mDownload completed!\x1b[0m");

    Ok(())
}

async fn download_file(
    link: &String,
    download_path: &str,
    download_status: Arc<Mutex<JobStatus>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url_str = Url::parse(link)?;

    if let Some(segments) = url_str.path_segments() {
        if let Some(file_name) = segments.last() {
            let file_path = format!("{}/{}", download_path, file_name);
            let response = reqwest::get(link).await?;
            let mut dest = std::fs::File::create(file_path)?;
            let mut content = Cursor::new(response.bytes().await?);
            std::io::copy(&mut content, &mut dest)?;
        }
    }
    download_status.lock().unwrap().download_status = DownloadStatus::Completed;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn download_listed_files_list_of_urls_returns_downloaded_files() {
        let links = vec![
            "https://example.com/file1.txt".to_string(),
            "https://example.com/file2.txt".to_string(),
            "https://example.com/file3.txt".to_string(),
        ];

        let download_path = "./test_files";
        std::fs::create_dir(download_path).unwrap();

        let exp_downloaded_files = vec![
            format!("{}/file1.txt", download_path),
            format!("{}/file2.txt", download_path),
            format!("{}/file3.txt", download_path),
        ];

        let result = download_listed_files(&links, download_path).await;

        assert!(result.is_ok());
        for file in exp_downloaded_files.iter() {
            assert!(std::fs::metadata(file).is_ok());
        }
        std::fs::remove_dir_all(download_path).unwrap();
    }

    #[tokio::test]
    async fn download_listed_files_invalid_url_returns_error() {
        let invalid_links = vec![
            "invalid_url".to_string(),
            "invalid_url".to_string(),
            "invalid_url".to_string(),
        ];

        let download_path = "./test_files";

        let result = download_listed_files(&invalid_links, download_path).await;

        assert!(result.is_err());
    }
}

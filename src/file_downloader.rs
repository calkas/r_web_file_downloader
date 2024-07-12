use futures::future::join_all;
use reqwest::Url;
use std::error::Error;
use std::fmt::{self, write};
use std::io::Cursor;

#[derive(Debug)]
struct DownloadError;

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error with download files.")
    }
}

impl Error for DownloadError {}

pub async fn download_listed_files(
    links: &Vec<String>,
    download_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut download_tasks = vec![];

    for link in links.iter() {
        download_tasks.push(download_file(link, download_path));
    }

    let res_output: Vec<Result<(), Box<dyn std::error::Error>>> = join_all(download_tasks).await;

    for res in res_output.iter() {
        if res.is_err() {
            let error: Box<dyn Error> = Box::new(DownloadError);
            return Err(error);
        }
    }

    Ok(())
}

async fn download_file(
    link: &String,
    download_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url_str = Url::parse(link)?;
    println!("Downloading file: {}", link);
    if let Some(segments) = url_str.path_segments() {
        if let Some(file_name) = segments.last() {
            let file_path = format!("{}/{}", download_path, file_name);
            let response = reqwest::get(link).await?;
            let mut dest = std::fs::File::create(file_path)?;
            let mut content = Cursor::new(response.bytes().await?);
            std::io::copy(&mut content, &mut dest)?;
        }
    }

    println!("Downloaded file: {}", link);
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

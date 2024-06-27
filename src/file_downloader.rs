use reqwest::Url;
use std::io::Cursor;

pub async fn download_listed_files(
    links: &Vec<String>,
    download_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for link in links.iter() {
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
    }

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

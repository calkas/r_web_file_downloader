use reqwest::Url;
use scraper::{Html, Selector};

fn get_fake_links_for_test(url: &str) -> String {
    let body = r#"
    <!DOCTYPE html>
    <meta charset="utf-8">
    <title>Hello, world!</title>
    <h1 class="foo">Hello, <i>world!</i></h1>
    <a href="https://www.test.com/file_pdf_1.pdf">Test File1 PDF</a>
    <a href="https://www.test.com/file_pdf_2.pdf">Test File2 PDF</a>
    <a href="https://www.test.com/file_txt_1.txt">Test File1 TXT</a>
    <a href="/secret/lab/file_pdf_secret.pdf">Secret PDF</a>"#;

    String::from(body)
}

pub async fn get_all_files_links(
    url: &str,
    extension: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let url_str = Url::parse(url)?;

    #[cfg(test)]
    let body = get_fake_links_for_test(url);

    #[cfg(not(test))]
    let body = reqwest::get(url).await?.text().await?;

    let document = Html::parse_document(&body);
    let selector = Selector::parse("a").unwrap();

    let mut link_file_list: Vec<String> = Vec::new();

    for element in document.select(&selector) {
        let href = element.value().attr("href").unwrap_or_default();

        if href.ends_with(extension) {
            if href.starts_with("http") || href.starts_with("https") {
                link_file_list.push(href.to_string());
            } else {
                let new_href =
                    url_str.scheme().to_string() + "://" + url_str.host_str().unwrap() + href;
                link_file_list.push(new_href);
            }
        }
    }

    Ok(link_file_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_all_files_links_for_pdf_extension_returns_list() {
        let exp_list_of_pdfs = vec![
            "https://www.test.com/file_pdf_1.pdf".to_string(),
            "https://www.test.com/file_pdf_2.pdf".to_string(),
            "https://www.test.com/secret/lab/file_pdf_secret.pdf".to_string(),
        ];

        let output = get_all_files_links("https://www.test.com", "pdf")
            .await
            .unwrap();

        for exp_url in exp_list_of_pdfs.iter() {
            assert!(output.contains(exp_url));
        }
    }

    #[tokio::test]
    async fn test_get_all_files_links_for_txt_extension_returns_list() {
        let exp_list_of_txts = vec!["https://www.test.com/file_txt_1.txt".to_string()];

        let output = get_all_files_links("https://www.test.com", "txt")
            .await
            .unwrap();

        for exp_url in exp_list_of_txts.iter() {
            assert!(output.contains(exp_url));
        }
    }

    #[tokio::test]
    async fn test_get_all_files_links_for_invalid_extension_returns_empty_list() {
        let output = get_all_files_links("https://www.test.com", "fake_extension")
            .await
            .unwrap();

        assert!(output.is_empty());
    }
}

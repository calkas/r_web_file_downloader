use r_web_file_downloader::file_downloader;
use r_web_file_downloader::file_link_scrapper;

#[tokio::test]
async fn download_all_pdf_files_from_web() {
    let www_sample_pdf = "https://uwaterloo.ca/onbase/help/sample-pdf-documents";
    let file_link = "https://uwaterloo.ca/onbase/sites/ca.onbase/files/uploads/files";

    let exp_download_file_names = vec![
        "sampleunsecuredpdf",
        "samplecertifiedpdf",
        "samplesecured_256bitaes_pdf",
    ];

    let pdf_links = file_link_scrapper::get_all_files_links(www_sample_pdf, "pdf")
        .await
        .unwrap();

    assert_eq!(exp_download_file_names.len(), pdf_links.len());

    for item in 0..exp_download_file_names.len() {
        assert_eq!(
            pdf_links[item],
            format!("{}/{}.pdf", file_link, exp_download_file_names[item])
        );
    }

    let download_path = "./test_files";
    std::fs::create_dir(download_path).unwrap();

    let result = file_downloader::download_listed_files(&pdf_links, download_path).await;
    assert!(result.is_ok());

    for file in exp_download_file_names.iter() {
        let file_path = format!("{}/{}.pdf", download_path, file);
        assert!(std::fs::metadata(file_path).is_ok());
    }

    std::fs::remove_dir_all(download_path).unwrap();
}

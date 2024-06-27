use r_web_file_downloader::file_downloader;
use r_web_file_downloader::file_link_scrapper;

#[tokio::main]
async fn main() {
    println!("..:: Welcome to the Web File Downloader ::..");

    let output = file_link_scrapper::get_all_files_links(
        "https://uwaterloo.ca/onbase/help/sample-pdf-documents",
        "pdf",
    )
    .await
    .unwrap();

    for link in output.iter() {
        println!("{}", link);
    }

    file_downloader::download_listed_files(
        &output,
        "/home/prybka/Programming/Rust_Workspace/r_web_file_downloader/",
    )
    .await
    .unwrap();
}

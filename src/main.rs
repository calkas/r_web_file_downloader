use std::path::Path;

use r_web_file_downloader::file_downloader;
use r_web_file_downloader::file_link_scrapper;

struct Args {
    link: String,
    extension: String,
}
impl Args {
    fn are_empty(&self) -> bool {
        self.link.is_empty() || self.extension.is_empty()
    }
}

/// # parse_input_args
/// Parse following input:
///
/// ./r_web_file_downloader --url https://uwaterloo.ca/onbase/help/sample-pdf-documents --e pdf
fn parse_input_args() -> Args {
    let mut link = String::new();
    let mut extension = String::new();
    {
        let mut arg_parser = argparse::ArgumentParser::new();
        arg_parser.set_description("Web File Downloader");
        arg_parser.refer(&mut link).add_option(
            &["--url"],
            argparse::Store,
            "URL to download files from",
        );
        arg_parser.refer(&mut extension).add_option(
            &["--e"],
            argparse::Store,
            "File extension to download",
        );
        arg_parser.parse_args_or_exit();
    }
    Args { link, extension }
}

#[tokio::main]
async fn main() {
    println!("\x1b[96m..:: Welcome to the Web File Downloader ::..\x1b[0m\n");

    let args = parse_input_args();

    if args.are_empty() {
        println!("Please provide a link and extension to download files.");
        return;
    }

    let output = file_link_scrapper::get_all_files_links(&args.link, &args.extension)
        .await
        .unwrap();

    for (id, link) in output.iter().enumerate() {
        println!("{}) {}", id + 1, link);
    }

    let download_path = "./download_files";

    // Check if the directory exists
    if !Path::new(download_path).exists() {
        std::fs::create_dir(download_path).unwrap();
    }

    println!("\nDownload Started...");

    file_downloader::download_listed_files(&output, download_path)
        .await
        .unwrap();
}

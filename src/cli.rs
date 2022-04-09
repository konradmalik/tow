use crate::download;
use clap::{Parser, Subcommand};
use reqwest::Client;
use url;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install { name: String },
    List,
    Uninstall { name: String },
    Versions { name: String },
}

pub async fn run_cli() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    match &cli.command {
        Commands::Install { name } => {
            println!("'install' was used, name is: {:?}", name);
            let file_url_str =
                "https://github.com/konradmalik/py4envi/archive/refs/tags/v0.0.3.zip";
            match url::Url::parse(file_url_str) {
                Err(e) => {
                    println!("Error parsing url: {}", e)
                }
                Ok(url) => {
                    println!("downloading file: {}", file_url_str);
                    let client = Client::new();
                    match download::download_file(&client, &url, "/home/konrad/Downloads/file.zip")
                        .await
                    {
                        Err(e) => {
                            println!("Error parsing url: {:?}", e)
                        }
                        Ok(()) => {
                            println!("downloaded!")
                        }
                    }
                }
            }
        }
        Commands::List => {
            println!("'list' was used")
        }
        Commands::Uninstall { name } => {
            println!("'uninstall' was used, name is: {:?}", name)
        }
        Commands::Versions { name } => {
            println!("'versions' was used, name is: {:?}", name)
        }
    }
}

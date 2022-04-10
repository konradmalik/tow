use crate::download;
use clap::{Parser, Subcommand};
use log::{error, info};
use std::path::Path;

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
            println!("'install' was used, name is: {}", name);
            let file_url_str =
                "https://github.com/konradmalik/py4envi/archive/refs/tags/v0.0.3.zip";
            match url::Url::parse(file_url_str) {
                Err(e) => {
                    error!("Error parsing url: {}", e)
                }
                Ok(url) => {
                    info!("downloading file: {}", file_url_str);
                    let path = Path::new("/home/konrad/Downloads");
                    match download::download_file(&url, path).await {
                        Err(e) => {
                            error!("Error downloading url: {}", e)
                        }
                        Ok(()) => {
                            info!("downloaded!")
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

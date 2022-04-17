use crate::app::App;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install { url: String },
    List,
    Uninstall { name: String },
    Versions { name: String },
}

pub async fn run_cli(app: App<'static>) {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    match &cli.command {
        Commands::Install { url } => {
            app.install_binary(url)
                .await
                .expect("could not install binary; see previous errors");
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

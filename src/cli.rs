use crate::{app::App, store};
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
    Uninstall { name: String, version: String },
}

pub async fn run_cli<T: store::TowStore>(mut app: App<T>) {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    match &cli.command {
        Commands::Install { url } => {
            app.install(url, None, None)
                .await
                .expect("could not install binary; see previous errors");
        }
        Commands::List => {
            for be in app.list() {
                println!("{}", be)
            }
        }
        Commands::Uninstall { name, version } => {
            app.remove(name.to_string(), version.to_string())
                .expect("could not uninstall binary; see previous errors");
        }
    }
}

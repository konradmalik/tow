use clap::{AppSettings, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::PropagateVersion))]
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

fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    match &cli.command {
        Commands::Install { name } => {
            println!("'install' was used, name is: {:?}", name)
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

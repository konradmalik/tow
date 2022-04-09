mod cli;
mod download;

#[tokio::main]
async fn main() {
    cli::run_cli().await
}

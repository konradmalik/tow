mod cli;
mod download;
mod errors;
mod logs;

#[tokio::main]
async fn main() {
    logs::init();

    cli::run_cli().await
}

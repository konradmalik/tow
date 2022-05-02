mod app;
mod cli;
mod download;
mod errors;
mod local_store;
mod logs;
mod store;

#[tokio::main]
async fn main() {
    logs::init(3);

    let app = app::App::new_from_env().expect("cannot start the application");
    cli::run_cli(app).await
}

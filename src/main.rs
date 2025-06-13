mod app;
mod config;
mod database;
mod web;

use app::App;
use log::error;

#[tokio::main]
async fn main() {
    env_logger::init();

    let app = match App::new().await {
        Ok(app) => app,
        Err(e) => {
            error!("Failed to initialize application: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = app.run().await {
        error!("Application error: {}", e);
        std::process::exit(1);
    }
}

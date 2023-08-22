use rinha::{
    config::{get_config, Server},
    server::{start_server, InitializedServer},
    tracing::configure_tracing,
};

use anyhow::Result;
use sqlx::{migrate, postgres::PgPoolOptions};

#[actix_web::main]
async fn main() -> Result<()> {
    let config = get_config()?;

    let connection = config.database.get_connection_string();
    let pool = PgPoolOptions::new()
        .max_connections(config.database.maxconnections)
        .min_connections(config.database.maxconnections - 5)
        .connect(&connection)
        .await?;

    if let Some(honeycomb_config) = &config.honeycomb {
        configure_tracing(&honeycomb_config.apikey)?;
    }

    migrate!("./migrations").run(&pool).await?;

    let Server { ip, port } = config.server;

    let InitializedServer { server, .. } = start_server(ip, port, pool).await?;
    server.await?;
    Ok(())
}

use actix_web::{dev::Server, web::Data, App, HttpServer};
use sqlx::PgPool;
use std::net::{IpAddr, TcpListener};
use tracing::{info, instrument};
use tracing_actix_web::TracingLogger;

use crate::controllers::{
    count_persons::count_persons, create_person::create_person, get_person::get_person,
    search_persons::search_persons,
};

pub struct InitializedServer {
    pub server: Server,
    pub server_address: String,
}

#[instrument]
pub async fn start_server(
    ip: IpAddr,
    port: u16,
    db_pool: PgPool,
) -> Result<InitializedServer, std::io::Error> {
    let listener = TcpListener::bind((ip, port))?;
    let server_address = listener.local_addr()?.to_string();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_pool.clone()))
            .wrap(TracingLogger::default())
            .service(count_persons)
            .service(create_person)
            .service(get_person)
            .service(search_persons)
    })
    .listen(listener)?
    .run();

    info!("Server listening on address {}", server_address);

    Ok(InitializedServer {
        server,
        server_address,
    })
}

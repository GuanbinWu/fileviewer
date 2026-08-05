// use std::net::SocketAddr;
// use warp::{Filter, filters::body::json};
mod route;
mod handlers;
mod config;
mod auth;
mod database;
mod cli;
mod errors;
mod files;
use std::sync::Arc;
#[tokio::main]
async fn main() {
    let config=config::Config::new();
    let store=database::Store::new(&config.db_url).await;

    store.sync_disk(Arc::new(config.clone())).await.unwrap();
    println!("数据库同步成功");
    let app = route::router(Arc::new(config.clone()),Arc::new(store));
    // let login =warp::path("login").map(||format!("Login!"));
    let addr =std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(config.ip[0], config.ip[1], config.ip[2], config.ip[3])), config.port);
    warp::serve(app).run(addr).await;
}



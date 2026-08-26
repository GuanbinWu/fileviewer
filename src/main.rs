mod route;
mod handlers;
mod config;
mod auth;
mod database;
mod cli;
mod errors;
mod files;
mod api;
use std::sync::Arc;

use crate::route::AppState;


#[tokio::main]
async fn main() {
    let config=Arc::new(config::Config::new());
    let store=database::Store::new(&config.db_url).await;
    
    store.sync_zones(config.clone()).await.unwrap();
    println!("文件区同步成功");
    store.sync_files(config.clone()).await.unwrap();
    println!("文件同步成功");
    let app = route::router(AppState::new(config.clone(), Arc::new(store)));
    // let login =warp::path("login").map(||format!("Login!"));
    let addr =std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(config.ip[0], config.ip[1], config.ip[2], config.ip[3])), config.port);
    warp::serve(app).run(addr).await;
}



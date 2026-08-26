
use crate::{auth::{self, Account}, config::{self, Config}, database::Store, handlers::{self, verify_handlers}, route};
use std::{clone, collections::{HashMap, HashSet}, os::windows::fs::FileExt};

use bytes::Bytes;
// use clap::error::Error;
// use serde_json::Error;
use warp::{Filter, filters::{body::json, path::{Exact, FullPath, path, tail}, query::query}, http::{Method, StatusCode}, reject::{Reject, Rejection}, reply::{Json, Reply}};
use serde::{Deserialize, Serialize,};
use std::sync::Arc;
use  crate::api::*;

#[derive(Clone)]
pub struct AppState{
    pub config:Arc<Config>,
    pub store:Arc<Store>,
}

impl AppState {
    pub fn new(config: Arc<Config>  ,store:Arc<Store>)->Self{
        AppState { config, store }
    }
}

// type RqRenameZone = 


#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Request{
    // Upload{}
    pub filename:String,
    pub is_dir:bool,
    pub args:Option<String>,
    pub bytes:Option<String>,
}

pub fn router(state:AppState)-> impl Filter<Extract = impl Reply, Error = Rejection> + Clone{

    let statics = warp::path("static").and(warp::fs::dir("./web_resources/static"));

    statics
        .or(portal(state.clone()))
        .or(account_filter(state.clone())
        .or(files_filter(state.clone()))
        .or(log_filter(state.clone()))
        .or(session_filter(state.clone()))
        .or(zone_filter(state))
    )
    
}

fn with_state(state:AppState)->impl Filter<Extract = (AppState,),Error = std::convert::Infallible>+Clone{
    warp::any().map(move || state.clone() )
}

fn portal(state:AppState)->impl Filter<Extract = impl Reply,Error = Rejection>+Clone{ 
    let tmp = state.clone();
    let login_portal = 
        warp::path("portal")
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::get())
        .map( move || {
            let mut path=tmp.config.web_resources.clone();
            path.push("pages/login.html");
            let page = std::fs::read_to_string(&path).unwrap_or_default();
            warp::reply::html(page)
        });
    
    let main_portal = 
        warp::path("portal")
        .and(warp::path("files"))
        .and(warp::path::end())
        .and(warp::get())
        .map( move || {
            let mut path=state.config.web_resources.clone();
            path.push("pages/main.html");
            let page = std::fs::read_to_string(&path).unwrap_or_default();
            warp::reply::html(page)
        });
    
    login_portal.or(main_portal)

}


fn account_filter(state:AppState) ->impl Filter<Extract = impl  Reply,Error=Rejection>+Clone{

    let base = 
    warp::path("api")
    .and(warp::path("accounts"))
    .and(with_state(state));

    let login = base.clone()
    .and(warp::path("login"))
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json::<RqAccountLogin>())
    .and_then(handlers::login_handler);

    let regi = base.clone()    
    .and(warp::path("regist"))
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json::<RqAccountRegist>())
    .and_then(handlers::register_handler);

    let new_pwd = base.clone()    
    .and(warp::path("updatepwd"))
    .and(warp::path::end())
    .and(warp::patch())
    .and(warp::body::json::<RqAccountNewPwd>())
    .and_then(handlers::new_pwd_handler);

    let logout = base.clone()
    .and(warp::path("logout"))
    .and(warp::path::end())
    .and(warp::post())
    .and(auth::auth())
    .and_then(handlers::logout_handler);

    let delete_account=base.clone()
    .and(warp::path("delete"))
    .and(warp::path::end())
    .and(warp::delete())
    .and(warp::body::json::<RqAccountDelete>()) // 独立 body
    .and_then(handlers::delete_account_handler);

    let list =base.clone()
    .and(warp::path("list"))
    .and(warp::path::end())
    .and(warp::get())
    .and(auth::auth())
    .and_then(handlers::list_account_handler);

    login.or(regi).or(new_pwd).or(logout).or(delete_account).or(list)
}

fn zone_filter(state:AppState)->impl Filter<Extract = impl Reply,Error=Rejection>+Clone{
    // GET /api/zone/list
    let base = warp::path("api")
    .and(warp::path("zone"));

    let list = base
    .and(warp::path("list"))
    .and(warp::path::end())
    .and(warp::get())
    .and(with_state(state.clone()))
    .and(auth::auth())
    .and_then(handlers::zone_list_handler);
    
    let rename = base
        .and(warp::path("rename"))
        .and(warp::path::end())
        .and(warp::patch())
        .and(with_state(state.clone()))
        .and(auth::auth())
        .and(warp::body::json::<RqZoneRename>())
        .and_then(handlers::zone_rename_handler);

    let create = base
        .and(warp::path("create"))
        .and(warp::path::end())
        .and(warp::post())
        .and(with_state(state.clone()))
        .and(auth::auth())
        .and(warp::body::json::<RqZoneCreate>())
        .and_then(handlers::zone_create_handler);

    let newlords = base
        .and(warp::path("newlords"))
        .and(warp::path::end())
        .and(warp::patch())
        .and(with_state(state.clone()))
        .and(auth::auth())
        .and(warp::body::json::<RqZoneNewLords>())
        .and_then(handlers::zone_newlords_handler);
    
    let tree = base
        .and(warp::path("tree"))
        .and(warp::path::end())
        .and(warp::get())
        .and(with_state(state.clone()))
        .and(auth::auth())
        .and(warp::query::<HashMap<String,String>>())
        .and_then(handlers::zone_tree_handler);
    
    let delete = base
    .and(warp::path("delete"))
    .and(warp::path::end())
    .and(warp::delete())
    .and(with_state(state.clone()))
    .and(auth::auth())
    .and(warp::query::<HashMap<String,String>>())
    .and_then(handlers::zone_delete_handler);

    let size = base
    .and(warp::path("size"))
    .and(warp::path::end())
    .and(warp::get())
    .and(with_state(state.clone()))
    .and(auth::auth())
    .and_then(handlers::zone_size_handler);

    list.or(rename).or(create).or(newlords).or(tree).or(delete).or(size)
}

fn files_filter(state:AppState) ->impl Filter<Extract = impl Reply,Error=Rejection>+Clone{

    let base = 
    warp::path("api")
    .and(warp::path("files"))
    .and(with_state(state));

    let list_dir= base.clone()
    .and(warp::path("list"))
    .and(warp::path::end())
    .and(warp::post())
    .and(auth::auth())
    .and(warp::body::json::<RqFileList>())
    .and_then(handlers::list_handler);
    
    // Upload files or mkdir
    let upload=base.clone()
    .and(warp::path("upload"))
    .and(warp::path::end())
    .and(warp::post())
    .and(auth::auth())
    .and(warp::body::json::<RqFileUpload>())
    .and_then(handlers::upload_handler);

    let download=base.clone()
    .and(warp::path("download"))
    .and(warp::path::end())
    .and(warp::post())
    .and(auth::auth())
    .and(warp::body::json::<RqFileDownload>())
    .and_then(handlers::download_handler);
    
    // rm files or rmdir
    let delete=base.clone()
    .and(warp::path("delete"))
    .and(warp::path::end())
    .and(warp::post())
    .and(auth::auth())
    .and(warp::body::json::<RqFileDelete>())
    .and_then(handlers::delete_handler);

    //rename file or dir
    let rename=base.clone()
    .and(warp::path("rename"))
    .and(warp::path::end())
    .and(warp::patch())
    .and(auth::auth())
    .and(warp::body::json::<RqFileRename>())
    .and_then(handlers::rename_handler);

   
    //cp file or dir
    let copy_file=base.clone()
    .and(warp::path("copy"))
    .and(warp::path::end())
    .and(warp::post())
    .and(auth::auth())
    .and(warp::body::json::<RqFileCopy>())
    .and_then(handlers::copy_handler);


    // file or dir
    let ch_creater=base.clone()
    .and(warp::path("chown"))
    .and(warp::path::end())
    .and(warp::patch())
    .and(auth::auth())
    .and(warp::body::json::<RqFileChown>())
    .and_then(handlers::ch_cerater_handler);


    list_dir
    .or(upload)
    .or(download)
    .or(delete)
    .or(rename)
    .or(copy_file)
    .or(ch_creater)

}


fn log_filter(state:AppState)->impl Filter<Extract = impl Reply,Error=Rejection>+Clone{
    warp::path::path("api")
    .and(warp::path("log"))
    .and(warp::path::end())
    .and(with_state(state))
    .and(warp::get())
    .and(auth::auth())
    .and(warp::query::<HashMap<String,String>>())
    .and_then(handlers::log_handler)

}


fn mycors()->warp::filters::cors::Builder{
    warp::cors()
    .allow_any_origin()
    .allow_header("content-type")
    .allow_methods(&[Method::PUT,Method::GET,Method::DELETE,Method::POST,Method::PATCH])
}

fn session_filter(state:AppState)->impl Filter<Extract = impl Reply,Error=Rejection>+Clone{
    warp::path("api")
    .and(warp::path("auth"))
    .and(warp::path("verify"))
    .and(warp::path::end())
    .and(warp::post())
    .and(with_state(state))
    .and(warp::query::<HashMap<String,String>>())
    .and_then(verify_handlers)
}

// async fn handel_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
//     if err.find::<Unauthorized>().is_some() {
//         return Ok(warp::reply::with_status(
//             "Unauthorized",
//             StatusCode::UNAUTHORIZED,
//         ));
//     }
//     if err.find::<warp::reject::MissingHeader>().is_some() {
//         return Ok(warp::reply::with_status(
//             "Missing Authorization header",
//             StatusCode::UNAUTHORIZED,
//         ));
//     }
//     if err.is_not_found() {
//         return Ok(warp::reply::with_status(
//             "Not Found",
//             StatusCode::NOT_FOUND,
//         ));
//     }
//     Ok(warp::reply::with_status(
//         "Internal Server Error",
//         StatusCode::INTERNAL_SERVER_ERROR,
//     ))
// }


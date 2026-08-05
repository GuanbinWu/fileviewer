
use crate::{auth::{self, Account}, config::{self, Config}, database::Store, handlers, route};
use std::{collections::HashMap, os::windows::fs::FileExt};

use bytes::Bytes;
use serde_json::Error;
use warp::{Filter, filters::{body::json, path::{Exact, FullPath, path, tail}, query::query}, http::{Method, StatusCode}, reject::{Reject, Rejection}, reply::{Json, Reply}};
use serde::{Deserialize, Serialize,};
use std::sync::Arc;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct JsonRequest{
    // pub src_path=Option<String>,
    pub filename:String,
    pub is_dir:bool,
    pub args:Option<String>,
    pub bytes:Option<String>,
}

pub fn router(config:Arc<Config>,store:Arc<Store>)-> impl Filter<Extract = impl Reply, Error = Rejection> + Clone{
    
    let statics = warp::path("static").and(warp::fs::dir("./static"));
    login_base(config.clone(),store.clone())
    .or(files_base(config.clone(),store.clone()))
    .or(statics)
    .or(log_base(config, store))
}

fn login_base(config:Arc<Config>,store:Arc<Store>) ->impl Filter<Extract = impl  Reply,Error=Rejection>+Clone{
    // let mut tmp =config.wgb 
    let mut login_page_path=config.web_resources.clone();
    login_page_path.push("pages/login.html");
    let store_filter=warp::any().map(move||store.clone());
    // warp::any(config).and(warp::any(store))
    let config_filter=warp::any().map(move||config.clone());
 
    let base = warp::path("login")
        .and(config_filter.clone())
        .and(store_filter.clone());
    // GET 路由
    let get = base.clone()
        .and(warp::path::end())
        .and(warp::get())
        .map(move |_, _| {
            let main_page = std::fs::read_to_string(&login_page_path).unwrap_or_default();
            println!("Main Page");
            warp::reply::html(main_page)
        });
    // 每个路由独立 body
    let login = base.clone()
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json::<Account>()) // 独立 body
        .and_then(handlers::login_handler);
    let regi = base.clone()
         // 独立 body
        .and(warp::post())
        .and(warp::path("regist"))
        .and(warp::path::end())
        .and(warp::body::json::<Account>())
        .and_then(handlers::register_handler);
    let new_pwd = base.clone()
        .and(warp::patch())
        .and(warp::path("updatepwd"))
        .and(warp::path::end())
        .and(warp::body::json::<Account>())
        .and(warp::query::<HashMap<String, String>>())
        .and_then(handlers::new_pwd_handler);


    let logout = base.clone()
        .and(warp::post())
        .and(warp::path("logout"))
        .and(warp::path::end())
        .and(auth::auth()) // 独立 body        
        .and_then(handlers::logout_handler);
    let delete_account=base.clone()
        .and(warp::delete())
        .and(warp::path("delete"))
        .and(warp::path::end())
        .and(warp::body::json::<Account>()) // 独立 body
        .and_then(handlers::delete_account_handler);

    let verify =base.clone()
        .and(warp::post())
        .and(warp::path("verify"))
        .and(warp::path::end())
        .and(auth::auth())
        .and_then(handlers::verify_handlers);
    get.or(login).or(regi).or(new_pwd).or(logout).or(delete_account).or(verify)
}



fn files_base(config:Arc<Config>,store:Arc<Store>) ->impl Filter<Extract = impl Reply,Error=Rejection>+Clone{
    let mut main_page_path=config.web_resources.clone();
    main_page_path.push("pages/main.html");
    let store_filter=warp::any().map(move||store.clone());
    let config_filter=warp::any().map(move||config.clone());
 
    let base = warp::path("files")
        .and(config_filter.clone())
        .and(store_filter.clone());
        // .and(auth::auth());

    let need_auth=base.clone().and(auth::auth());

    let main=base.clone()
    .and(warp::path::end())
    .and(warp::get())
    .map(move|_,_|{
        println!("Into Main page");
        let main_page=std::fs::read_to_string(&main_page_path).unwrap_or(String::from("Empty Page"));
        warp::reply::html(main_page)
    });


    let list_dir= need_auth.clone()
    .and(warp::path("list"))
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json::<JsonRequest>())
    .and_then(handlers::list_handler);
    
    // Upload files or mkdir
    let upload=need_auth.clone()
    .and(warp::path("upload"))
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json::<JsonRequest>())
    .and_then(handlers::upload_handler);

    let download=need_auth.clone()
    .and(warp::path("download"))
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json::<JsonRequest>())
    .and_then(handlers::download_handler);
    
    // rm files or rmdir
    let delete=need_auth.clone()
    .and(warp::path("delete"))
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json::<JsonRequest>())
    .and_then(handlers::delete_handler);

    //rename file or dir
    let rename=need_auth.clone()
    .and(warp::path("rename"))
    .and(warp::path::end())
    .and(warp::patch())
    .and(warp::body::json::<JsonRequest>())
    .and_then(handlers::rename_handler);

   
    //cp file or dir
    let copy_file=need_auth.clone()
    .and(warp::path("copy"))
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json::<JsonRequest>())
    .and_then(handlers::copy_handler);


    // file or dir
    let ch_creater=need_auth.clone()
    .and(warp::path("chown"))
    .and(warp::path::end())
    .and(warp::patch())
    .and(warp::body::json::<JsonRequest>())
    .and_then(handlers::ch_cerater_handler);


    list_dir
    .or(upload)
    .or(download)
    .or(delete)
    .or(rename)
    .or(copy_file)
    .or(main)
    .or(ch_creater)

}


fn log_base(config:Arc<Config>,store:Arc<Store>)->impl Filter<Extract = impl Reply,Error=Rejection>+Clone{
    warp::path::path("log")
    .and(warp::any().map(move||config.clone()))
    .and(warp::any().map(move||store.clone()))
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




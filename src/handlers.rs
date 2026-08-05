use std::{cell::Ref, collections::HashMap, path::Path};

use chrono::Utc;
use sqlx::encode::IsNull::No;
use std::path::PathBuf;
use warp::{Filter, filters::{body::json, path::{Exact, path}, query::query}, http::{Method, StatusCode}, reject::{Reject, Rejection}, reply::{Json, Reply}};
use serde::{Serialize,Deserialize, de::value::Error};

use base64::Engine;
use std::sync::Arc;
use crate::{database::Store, errors::{AuthError::{self, NoSuchUser},FVErrors}, files::{FileEntryResponse, add_disk_base_prefix, get_parents, replace_prefix, rm_prefix}};
use crate::errors::{self, error_to_rejection};
use crate::files::{self, FileEntry,};
use crate::auth::{self, Account, Session};
use auth::{UserAction,Attempt,Event};
use crate::config::Config;

use crate::route::JsonRequest;
// Login
pub async fn register_handler(config:Arc<Config>,store:Arc<Store>,body:Account)->Result<impl Reply,Rejection>{
    println!("CD Regis");
    match auth::regist(body, config, store).await {
        Ok(_)=> Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Mkdir Success"})),
            StatusCode::OK,
                )),
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

pub async  fn login_handler(config:Arc<Config>,store:Arc<Store>,body:Account)->Result<impl Reply,Rejection>{
    let username = &body.username.clone();
    match auth::login(body, config, store).await {
        Ok(_)=> {
            let token = auth::create_token(username, 1, Utc::now());
            println!("Token created : {}",token.clone());
            
            Ok(warp::reply::json(&serde_json::json!({
                "status": "success",
                "token": token
            })))
            // Ok(warp::reply::with_status("Login Success", StatusCode::OK)),
        }
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

pub async fn new_pwd_handler(config:Arc<Config>,store:Arc<Store>,body:Account,new_pwd:HashMap<String,String>)->Result<impl Reply,Rejection>{
    let new_pwd=new_pwd.get("newpwd").unwrap().to_string();

    match auth::update_pwd(body,new_pwd, config, store).await {
        Ok(_)=> Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Update Password Success"})),
            StatusCode::OK,
                )),
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

pub async fn logout_handler(config:Arc<Config>,store:Arc<Store>,session:Session)->Result<impl Reply,Rejection>{
    let username = &session.username.clone();
    match auth::record_logout( config, store,session).await {
        Ok(_)=> {
            let nbf = Utc::now() - chrono::Duration::days(365 * 100);
            let token = auth::create_token(username, 0, nbf);
            Ok(warp::reply::json(&serde_json::json!({
                "status": "success",
                "token": token
            })))
        }
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

pub async fn delete_account_handler(config:Arc<Config>,store:Arc<Store>,body:Account)->Result<impl Reply,Rejection>{
    // let new_pwd=new_pwd.get("new_pwd").unwrap().to_string();
    match auth::delete_account(body,config, store).await {
        Ok(_)=> Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Delete Account Success"})),
            StatusCode::OK,
                )),
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

//??????????????????????????????????????????????
pub async fn  verify_handlers(config:Arc<Config>,store:Arc<Store>,session:Session)->Result<impl Reply,Rejection>{
    let username = &session.username.clone();
    match auth::record_logout( config, store,session).await {
        Ok(_)=> {
            let nbf = Utc::now() - chrono::Duration::days(365 * 100);
            let token = auth::create_token(username, 0, nbf);
            Ok(warp::reply::json(&serde_json::json!({
                "status": "success",
                "token": token
            })))
        }
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

//Files

pub async fn upload_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest)->Result<impl Reply,Rejection>{
    store.log_try(&session, UserAction::Upload, &rq.filename, &rq.args).await?;

    if rq.is_dir {
        mkdir_handler(config.clone(), store.clone(), session.clone(), rq.clone()).await.map(|r|r.into_response())
    } else {
        let disk_path=add_disk_base_prefix(&PathBuf::from( rq.filename.clone()), config.clone());
        println!("Uploading files : {:?}",&disk_path);
        let origin_md5=rq.args.clone().unwrap_or_default();
        

        let bytes = match rq.bytes {
            Some(b64_str) => {
                base64::engine::general_purpose::STANDARD
                    .decode(&b64_str)
                    .map_err(|e| warp::reject::custom(FVErrors::IOError( format!("{:?}",e))))?
            }
            None => Vec::new(),
        };

        // 3. 获取文件元数据 (文件名作为 name)
        let size = bytes.len() as i64;
        let content_type = mime_guess::from_path(disk_path.clone())
            .first_or_octet_stream()
            .to_string();
        //核对md5
        let md5 = {
            let digest = md5::compute(&bytes);
            format!("{:x}", digest)
        };


        if md5 != origin_md5 {
            println!("MD5核对失败");
            return Err(warp::reject::custom(FVErrors::IOError( "md5核对不上，文件传输错误".to_string())))
        }
    
        match store.get_file_entry(disk_path.clone()).await
        {
            Ok(f) => {
                store.update_files_info_time(f.id, "modified_at".to_string(), Utc::now()).await?;
                store.update_files_info_string(f.id, "last_modifier".to_string(), session.username.clone()).await?;
            }
            Err(_) =>{
                let file = FileEntry { 
                id: 0, 
                name: disk_path.clone(),
                parent_name:get_parents(&disk_path), 
                is_directory: false, 
                size, 
                content_type, 
                md5:Some(md5), 
                created_at:Utc::now(),
                modified_at: Utc::now(),
                creator: session.username.clone(),
                last_modifier: session.username.clone() };
                
                store.upload_files(file).await.map_err(|e|error_to_rejection(e))?;
            }
        }

        // 5. 写入磁盘
        
        tokio::fs::write(&disk_path, &bytes)
            .await
            .map_err(|e| FVErrors::IOError(e.to_string()))?;


        store.log_sucess(&session, UserAction::Upload, &rq.filename, &rq.args).await?;
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Upload Success"})),
            StatusCode::OK,
                ).into_response())
}}


pub async fn list_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest,)->Result<impl Reply,Rejection>{
    if !rq.is_dir{
        return Err(error_to_rejection(FVErrors::NotFound));
    }else {     
    store.log_try(&session,UserAction::List, &rq.filename, &rq.args).await?;
    let disk_dir=add_disk_base_prefix(&PathBuf::from(rq.filename.clone()), config.clone());
    println!("Listing {:?}",disk_dir.clone());

    let result = store.list_dir(disk_dir.clone()).await.map_err(|e|error_to_rejection(e))?;
    let result:Vec<FileEntryResponse> = result.into_iter()
    .map(|f|f.into_response(config.clone()))
    .collect();
    
    // println!("{:?}",&result);

    store.log_sucess(&session, UserAction::List, &rq.filename, &rq.args).await?;
    Ok(warp::reply::json(&result))
}}



pub async fn download_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest)->Result<impl Reply,Rejection>{
    store.log_try(&session, UserAction::Download, &rq.filename, &rq.args).await?;
    let disk_path = add_disk_base_prefix(&PathBuf::from(rq.filename.clone()), config.clone());
    

    // 查看数据库有无文件
    let file_entry=store.get_file_entry(disk_path.clone())
    .await
    .map(|f|f.into_response(config.clone()))
    .map_err(|e|error_to_rejection(e))?;
    //读取字节
    let content = tokio::fs::read(&disk_path).await.map_err(|e|   warp::reject::custom(FVErrors::IOError(e.to_string())))?;

    // let content_type = ;
    store.log_sucess(&session, UserAction::Download, &rq.filename, &rq.args).await?;
    // Ok(warp::reply::with_header(content, "Content-Type", file_entry.content_type).into_response())
    
    Ok(warp::http::Response::builder()
    .status(200)
    .header("Content-Type", file_entry.content_type)
    .header("X-Content-MD5", &file_entry.md5.unwrap())
    .body(warp::hyper::Body::from(content))
    .unwrap())
    // Ok(warp::reply::with_status(
    //         warp::reply::json(&serde_json::json!({"message": "Download Success",
    //         "bytes":content,
    //         "content_type":file_entry.content_type,
    //         "md5":file_entry.md5})),
    //         StatusCode::OK,
    //     ))
}

pub async fn delete_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest,)->Result<impl Reply,Rejection>{
    if rq.is_dir {
        rmdir_handler(config, store, session, rq).await.map(|r|r.into_response())
    }else{

    store.log_try(&session, UserAction::DeleteFile, &rq.filename, &rq.args).await?;

   let disk_path = add_disk_base_prefix(&PathBuf::from(rq.filename.clone()), config.clone());
    
    store.delete_files(disk_path.clone()).await?;

    tokio::fs::remove_file(disk_path.clone()).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;
    
    store.log_sucess(&session, UserAction::DeleteFile, &rq.filename, &rq.args).await?;
    Ok(warp::reply::with_status(
    warp::reply::json(&serde_json::json!({"message": "Delete Success"})),
    StatusCode::OK,
        ).into_response())
}
}

pub async fn rename_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest)->Result<impl Reply,Rejection>{
    // println!("Into Rename Handler");
    store.log_try(&session, UserAction::Rename, &rq.filename, &rq.args).await?;
    if let None =rq.args {
        return Err(error_to_rejection(FVErrors::NotFound));
    }

    let disk_path = add_disk_base_prefix(&PathBuf::from(rq.filename.clone()), config.clone());
    let new_disk_path =add_disk_base_prefix(&PathBuf::from(rq.args.clone().unwrap()), config.clone());
    
    match store.get_file_entry(new_disk_path.clone()).await
    {
        Err(_) =>
        {
            println!("!");
            let f = store.get_file_entry(disk_path.clone()).await.map_err(|_|error_to_rejection(FVErrors::IOError("被重命名的源文件不存在".to_string())))?;

            let meta = tokio::fs::metadata(disk_path.clone()).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;
            if meta.is_dir() != rq.is_dir{
                return Err(error_to_rejection(FVErrors::NotFound));
            }
            if meta.is_dir(){
                store.rename_dir(disk_path.clone(), new_disk_path.clone()).await.map_err(|e|error_to_rejection(e))?;

            }else {
                println!("将文件{:?}重命名为{:?}",&disk_path,&new_disk_path);
                store.update_files_info_pthbuf(f.id,  "parent_name".to_string(), get_parents(&new_disk_path)).await.map_err(|e|error_to_rejection(e))?;
                store.update_files_info_pthbuf(f.id,  "name".to_string(), new_disk_path.clone()).await.map_err(|e|error_to_rejection(e))?;
                
            }
            tokio::fs::rename(disk_path.clone(), new_disk_path).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;
            store.log_sucess(&session, UserAction::Rename, &rq.filename, &rq.args).await?;
            Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Rename Success"})),
            StatusCode::OK,
                ))
        },
        Ok(_) => return Err(error_to_rejection(FVErrors::IOError("新文件名已被占用".to_string()))),
    }
    
}


pub async fn copy_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest)->Result<impl Reply,Rejection>{
    if rq.is_dir{
        cpdir_handler(config, store, session, rq).await.map(|r|r.into_response())
    }else{
    store.log_try(&session, UserAction::CpFile, &rq.filename, &rq.args).await?;
    if let None =rq.args {
        return Err(error_to_rejection(FVErrors::NotFound));
    }
    let disk_path = add_disk_base_prefix(&PathBuf::from(rq.filename.clone()), config.clone());
    let new_disk_path =add_disk_base_prefix(&PathBuf::from(rq.args.clone().unwrap()), config.clone());

    //文件是否存在
    let file = store.get_file_entry(disk_path.clone())
    .await.map_err(|e|error_to_rejection(e))?;

    store.upload_files(
        FileEntry { 
        id:0, 
        name: new_disk_path.clone(),
        parent_name:get_parents(&new_disk_path), 
        is_directory: false, 
        size: file.size, 
        content_type: file.content_type, 
        md5: file.md5, 
        created_at: Utc::now(), 
        modified_at: Utc::now(), 
        creator: session.username.clone(), 
        last_modifier: session.username.clone() }).await.map_err(|e|error_to_rejection(e))?;

    tokio::fs::copy(disk_path.clone(), new_disk_path).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;

    store.log_sucess(&session, UserAction::CpFile, &rq.filename, &rq.args).await?;
    Ok(warp::http::Response::builder()
        .header("Content-Type", "text/plain")
        .body("Copy Success")
        .unwrap().into_response())
}}

pub async fn cpdir_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest)->Result<impl Reply,Rejection>{
    store.log_try(&session, UserAction::CpFile, &rq.filename, &rq.args).await?;
    let disk_path=add_disk_base_prefix(&PathBuf::from(rq.filename.clone()),config.clone());
    let new_disk_path =add_disk_base_prefix(&PathBuf::from(rq.args.clone().unwrap()), config.clone());
    match store.get_dir_entry(new_disk_path.clone()).await {
        Ok(_) => return Err(error_to_rejection(FVErrors::IOError("新文件名已被占用".to_string()))),
        Err(_) => {    
        let entries = store
            .list_dir_all(disk_path.clone())
            .await
            .map_err(error_to_rejection)?;
        // println!("{:?}",&entries);
        
        let mut this_dir = store.get_dir_entry(disk_path.clone()).await?;
        this_dir.name = new_disk_path.clone();
        this_dir.parent_name =get_parents(&new_disk_path);
        tokio::fs::create_dir_all(&&this_dir.name).await.map_err(|e| error_to_rejection(FVErrors::IOError(e.to_string())))?;
        store.upload_files(this_dir)
            .await
            .map_err(error_to_rejection)?;

        let new :Vec<(PathBuf,FileEntry)>= entries.into_iter()
    .map(|f|{
        let prefix = add_disk_base_prefix(&PathBuf::from(rq.filename.clone()),config.clone());
        let new_prefix = add_disk_base_prefix(&PathBuf::from(rq.args.clone().unwrap()), config.clone());
        let new_name= replace_prefix(&f.name, &prefix, &new_prefix);
        // println!("New name {:?}",&new_name);
        (f.name,FileEntry{
        id: 0,
        name: new_name.clone(),
        parent_name:get_parents(&new_name),
        is_directory: f.is_directory,
        size: f.size,
        content_type: f.content_type,
        md5: f.md5,
        created_at: Utc::now(),
        modified_at: Utc::now(),
        creator:session.username.clone(),
        last_modifier:session.username.clone(),
        })}
    )
    .collect();


        for entry in new.iter() {
            if entry.1.is_directory {
                tokio::fs::create_dir_all(&entry.1.name)
                    .await
                    .map_err(|e| error_to_rejection(FVErrors::IOError(e.to_string())))?;
            } else {
                if let Some(parent) = Path::new(&entry.1.name).parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| error_to_rejection(FVErrors::IOError(e.to_string())))?;
                }
                // 磁盘复制
                tokio::fs::copy(&entry.0, &entry.1.name)
                    .await
                    .map_err(|e| error_to_rejection(FVErrors::IOError(e.to_string())))?;
            }

            store
                .upload_files(entry.1.clone())
                .await
                .map_err(error_to_rejection)?;
        }
    }}
    store.log_sucess(&session, UserAction::CpFile, &rq.filename, &rq.args).await?;
    Ok(warp::reply::with_status(
    warp::reply::json(&serde_json::json!({"message": "Cpdir Success"})),
    StatusCode::OK,
    ))
}

pub async fn mkdir_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest)->Result<impl Reply,Rejection>{
    store.log_try(&session, UserAction::Mkdir, &rq.filename, &rq.args).await?;

    let disk_path=add_disk_base_prefix(&PathBuf::from( rq.args.clone().unwrap()), config.clone());
    println!("Mkdir : {:?}",disk_path);
    let file = FileEntry { 
        id: 0, 
        name: disk_path.clone(),
        parent_name:get_parents(&disk_path), 
        is_directory: true, 
        size:0, 
        content_type:"Dir".to_string(), 
        md5:None, 
        created_at:Utc::now(),
        modified_at: Utc::now(),
        creator: session.username.clone(),
        last_modifier: session.username.clone() };
    // println!("{:?}",file);
    store.upload_files(file).await.map_err(|e|error_to_rejection(e))?;


    tokio::fs::create_dir_all(disk_path.clone()).await.map_err(|e| error_to_rejection(FVErrors::IOError(e.to_string())))?;


    store.log_sucess(&session, UserAction::Mkdir, &rq.filename, &rq.args).await?;
    Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Mkdir Success"})),
            StatusCode::OK,
                ))

}

pub async fn rmdir_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest)->Result<impl Reply,Rejection>{

    store.log_try(&session, UserAction::Rmdir, &rq.filename, &rq.args).await?;

    let disk_path = add_disk_base_prefix(&PathBuf::from(rq.filename.clone()), config.clone());
    
    store.delete_dir_all(disk_path.clone()).await.map_err(|e|error_to_rejection(e))?;

    tokio::fs::remove_dir_all(disk_path.clone()).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;
    
    store.log_sucess(&session, UserAction::Rmdir, &rq.filename, &rq.args).await?;
    Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Rmdir Success"})),
            StatusCode::OK,
                ))
}

pub async fn ch_cerater_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest)->Result<impl Reply,Rejection>{
    if let None =rq.args{
        return Err(error_to_rejection(FVErrors::NotFound));
    }
    store.log_try(&session, UserAction::Chown, &rq.filename, &rq.args).await?;
    
    
    if rq.is_dir{
        ch_dircerater_handler(config.clone(), store.clone(), session.clone(), rq.clone()).await?;
    }else {

        // let disk_path = add_disk_base_prefix(&PathBuf::from(rq.filename.clone()), config.clone());
        match store.get_file_entry(PathBuf::from(rq.filename.clone())).await {
            Ok(f) =>{
                store.update_files_info_string(f.id,"creator".to_string(),rq.args.clone().unwrap())
                .await
                .map_err(|e|error_to_rejection(e))?;
            },
            Err(_)=> return Err(error_to_rejection(FVErrors::IOError("File not exists".to_string()))),
        }
        
        
    }
    
    store.log_sucess(&session, UserAction::Chown, &rq.filename, &rq.args).await?;
    Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Change creator Success"})),
            StatusCode::OK,
        ))
}

pub async fn ch_dircerater_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:JsonRequest)->Result<impl Reply,Rejection>{
    let disk_path = add_disk_base_prefix(&PathBuf::from(rq.filename.clone()), config.clone());
    store.chdir_creator(disk_path,rq.args.unwrap())
        .await
        .map_err(|e|error_to_rejection(e))?;
    Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Change dir creator Success"})),
            StatusCode::OK,
    ))
}

pub async fn log_handler(config:Arc<Config>,store:Arc<Store>,session:Session,rq:HashMap<String,String>)->Result<impl Reply,Rejection>{
    let len= rq.get("len").unwrap_or(&"100".to_string()).parse::<i32>().map_err(|e|error_to_rejection(FVErrors::NotFound))?;
    // println!("{}",len);
    let events = store.list_log(len).await.map_err(error_to_rejection)?;
    // println!("{:?}",&events);
    Ok(warp::reply::json(&events))

}
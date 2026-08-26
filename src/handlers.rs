use std::{cell::Ref, collections::HashMap, default, path::Path, vec};

use chrono::Utc;
use serde_json::to_string;
use sqlx::encode::IsNull::No;
use std::path::PathBuf;
use warp::{Filter, filters::{body::json, path::{Exact, path}, query::query}, http::{Method, StatusCode}, reject::{Reject, Rejection}, reply::{Json, Reply}};
use serde::{Serialize,Deserialize, de::value::Error};

use base64::Engine;
use std::sync::Arc;
use crate::{api::*, auth::Attempt::Success, database::Store, errors::{AuthError::{self, NoSuchUser},FVErrors}, files::{FileZone, PathBehavior}, route::AppState};
use crate::errors::{self, error_to_rejection};
use crate::files::{self, FileEntry,};
use crate::auth::{self, Account, Session,UserAction,Attempt,Event,};
// use crate::state.config::Config;

use crate::route::Request;
// Login
pub async fn register_handler(state:AppState,body:Account)->Result<impl Reply,Rejection>{
    // println!("CD Regis");
    match auth::regist(body, state.config,state.store.clone()).await {
        Ok(_)=> {
        state.store.log_sucess(&Session::default(), UserAction::Regist, &"/api/accounts/regist".to_string(), &None).await?;    
        Ok(warp::reply::with_status(
            "Regist Success",
            StatusCode::OK,
                ))},
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

pub async  fn login_handler(state:AppState,rq:RqAccountLogin)->Result<impl Reply,Rejection>{
    let username = &rq.username.clone();
    match auth::login(rq, state.config, state.store).await {
        Ok(_)=> {
            let token = auth::create_token(username, 1, Utc::now());
            dbg!(&token);
            Ok(warp::reply::json(&serde_json::json!({
                "status": "success",
                "token": token
            })))
        }
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

pub async fn new_pwd_handler(state:AppState,rq:RqAccountNewPwd)->Result<impl Reply,Rejection>{
    let new_pwd=rq.newpwd;
    match auth::update_pwd(Account { username:rq.username, password: rq.password },new_pwd, state.config, state.store).await {
        Ok(_)=> Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Update Password Success"})),
            StatusCode::OK,
                )),
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

pub async fn logout_handler(state:AppState,session:Session)->Result<impl Reply,Rejection>{
    let username = &session.username.clone();
    match auth::record_logout( state.config, state.store,session).await {
        Ok(_)=> {
            let nbf = Utc::now() - chrono::Duration::days(365 * 100);
            let token = auth::create_token(username, 0, nbf);
            Ok(warp::reply::with_status(
            token,
            StatusCode::OK,
            ))
        },
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

pub async fn delete_account_handler(state:AppState,rq:RqAccountDelete)->Result<impl Reply,Rejection>{
    match auth::delete_account(rq,state.config, state.store).await {
        Ok(_)=> Ok(warp::reply::with_status(
            "Delete Account Success",
            StatusCode::OK,
                )),
        Err(e)=>Err(errors::error_to_rejection(e))
    }
}

pub async fn list_account_handler(state:AppState,session:Session)->Result<impl Reply,Rejection>{
    match state.store.list_account().await{
        Ok(v) => Ok(warp::reply::json(&v)),
        Err(e)=>Err(error_to_rejection(e))
    }
}
//Files




pub async fn upload_handler(state:AppState,session:Session,rq:RqFileUpload)->Result<impl Reply,Rejection>{
    // state.store.log_try(&session, UserAction::Upload, &rq.filename, &rq.args).await?;

    if rq.is_dir {
        mkdir_handler(state.clone(), session.clone(), rq.clone()).await.map(|r|r.into_response())
    } else {
        let db_path= PathBuf::from(rq.filename.clone());

        let disk_path= PathBuf::from(rq.filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();
        // println!("Uploading files : {:?}",&disk_path);
        let origin_md5=rq.md5.clone();
        

        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&rq.bytes)
            .map_err(|e| warp::reject::custom(FVErrors::IOError( format!("{:?}",e))))?;

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
    
        match state.store.get_entry(disk_path.clone(),rq.is_dir,rq.zone.clone()).await
        {
            Ok(f) => {
                state.store.update_files_info_time(f.id, "modified_at".to_string(), Utc::now(),rq.zone.clone()).await?;
                state.store.update_files_info_string(f.id, "last_modifier".to_string(), session.username.clone(),rq.zone.clone()).await?;
            }
            Err(_) =>{
                let file = FileEntry { 
                id: 0, 
                name: db_path.clone(),
                parent_name:db_path.clone().get_parent().unwrap(), 
                is_directory: false, 
                size, 
                content_type, 
                md5:Some(md5), 
                created_at:Utc::now(),
                modified_at: Utc::now(),
                creator: session.username.clone(),
                last_modifier: session.username.clone(),
                zone:rq.zone };
                
                state.store.upload_files(file).await.map_err(|e|error_to_rejection(e))?;
            }
        }

        // 5. 写入磁盘
        
        tokio::fs::write(&disk_path, &bytes)
            .await
            .map_err(|e| FVErrors::IOError(e.to_string()))?;


        state.store.log_sucess(&session, UserAction::Upload, &rq.filename, &None).await?;
        Ok(warp::reply::with_status(
            "Upload Success",
            StatusCode::OK,
                ).into_response())
}}


pub async fn list_handler(state:AppState,session:Session,rq:RqFileList,)->Result<impl Reply,Rejection>{
         
    let db_path= PathBuf::from(rq.dir.clone());

    let result = state.store.list_dir(db_path.clone(),rq.zone.clone()).await.map_err(|e|error_to_rejection(e))?;

    state.store.log_sucess(&session, UserAction::List, &rq.dir, &None).await?;
    Ok(warp::reply::json(&result))
}



pub async fn download_handler(state:AppState,session:Session,rq:RqFileDownload)->Result<impl Reply,Rejection>{
    // state.store.log_try(&session, UserAction::Download, &rq.filename, &rq.args).await?;
    if  rq.is_dir {
        download_dir_handler(state, session, rq).await.map(|v|v.into_response())
    }else{
        let db_path= PathBuf::from(rq.filename.clone());
        let disk_path= PathBuf::from(rq.filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();
        dbg!(&db_path,&disk_path);

        let file_entry=state.store.get_entry(db_path.clone(),rq.is_dir,rq.zone.clone())
        .await
        .map_err(|e|error_to_rejection(e))?;
        
        //读取字节
        let content = tokio::fs::read(&disk_path).await.map_err(|e|  warp::reject::custom(FVErrors::IOError(e.to_string())));
        // dbg!(&content);

        // let content_type = ;
        state.store.log_sucess(&session, UserAction::Download, &rq.filename, &None).await?;
        // Ok(warp::reply::with_header(content, "Content-Type", file_entry.content_type).into_response())
        
        Ok(warp::http::Response::builder()
        .status(200)
        .header("Content-Type", file_entry.content_type)
        .header("X-Content-MD5", &file_entry.md5.unwrap())
        .body(warp::hyper::Body::from(content.unwrap()))
        .unwrap())
    }

}


pub async fn download_dir_handler(state:AppState,session:Session,rq:RqFileDownload)->Result<impl Reply,Rejection>{
            
    let db_path= PathBuf::from(rq.filename.clone());
    let disk_path= PathBuf::from(rq.filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();
    dbg!(&db_path,&disk_path);
    
    let content = tokio::task::spawn_blocking(move || files::zip_dir(&disk_path))
    .await.map_err(|_|error_to_rejection(FVErrors::IOError("ZipError".to_string())))?
    .map_err(|_|error_to_rejection(FVErrors::IOError("ZipError".to_string())))?;

    let md5 = {
        let digest = md5::compute(&content);
        format!("{:x}", digest)
    };
    state.store.log_sucess(&session, UserAction::Download, &rq.filename, &None).await?;
    
    Ok(warp::http::Response::builder()
    .status(200)
    .header("Content-Type", "application/zip")
    .header("X-Content-MD5", md5)
    .body(warp::hyper::Body::from(content))
    .unwrap())

}

pub async fn delete_handler(state:AppState,session:Session,rq:RqFileDelete,)->Result<impl Reply,Rejection>{
    if rq.is_dir {
        rmdir_handler(state, session, rq).await.map(|r|r.into_response())
    }else{

    // state.store.log_try(&session, UserAction::DeleteFile, &rq.filename, &rq.args).await?;
    
    let db_path= PathBuf::from(rq.filename.clone());
    let disk_path= PathBuf::from(rq.filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();
    
    state.store.delete_files(db_path.clone(),rq.zone.clone()).await?;

    tokio::fs::remove_file(disk_path.clone()).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;
    
    state.store.log_sucess(&session, UserAction::DeleteFile, &rq.filename, &None).await?;
    Ok(warp::reply::with_status(
    "Delete Success",
    StatusCode::OK,
        ).into_response())
}
}

pub async fn rename_handler(state:AppState,session:Session,rq:RqFileRename)->Result<impl Reply,Rejection>{

    let db_path= PathBuf::from(rq.filename.clone());
    let disk_path= PathBuf::from(rq.filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();

    let new_db_path = PathBuf::from(rq.new_filename.clone());
    let new_disk_path = PathBuf::from(rq.new_filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();
    
    dbg!(&db_path,&disk_path,&new_db_path,&new_disk_path);

    match state.store.get_entry(new_db_path.clone(),rq.is_dir,rq.zone.clone()).await
    {
        Err(_) =>
        {
            let f = state.store.get_entry(db_path.clone(),rq.is_dir,rq.zone.clone())
            .await.map_err(|_|error_to_rejection(FVErrors::IOError(dbg!("被重命名的源文件不存在".to_string()))))?;
            dbg!(&f,&rq.is_dir);
            let meta = tokio::fs::metadata(disk_path.clone()).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;
            if dbg!(meta.is_dir()) != rq.is_dir{
                return Err(error_to_rejection(FVErrors::NotFound));
            }

            let a =tokio::fs::rename(disk_path.clone(), new_disk_path.clone()).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;
            dbg!(a);

            if meta.is_dir(){
                let a =state.store.rename_dir(db_path.clone(), new_db_path.clone(),rq.zone.clone()).await.map_err(|e|error_to_rejection(e));
                dbg!(a);

                state.store.update_files_info_string(f.id,  "last_modifier".to_string(), session.username.clone(),rq.zone.clone()).await.map_err(|e|error_to_rejection(e))?;

            }else {
                dbg!("将文件{:?}重命名为{:?}",&disk_path,&new_disk_path);
                state.store.update_files_info_pthbuf(f.id,  "parent_name".to_string(), new_db_path.get_parent().unwrap(),rq.zone.clone()).await.map_err(|e|error_to_rejection(e))?;

                state.store.update_files_info_pthbuf(f.id,  "name".to_string(), new_db_path.clone(),rq.zone.clone()).await.map_err(|e|error_to_rejection(e))?;
                dbg!(&session.username);

                state.store.update_files_info_string(f.id,  "last_modifier".to_string(), session.username.clone(),rq.zone.clone()).await.map_err(|e|error_to_rejection(e))?;
            }

            state.store.log_sucess(&session, UserAction::Rename, &rq.filename, &Some(rq.new_filename)).await?;
            Ok(warp::reply::with_status(
            "Rename Success",
            StatusCode::OK,
            ))
        },
        Ok(_) => return Err(error_to_rejection(FVErrors::IOError(dbg!("新文件名已被占用".to_string())))),
    }
    
}


pub async fn copy_handler(state:AppState,session:Session,rq:RqFileCopy)->Result<impl Reply,Rejection>{
    if rq.is_dir{
        cpdir_handler(state, session, rq).await.map(|r|r.into_response())
    }else{
    // state.store.log_try(&session, UserAction::CpFile, &rq.filename, &rq.args).await?;
    // if let None =rq.args {
    //     return Err(error_to_rejection(FVErrors::NotFound));
    // }
    let db_path= PathBuf::from(rq.filename.clone());

    let disk_path= PathBuf::from(rq.filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();

    let new_db_path = PathBuf::from(rq.new_filename.clone());
    let new_disk_path = PathBuf::from(rq.new_filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();

    // let disk_path = add_disk_base_prefix(&PathBuf::from(rq.filename.clone()), state.config.clone());
    // let new_disk_path =add_disk_base_prefix(&PathBuf::from(rq.args.clone().unwrap()), state.config.clone());

    //文件是否存在
    let file = state.store.get_entry(db_path.clone(),rq.is_dir,rq.zone.clone())
    .await.map_err(|e|error_to_rejection(e))?;

    state.store.upload_files(
        FileEntry { 
        id:0,
        name: new_db_path.clone(),
        parent_name:new_db_path.get_parent().unwrap(), 
        is_directory: false, 
        size: file.size, 
        content_type: file.content_type, 
        md5: file.md5, 
        created_at: Utc::now(), 
        modified_at: Utc::now(), 
        creator: session.username.clone(), 
        last_modifier: session.username.clone(),
        zone:rq.zone.clone()
     }).await.map_err(|e|error_to_rejection(e))?;

    tokio::fs::copy(disk_path.clone(), new_disk_path).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;

    state.store.log_sucess(&session, UserAction::CpFile, &rq.filename, &Some(rq.new_filename)).await?;
    Ok(warp::http::Response::builder()
        .header("Content-Type", "text/plain")
        .body("Copy Success")
        .unwrap().into_response())

    // Ok(warp::reply::with_status(
    //         "CopyFile Success",
    //         StatusCode::OK,
    //         ))
}}

pub async fn cpdir_handler(state:AppState,session:Session,rq:RqFileCopy)->Result<impl Reply,Rejection>{
    // state.store.log_try(&session, UserAction::CpFile, &rq.filename, &rq.args).await?;
    let db_path= PathBuf::from(rq.filename.clone());

    let disk_path= PathBuf::from(rq.filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();

    let new_db_path = PathBuf::from(rq.new_filename.clone());
    let new_disk_path = PathBuf::from(rq.new_filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();

    // let disk_path=add_disk_base_prefix(&PathBuf::from(rq.filename.clone()),state.config.clone());
    // let new_disk_path =add_disk_base_prefix(&PathBuf::from(rq.args.clone().unwrap()), state.config.clone());
    match state.store.get_entry(new_db_path.clone(),rq.is_dir,rq.zone.clone()).await {
        Ok(_) => return Err(error_to_rejection(FVErrors::IOError("新文件名已被占用".to_string()))),
        Err(_) => {
        let entries = state.store
            .list_dir_all(db_path.clone(),rq.zone.clone())
            .await
            .map_err(error_to_rejection)?;
        // println!("{:?}",&entries);
        
        let mut this_dir = state.store.get_entry(db_path.clone(),rq.is_dir,rq.zone.clone()).await?;
        this_dir.name = new_db_path.clone();
        this_dir.parent_name =new_db_path.get_parent().unwrap();
        
        tokio::fs::create_dir_all(new_disk_path.clone()).await.map_err(|e| error_to_rejection(FVErrors::IOError(e.to_string())))?;
        
        state.store.upload_files(this_dir)
            .await
            .map_err(error_to_rejection)?;

    let new :Vec<(PathBuf,PathBuf,PathBuf,FileEntry)>= entries.into_iter()
        .map(|f| {
        let disk_name = f.name.to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();

        let new_db_name= f.name
        .replace_prefix(
        &PathBuf::from(rq.filename.clone()),
        &PathBuf::from(rq.new_filename.clone())
        ).unwrap();

        let new_disk_name= f.name
        .to_sys_path(&state.config.sys_disk_dir, &rq.zone)
        .and_then(|f|f
            .replace_prefix(
            &PathBuf::from(rq.filename.clone())
            .to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap(),
            &PathBuf::from(rq.new_filename.clone())
            .to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap()
        )).unwrap();
 
        (f.name,
        disk_name,
        new_disk_name,
        FileEntry{
            id: 0,
            name: new_db_name.clone(),
            parent_name:new_db_name.get_parent().unwrap(),
            is_directory: f.is_directory,
            size: f.size,
            content_type: f.content_type,
            md5: f.md5,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            creator:session.username.clone(),
            last_modifier:session.username.clone(),
            zone:rq.zone.clone()})
        }).collect();


        for entry in new.iter() {
            if entry.3.is_directory {
                tokio::fs::create_dir_all(&entry.2)
                    .await
                    .map_err(|e| error_to_rejection(FVErrors::IOError(e.to_string())))?;
            } else {
                if let Some(parent) = entry.2.parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| error_to_rejection(FVErrors::IOError(e.to_string())))?;
                }
                // 磁盘复制
                tokio::fs::copy(&entry.1, &entry.2)
                    .await
                    .map_err(|e| error_to_rejection(FVErrors::IOError(e.to_string())))?;
            }

            state.store
                .upload_files(entry.3.clone())
                .await
                .map_err(error_to_rejection)?;
        }
    }}
    state.store.log_sucess(&session, UserAction::CpFile, &rq.filename, &Some(rq.new_filename)).await?;
    // Ok(warp::reply::with_status(
    // warp::reply::json(&serde_json::json!({"message": "Cpdir Success"})),
    // StatusCode::OK,
    // ))
    Ok(warp::reply::with_status(
            "CopyDir Success",
            StatusCode::OK,
        ))
}

pub async fn mkdir_handler(state:AppState,session:Session,rq:RqFileUpload)->Result<impl Reply,Rejection>{
    // state.store.log_try(&session, UserAction::Mkdir, &rq.filename, &rq.args).await?;
    let db_path = PathBuf::from(rq.filename.clone());

    let disk_path=PathBuf::from(rq.filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();
    // println!("Mkdir : {:?}",disk_path);
    let file = FileEntry {
        id: 0, 
        name: db_path.clone(),
        parent_name:db_path.clone().get_parent().unwrap(), 
        is_directory: true, 
        size:0, 
        content_type:"Dir".to_string(), 
        md5:None, 
        created_at:Utc::now(),
        modified_at: Utc::now(),
        creator: session.username.clone(),
        last_modifier: session.username.clone(),
        zone:rq.zone.clone()};
    
    match state.store.get_entry(db_path.clone(),rq.is_dir,rq.zone.clone()).await{
        Ok(v)=>{return Ok(warp::reply::with_status(
            "存在同名目录",StatusCode::BAD_REQUEST));
        },
        Err(_)=>{
            state.store.upload_files(file).await.map_err(|e|error_to_rejection(e))?;
            tokio::fs::create_dir_all(disk_path.clone()).await.map_err(|e| error_to_rejection(FVErrors::IOError(e.to_string())))?;
        }
    }

    


    


    state.store.log_sucess(&session, UserAction::Mkdir, &rq.filename, &None).await?;
    Ok(warp::reply::with_status(
        "Mkdir Success",
            StatusCode::OK,
    ))

}

pub async fn rmdir_handler(state:AppState,session:Session,rq:RqFileDelete)->Result<impl Reply,Rejection>{

    // state.store.log_try(&session, UserAction::Rmdir, &rq.filename, &rq.args).await?;
    
    let db_path= PathBuf::from(rq.filename.clone());
    let disk_path= PathBuf::from(rq.filename.clone()).to_sys_path(&state.config.sys_disk_dir, &rq.zone).unwrap();
    
    state.store.delete_dir_all(db_path.clone(),rq.zone.clone()).await.map_err(|e|error_to_rejection(e))?;

    tokio::fs::remove_dir_all(disk_path.clone()).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;
    
    state.store.log_sucess(&session, UserAction::Rmdir, &rq.filename, &None).await?;
    Ok(warp::reply::with_status(
            "Rmdir Success",
            StatusCode::OK,
    ))
}

pub async fn ch_cerater_handler(state:AppState,session:Session,rq:RqFileChown)->Result<impl Reply,Rejection>{

    dbg!(&rq);
    if rq.is_dir{
        ch_dircerater_handler(state.clone(), session.clone(), rq.clone()).await?;
    }else {
        match state.store.get_entry(PathBuf::from(rq.filename.clone()),rq.is_dir,rq.zone.clone()).await {
            Ok(f) =>{
                state.store.update_files_info_string(f.id,"creator".to_string(),rq.creator.clone(),rq.zone.clone())
                .await
                .map_err(|e|error_to_rejection(e))?;
                
                state.store.update_files_info_string(f.id,"last_modifier".to_string(),rq.creator.clone(),rq.zone.clone())
                .await
                .map_err(|e|error_to_rejection(e))?;
            },
            Err(_)=> return Err(error_to_rejection(FVErrors::IOError("File not exists".to_string()))),
        }
        
        
    }
    
    state.store.log_sucess(&session, UserAction::Chown, &rq.filename, &Some(rq.creator)).await?;
    Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"message": "Change creator Success"})),
            StatusCode::OK,
        ))
}

pub async fn ch_dircerater_handler(state:AppState,session:Session,rq:RqFileChown)->Result<impl Reply,Rejection>{
    state.store.chdir_creator(PathBuf::from(rq.filename.clone()),rq.creator.clone(),rq.zone.clone())
        .await
        .map_err(|e|error_to_rejection(e))?;
    Ok(warp::reply::with_status(
            "Change dir creator Success",
            StatusCode::OK,
    ))
}

pub async fn log_handler(state:AppState,session:Session,query:HashMap<String,String>)->Result<impl Reply,Rejection>{
    let len= query.get("len").unwrap_or(&"100".to_string()).parse::<i32>().map_err(|e|error_to_rejection(FVErrors::NotFound))?;
    let events = state.store.list_log(len).await.map_err(error_to_rejection)?;
    Ok(warp::reply::json(&events))
}

pub async fn verify_handlers(state:AppState,query:HashMap<String,String>)->Result<impl Reply,Rejection> {
    let token = query.get("token").unwrap();
    match auth::verify_token(dbg!(token)){
        Ok(session) => {
            dbg!(&session);
            if session.exp.gt(&Utc::now()) && session.nbf.le(&Utc::now()) {
                Ok(warp::reply::with_status("Valid Token", warp::http::StatusCode::OK))}
            else{
                Ok(warp::reply::with_status("Invalid Token", warp::http::StatusCode::UNAUTHORIZED))
            }
        },
        
        Err(_) => Ok(warp::reply::with_status("Invalid Token", warp::http::StatusCode::UNAUTHORIZED))
    }
}


pub async fn zone_list_handler(state:AppState,session:Session)->Result<impl Reply,Rejection>{
    // Ok(warp::reply::with_status("Valid Token", warp::http::StatusCode::OK))
    let a = state.store.all_zones().await.map_err(|e|error_to_rejection(e))?;
    Ok(warp::reply::json(&a))
}

pub async fn zone_rename_handler(state:AppState,session:Session,rq:RqZoneRename)->Result<impl Reply,Rejection>{
    let path = PathBuf::from("").to_sys_path(&state.config.sys_disk_dir, &rq.name)
    .map_err(|e|error_to_rejection(FVErrors::NotFound))?;
    
    let new_path = PathBuf::from("").to_sys_path(&state.config.sys_disk_dir, &rq.new_name)
    .map_err(|e|error_to_rejection(FVErrors::NotFound))?;
    
    dbg!(&path,&new_path);
    let c = tokio::fs::rename(path, new_path).await.map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;
    dbg!(c);
    state.store.update_zone_name(rq.name, rq.new_name).await.map_err(error_to_rejection)?;
    Ok(warp::reply::with_status("Zone Rename Success", warp::http::StatusCode::OK))
}

pub async fn zone_create_handler(state:AppState,session:Session,rq:RqZoneCreate)->Result<impl Reply,Rejection>{
    state.store.insert_zone(FileZone{id:0,name:rq.name.clone(),lords:rq.lords}).await.map_err(error_to_rejection)?;

    tokio::fs::create_dir_all(PathBuf::from("").to_sys_path(&state.config.sys_disk_dir, &rq.name).unwrap())
    .await
    .map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())))?;
    Ok(warp::reply::with_status("Zone Create Success", warp::http::StatusCode::OK))
}

pub async fn zone_newlords_handler(state:AppState,session:Session,rq:RqZoneNewLords)->Result<impl Reply,Rejection>{
    state.store.update_zone_lords(rq.name,rq.lords).await.map_err(error_to_rejection)?;
    Ok(warp::reply::with_status("Zone Lords Update Success", warp::http::StatusCode::OK))
}

pub async  fn zone_delete_handler(state:AppState,session:Session,rq:HashMap<String,String>)->Result<impl Reply,Rejection>{
    let default = &String::new();
    let zone = rq.get("zone").unwrap_or(default);

    dbg!(&rq);
    let a = tokio::fs::remove_dir_all(PathBuf::from("").to_sys_path(&state.config.sys_disk_dir, zone).unwrap()).await
    .map_err(|e|error_to_rejection(FVErrors::IOError(e.to_string())));
    dbg!(&a);
   
    state.store.del_zone_by_name(zone.to_string()).await?;
    Ok(warp::reply::with_status("Zone Lords Delete Success", warp::http::StatusCode::OK))
}


pub async fn zone_tree_handler(state:AppState,session:Session,query: HashMap<String,String>)->Result<impl Reply,Rejection>{
    // dbg!(&query,&session);
    let zone= query.get("zone").unwrap().to_string();
    let a = state.store.tree_zones(zone).await.map_err(error_to_rejection)?;
    // dbg!(&a);
    Ok(warp::reply::json(&a))
}


pub async fn zone_size_handler(state:AppState,session:Session)->Result<impl Reply,Rejection>{
    let size = files::folder_size(&state.config.sys_disk_dir);
    if let  Some(v) = files::disk_capacity(&state.config.sys_disk_dir.to_absolute().unwrap_or_default()){
        Ok(warp::reply::json(&vec![size,v.0,v.1,v.2]))
    }else{
        Err(error_to_rejection(FVErrors::NotFound))
    }
    
}
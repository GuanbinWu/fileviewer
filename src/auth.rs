use std::{clone, future, path::PathBuf};
use chrono::{DateTime, Days, Utc};

use clap::{ValueHint::Username, builder::Str};
use serde::{Deserialize,Serialize};
use rand::{self, Rng};
// use tower_http::classify::GrpcCode::Ok;
use warp::{Filter, filters::path::Exact, reject::Rejection, reply::Reply};
use std::sync::Arc;
use crate::{config::Config, database::Store, errors::{AuthError::NoSuchUser, *}};
// use sqlx::encode::IsNull::No;
#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct Account{
    pub username:String,
    pub password:String,
}

impl Account {
    fn new(username:String,passwd:String)->Self{
        Account { username, password: passwd }
    }
    fn defualt()->Self{
        Account {username: "Default".to_string(), password: "Default".to_string() }
    }
}

#[derive(Debug,Clone)]
pub struct AccountForStore{
    pub username:String,
    pub hashed:String
}


#[derive(Debug,Serialize,Deserialize)]
pub enum Attempt {
    Try,
    Success,
}
#[derive(Debug,Serialize,Deserialize)]
pub struct  Event {
    pub user:String,
    pub action:String,  //From UserAction
    pub time:DateTime<Utc>,
    pub status:String, //From Attempt
    pub filepath:String,
    pub args:String,
}

#[derive(Debug,Serialize,Deserialize)]
pub enum UserAction {
    UpdatePassword,
    Regist,
    DeleteAccount,
    DeleteFile,
    Login,
    Logout,
    Mkdir,
    Rmdir,
    Upload,
    Download,
    List,
    ViewDetail,
    Preview,
    Rename,
    CpFile,
    Chown,
    CreateZone,
    RenameZone,
    UpdateZoneLords,

}

impl std::fmt::Display for UserAction{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug = format!("{:?}", self);
        let name = debug.split("::").last().unwrap_or(&debug);
        write!(f, "{}", name)
    }
}


impl std::fmt::Display for Attempt{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug = format!("{:?}", self);
        let name = debug.split("::").last().unwrap_or(&debug);
        write!(f, "{}", name)
    }
}


pub async fn regist(account:Account,conf:Arc<Config>,store:Arc<Store>)->Result<(),FVErrors>{

    let Account { username: name, password } =account;

    let _ = is_valid_user_name(&name, conf.clone()).await?;

    let _=is_vaild_pwd(&password, conf).await?;

    match store.query_username(&name).await {
        Err(FVErrors::AuthError(AuthError::NoSuchUser))=>{
            dbg!("数据库无重名！");
            let hashed=hash_passwd(&password);
            dbg!("密码已被加密为",&hashed);

            store.new_account(&name, hashed).await?;

            Ok(())
        },
        _=>Err(FVErrors::NotFound)
    }    
}

async fn verify_current_user(account:Account,conf:Arc<Config>,store:Arc<Store>)->Result<(),FVErrors>{
    let Account { username: name, password }=account;
    //检索用户密码hash值
    // let hashed=String::new();
    let user = store.query_username(&name).await?;
    println!("{:?}",user);
    let hashed=user.hashed;
    match verify_passwd(&password, hashed){
        Ok(_)=>{println!("密码验证成功");Ok(())},
        Err(e)=>Err(e)
    }
   
}

pub async fn update_pwd(account:Account,new_pwd:String,conf:Arc<Config>,store:Arc<Store>)->Result<(),FVErrors>{
    let Account { username: name, password }=account.clone();

    let v=verify_current_user(account, conf.clone(),store.clone()).await?;
    
    //新密码是否合法
    let v = is_vaild_pwd(&new_pwd, conf).await?;

    //加密新密码
    let hashed=hash_passwd(&new_pwd);

    //数据库更改密码
    store.new_pwd(&name, hashed.clone()).await?;

    //日志记录
    store.append_log(Event {
        user: name.clone(),
        action: UserAction::UpdatePassword.to_string(),
        time: Utc::now(),
        status: Attempt::Success.to_string(),
        filepath: "login/regist".to_string(),
        args: String::new(),
    }).await?;
    Ok(())
}

pub async fn delete_account(account:Account,conf:Arc<Config>,store:Arc<Store>)->Result<(),FVErrors>{
    let Account { username: name, password }=account.clone();


    //查询原密码是否正确
    let _=verify_current_user(account, conf.clone(),store.clone()).await?;
    //数据库删除用户
    store.del_account(name.clone()).await?;

    //日志记录
    store.append_log(Event {
        user: name.clone(),
        action: UserAction::DeleteAccount.to_string(),
        time: Utc::now(),
        status: Attempt::Success.to_string(),
        filepath: "login/regist".to_string(),
        args: String::new(),
    }).await?;
    Ok(())
}

pub async fn login(account:Account,conf:Arc<Config>,store:Arc<Store>)->Result<(),FVErrors>{
    //验证当前用户
    let Account { username: name, password }=account.clone();

    let _=verify_current_user(account, conf,store.clone()).await?;

    //日志记录
    // let Account { name, password }=account.clone();
    store.append_log(Event {
        user: name.clone(),
        action: UserAction::Login.to_string(),
        time: Utc::now(),
        status: Attempt::Success.to_string(),
        filepath: "login".to_string(),
        args: String::new(),
    }).await?;
    Ok(())
    
}

pub async fn record_logout(conf:Arc<Config>,store:Arc<Store>,session:Session)->Result<(),FVErrors>{
    let name =session.username;


     store.append_log(Event {
        user: name.clone(),
        action: UserAction::Logout.to_string(),
        time: Utc::now(),
        status: Attempt::Success.to_string(),
        filepath: "login/logout".to_string(),
        args: String::new(),
    }).await?;
    Ok(())    
}




async fn is_vaild_pwd(pwd:&str,conf:Arc<Config>)->Result<(),FVErrors>{
    if pwd.len()<conf.min_pwd_len as usize{
        println!("Password Tooshort");
        return Err(FVErrors::from(AuthError::InvalidPassword))
    }
    
    
    
    let mut tmp=conf.config_dir.clone();
    tmp.push("bad_pwd.txt");
    let bad_pwd_list:Vec<String>= tokio::fs::read_to_string(tmp)
    .await
    .map_err(|e|  FVErrors::IOError("Fail Reading Files".to_string()))?
    .lines()
    .map(|s|s.to_string())
    .filter(|s| !s.is_empty())
    .collect();    
    
    
    if bad_pwd_list.contains(&pwd.to_string()){
        println!("Too Simple Password");
        return Err(FVErrors::from(AuthError::InvalidPassword))
    }else {
        println!("Valid Password");
        return Ok(())
    }
}

fn hash_passwd(passwd:&str)->String{
    let salt=rand::thread_rng().r#gen::<[u8;32]>();
    let conf=argon2::Config::default();
    let hashed=argon2::hash_encoded(passwd.as_bytes(), &salt, &conf).unwrap();
    return hashed
}

fn verify_passwd(passwd:&str,hashed:String)->Result<(), FVErrors>{
    // println!("{}",hashed.clone());
    println!("{:?}",argon2::verify_encoded(&hashed, passwd.as_bytes()));
    if !argon2::verify_encoded(&hashed, passwd.as_bytes()).unwrap(){
        return Err(FVErrors::from(AuthError::DecodeError))
    }
    Ok(())
}

async fn is_valid_user_name(username:&str,conf:Arc<Config>)->Result<(), FVErrors>{
    if username.is_empty() {
        println!("不能为空");
        return Err(FVErrors::from(AuthError::InvalidUsername));
    }
    //字符个数长度限制
    if username.chars().count() > conf.max_username_len as usize {
        println!("字符个数长度限制");
        return Err(FVErrors::from(AuthError::InvalidUsername));
    }
    //仅允许字符与下划线
    if !username.chars().all(|c| c.is_alphabetic() || c == '_') {
        println!("仅允许字符与下划线");
        return Err(FVErrors::from(AuthError::InvalidUsername));
    }
    let mut tmp=conf.config_dir.clone();
    tmp.push("username_whitelist.txt");
    let username_whitelist:Vec<String>= tokio::fs::read_to_string(tmp)
    .await
    .map_err(|e|  FVErrors::IOError("Fail Reading Files".to_string()))?
    .lines()
    .map(|s|s.to_string())
    .filter(|s| !s.is_empty())
    .collect();
    
    if username_whitelist.contains(&username.to_string()){
        Ok(())
    }else {
        println!("不在白名单里");
        return Err(FVErrors::from(AuthError::InvalidUsername));
    }
    
}

//管理员功能
pub async fn reset_pwd(opertar:Account,target:Account,conf:Arc<Config>,store:Arc<Store>)->Result<(),FVErrors>{
    
    //验证操作人员
    let _=verify_current_user(opertar, conf.clone(),store.clone()).await?;
    
    //获取默认密码
    let new_pwd=conf.default_pwd.clone();
    
    //修改数据库

    //日志记录操作员行为

    
    Ok(())
}


//Session
pub fn create_token(username:&str,days:i64,nbf:DateTime<Utc>)->String{
    let now_time=Utc::now();
    let dt=now_time+chrono::Duration::days(days);

    // let state=serde_json::to_string(username).expect("Fail to serialize");
    // local_paseto(&state, None, "Random".as_bytes()).expect("Fail to create token")
    paseto::tokens::PasetoBuilder::new()
    .set_encryption_key(&Vec::from("0123456789abcdef0123456789abcdef".as_bytes()))
    .set_expiration(&dt)
    .set_not_before(&nbf)
    .set_claim("username", serde_json::json!(username))
    .build()
    .expect("Fail to construct paseto token builder")
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Session{
    pub exp:DateTime<Utc>,
    pub username:String,
    pub nbf:DateTime<Utc>,
}

impl Default for Session {
    fn default() -> Self {
    Session{exp:Utc::now(),username:String::new(),nbf:Utc::now(),
    }
}
}

pub fn verify_token(token:&str)->Result<Session,String>{
    if token.is_empty() {
        return Err("Token is empty".to_string());
    }
    let token=paseto::tokens::validate_local_token(
        &token,
         None, 
         "0123456789abcdef0123456789abcdef".as_bytes(), 
         &paseto::tokens::TimeBackend::Chrono)
         .map_err(|_|"Cannot DecryptToken".to_string())?;
    // println!("{}",token.clone());
    serde_json::from_value::<Session>(token).map_err(|_|"Cannot DecryptToken".to_string())
}


pub fn auth() -> impl Filter<Extract = (Session,), Error = Rejection> + Clone {
    warp::header::<String>("Authorization")
        .and_then(|token: String| {
            // println!("{}",token);
            async move {
                match verify_token(&token) {
                    Ok(session) => {
                        // println!("Auth Ok");
                        Ok(session)},
                    Err(_) => {
                        // println!("Auth Fail");
                        // Err(warp::reject::)
                        Err(warp::reject::reject())},
                }
            }
        })
}



#[cfg(test)]
mod tests{
use super::*;

    #[test]
    fn verifyToken(){
        let token = create_token("Wuguanbin", 1,Utc::now());
        let session = verify_token(&token);
        println!("{:?}",session);
    }
}
use serde::{Deserialize, Serialize,};
use std::collections::HashSet;
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FileRequestA{
    pub zone:String,
    pub dir:String}


#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FileRequestB{
    pub zone: String,
    pub is_dir:bool,
    pub filename: String,
    pub md5: String,
    pub bytes:String}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FileRequestC{
    pub zone:String,
    pub is_dir:bool,
    pub filename: String,
    pub new_filename: String,}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FileRequestD{
    pub zone:String,
    pub is_dir:bool,
    pub filename: String}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FileRequestE{
    pub zone:String,
    pub is_dir:bool,
    pub filename: String,
    pub creator:String}

// #[derive(Debug,Clone,Serialize,Deserialize)]
// pub struct AccountRequestA{
//     pub username:String,
//     pub password:String}

type AccountRequestA = crate::auth::Account;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AccountRequestB{
    pub username:String,
    pub password:String,
    pub newpwd:String,}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ZoneRequestA{
    pub name:String,
    pub lords:HashSet<String>}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ZoneRequestB{
    pub name:String,
    pub new_name:String}

pub type RqFileList = FileRequestA;
pub type RqFileUpload  =FileRequestB;
pub type RqFileRename = FileRequestC;
pub type RqFileCopy = FileRequestC;
pub type RqFileDownload = FileRequestD;
pub type RqFileDelete = FileRequestD;
pub type RqFileChown = FileRequestE;
pub type RqAccountLogin = AccountRequestA;
pub type RqAccountRegist = AccountRequestA;
pub type RqAccountNewPwd = AccountRequestB;
pub type RqAccountDelete = AccountRequestA;
pub type RqZoneRename  =ZoneRequestB;
pub type RqZoneCreate = ZoneRequestA;
pub type RqZoneNewLords= ZoneRequestA;

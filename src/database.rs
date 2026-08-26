

use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::Config;
use sqlx::any::AnyArguments;
// use sqlx::encode::IsNull::No;
use sqlx::postgres::{PgPoolOptions,PgPool,PgRow};
use warp::reject::{Reject, Rejection};
use std::collections::{HashSet,HashMap,VecDeque};
use sqlx::Row;

use crate::auth::{Account,AccountForStore, Event, Session, UserAction,Attempt};
// use crate::errors::FVErrors::DbError;
use crate::errors::{AuthError, DbError, FVErrors, error_to_rejection};
use crate::files::{FileEntry, FileZone,PathBehavior, collect_dirs, collect_files, collect_zones};
use chrono::{DateTime,Utc};
#[derive(Debug,Clone)]
pub struct Store{
    pub connection:PgPool
}

//用户表:accounts,字段:username,hashed
//文件表:files,字段[id: u64,parent_id: Option<u64>,name: PathBuf,is_directory: bool,size: i64,content_type: String,md5: Option<String>,created_at: DateTime<Utc>,modified_at: DateTime<Utc>,creator:String,last_modifier:String,zone:String],
//log表:events,字段:[user:String,action:UserAction,time:DateTime<Utc>,state:Attempt,files:PathBuf,args:Vec<String>]


impl Store {
    pub async fn new(db_url:&str) ->Self{
        let db_pool =match PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url).await{
            Ok(pool) =>pool,
            Err(e)=>panic!("Cannot connet to database:{db_url}!\nError:{e}")
        };
        println!("Connect to Database Successfully!");
        Store{connection:db_pool}
        
    }
    
    pub async fn sync_files(&self, config: Arc<Config>) -> Result<(), FVErrors> {
        // 1. 从数据库加载所有目录和文件
        let zones: HashSet<String> = self.all_zones().await.unwrap().iter().map(|z|z.name.clone()).collect::<HashSet<String>>();

        for z in zones{
            self.sync_files_by_zone(config.clone(), z).await.unwrap();
        }

        self.sync_parent_name().await?;
        
        Ok(())

    }
    

    async fn sync_files_by_zone(&self,config:Arc<Config>,zone:String)->Result<(),FVErrors>{
        let dbdirs:Vec<PathBuf>=match  sqlx::query("SELECT name FROM files WHERE is_directory = true AND zone = $1;")
        .bind(zone.clone())
        .map(|row: PgRow|
            {let v:PathBuf=serde_json::from_value(row.get("name")).unwrap();v}
        )
        .fetch_all(&self.connection)
        .await
        {
            Ok(v)=>v,
            Err(e)=>{println!("?");return Err(FVErrors::from(DbError(e)));}
        };
        // println!("{:?}",&dbdirs);
        

        let dbfiles:Vec<PathBuf>=match  sqlx::query("SELECT name FROM files WHERE is_directory = false AND zone =$1;")
        .bind(zone.clone())
        .map(|row: PgRow|
            {let v:PathBuf=serde_json::from_value(row.get("name")).unwrap();
            v}
        )
        .fetch_all(&self.connection)
        .await
        {
            Ok(v)=>v,
            Err(e)=>{println!("1");return Err(FVErrors::from(DbError(e)));}
        };


        let disk_dirs = collect_dirs(config.clone(),&zone).await
        .map(|v|
        v.into_iter().map(|f|f.to_web_path(&config.sys_disk_dir, &zone).unwrap()).collect::<Vec<PathBuf>>()
        )?;

        let disk_files = collect_files(config.clone(),&zone).await
        .map(|v|
        v.into_iter().map(|f|f.to_web_path(&config.sys_disk_dir, &zone).unwrap()).collect::<Vec<PathBuf>>()
        )?;

        let set_db_dir:HashSet<PathBuf>=dbdirs.into_iter().collect();
        let set_db_files:HashSet<PathBuf>=dbfiles.into_iter().collect();
        let set_disk_dir:HashSet<PathBuf>=disk_dirs.into_iter().collect();
        let set_disk_files:HashSet<PathBuf>=disk_files.into_iter().collect();

        let to_delete_dir: Vec<_> = set_db_dir
        .difference(&set_disk_dir)
        .cloned()
        .collect();

        let to_delete_file: Vec<_> = set_db_files
        .difference(&set_disk_files)
        .cloned()
        .collect();

        let to_add_dir: Vec<_> = set_disk_dir
        .difference(&set_db_dir)
        .cloned()
        .collect();

        let to_add_files: Vec<_> = set_disk_files
        .difference(&set_db_files)
        .cloned()
        .collect();

        for i in to_delete_file.into_iter(){self.delete_files(i,zone.clone()).await?};
        for i in to_delete_dir.into_iter(){self.delete_files(i,zone.clone()).await?};
        
        for i in to_add_dir.into_iter(){
            let file = FileEntry { id: 0, 
                name: i.clone(), 
                parent_name: i.get_parent().unwrap(), 
                is_directory: true, 
                size: 0, 
                content_type: "Dir".to_string(), 
                md5: None, 
                created_at: Utc::now(), 
                modified_at: Utc::now(), 
                creator: zone.clone(), 
                last_modifier: zone.clone(),
                zone:zone.clone()
            };
            self.upload_files(file).await?;
        }

        
        for i in to_add_files.into_iter(){
            let disk_path = i.to_sys_path(&config.sys_disk_dir, &zone).unwrap();
            let bytes=tokio::fs::read(disk_path).await.map_err(|e|FVErrors::IOError(e.to_string()))?;

            let size = bytes.len() as i64;
            let content_type = mime_guess::from_path(i.clone())
                .first_or_octet_stream()
                .to_string();
            let md5 = {
                let digest = md5::compute(&bytes);
                format!("{:x}", digest)
            };
            let file = FileEntry { id: 0, 
                name: i.clone(), 
                parent_name: i.get_parent().unwrap(), 
                is_directory: false, 
                size: size, 
                content_type: content_type, 
                md5: Some(md5), 
                created_at: Utc::now(), 
                modified_at: Utc::now(), 
                creator: zone.clone(), 
                last_modifier: zone.clone(),
                zone:zone.clone()
            };
            self.upload_files(file).await?;
        }
        
        Ok(())
    }

    async fn sync_parent_name(&self)->Result<(),FVErrors>{
        let dbdirs:Vec<(i64,PathBuf,String)>=match  sqlx::query("SELECT id,name,zone FROM files;")
        .map(|row: PgRow|{
            let id = row.get("id");
            let name:PathBuf=serde_json::from_value(row.get("name")).unwrap();
            let zone:String = row.get("zone");
            (id,name.get_parent().unwrap(),zone)
        }
        )
        .fetch_all(&self.connection)
        .await
        {
            Ok(v)=>v,
            Err(e)=>{println!("?");return Err(FVErrors::from(DbError(e)));}
        };
        // println!("{:?}",&dbdirs);
        for (id,pname,zone) in dbdirs{
        self.update_files_info_pthbuf(id,  "parent_name".to_string(), pname,zone.clone())
            .await?;}
        Ok(())
    }
    //TABLE accounts
    pub async fn query_username(&self,name:&str)->Result<AccountForStore,FVErrors>{

        match sqlx::query("SELECT * from accounts WHERE username = $1")
        .bind(name)
        .map(|row:PgRow|
        AccountForStore{
            username:row.get("username"),
            hashed:row.get("hashed")
        })
        .fetch_all(&self.connection)
        .await
        {
            Ok(account) if account.is_empty() =>Err(FVErrors::from(AuthError::NoSuchUser)),
            Ok(account) => Ok(account[0].clone()),
            Err(e)=>Err(FVErrors::from(DbError(e)))
        }
    }

    pub async fn new_account(&self,name:&str,hashed_pwd:String)->Result<AccountForStore,FVErrors>{
        println!("数据库尝试新增用户！");
        match self.query_username(name).await
        {
            Ok(_) => Err(FVErrors::from(AuthError::UsernameUsed)),
            Err(_)=>
            {   
                let r=
                match sqlx::query("INSERT INTO accounts (username,hashed) VALUES ($1,$2) RETURNING username,hashed ")
                .bind(name)
                .bind(hashed_pwd)
                .map(|row:PgRow|AccountForStore{username:row.get("username"),hashed:row.get("hashed")})
                .fetch_one(&self.connection)
                .await
                {
                Ok(account)=>{println!("数据库新增用户成功！");Ok(account)},
                Err(e)=>Err(FVErrors::from(DbError(e)))
                };
                r
            }
        }
    }
    
    pub async fn new_pwd(&self,name:&str,hashed:String)->Result<AccountForStore,FVErrors>{
        match self.query_username(name).await
        {
            Ok(_) =>{
            match sqlx::query("UPDATE accounts
            SET username=$1,hashed=$2
            WHERE username =$1
            RETURNING username,hashed")
            .bind(name.to_string())
            .bind(hashed)
            .map(|row:PgRow|AccountForStore{username:row.get("username"),hashed:row.get("hashed")})
            .fetch_one(&self.connection)
            .await
            {
                Ok(account)=>Ok(account),
                Err(e)=>Err(FVErrors::from(DbError(e)))
            }
        },
        Err(e) => Err(e)
        }
    }

    pub async fn del_account(&self,name:String)->Result<(),FVErrors>{

        match self.query_username(&name).await
        {
            Ok(_)=>{
            println!("存在用户");
            match sqlx::query("DELETE FROM accounts WHERE username=$1")
            .bind(name)
            .execute(&self.connection)
            .await{
                Ok(_) =>Ok(()),
                Err(e)=>Err(FVErrors::from(DbError(e)))
            }},
            Err(e)=> Err(e)
        }
    }
    

    pub async fn list_account(&self)->Result<Vec<String>,FVErrors>{
        match sqlx::query("SELECT * from accounts")
        .map(|row:PgRow| 
            {let name:String = row.get("username");
            name        
        })
        .fetch_all(&self.connection)
        .await{
            Ok(v)=> Ok(v),
            Err(e)=>Err(FVErrors::DbError(DbError(e)))
        }
        
    }
    //TABLE files;
    pub async fn list_dir (&self,pname:PathBuf,zone:String)->Result<Vec<FileEntry>,FVErrors>{
        // println!("{:?}",&pname);
        let sql = "SELECT * FROM files WHERE parent_name = $1 AND zone = $2";
        match sqlx::query(sql)
        .bind(serde_json::json!(pname))
        .bind(zone)
        .map(|row:PgRow|{
            FileEntry{
            id: row.get("id"),
            name: serde_json::from_value(row.get("name")).unwrap(),
            parent_name:serde_json::from_value(row.get("parent_name")).unwrap(),
            is_directory: row.get("is_directory"),
            size: row.get("size"),
            content_type: row.get("content_type"),
            md5: row.get("md5"),
            created_at: row.get("created_at"),
            modified_at: row.get("modified_at"),
            creator:row.get("creator"),
            last_modifier:row.get("last_modifier"),
            zone:row.get("zone")
        }

        }

        )
        .fetch_all(&self.connection)
        .await{
            Ok(v)=>Ok(v),
            Err(e)=>Err(FVErrors::from(DbError(e)))
        }
    }
    
    // pub async fn get_dir_entry(&self,name:PathBuf,zone:String)->Result<FileEntry,FVErrors>{
    //     match sqlx::query(
    //         "SELECT * FROM files WHERE name = $1 AND is_directory = true AND zone=$2"
    //     )
    //     .bind(serde_json::json!(name))
    //     .bind(zone)
    //     .fetch_one(&self.connection)
    //     .await
    //     {
    //         Ok(row) =>{
    //             Ok(            
    //         FileEntry{
    //         id: row.get("id"),
    //         name: serde_json::from_value(row.get("name")).unwrap(),
    //         parent_name:serde_json::from_value(row.get("parent_name")).unwrap(),
    //         is_directory: row.get("is_directory"),
    //         size: row.get("size"),
    //         content_type: row.get("content_type"),
    //         md5: row.get("md5"),
    //         created_at: row.get("created_at"),
    //         modified_at: row.get("modified_at"),
    //         creator:row.get("creator"),
    //         last_modifier:row.get("last_modifier"),
    //         zone:row.get("zone")})
    //         },
    //         _ => Err(FVErrors::NotFound)
    //     }
    // }
    
    pub async fn get_entry(&self,name:PathBuf,is_dir:bool,zone:String)->Result<FileEntry,FVErrors>{
        match sqlx::query(
            "SELECT * FROM files WHERE name = $1 AND is_directory = $2 AND zone=$3"
        )
        .bind(serde_json::json!(name))
        .bind(is_dir)
        .bind(zone)
        .fetch_one(&self.connection)
        .await
        {
            Ok(row) =>{
                Ok(            
            FileEntry{
            id: row.get("id"),
            name: serde_json::from_value(row.get("name")).unwrap(),
            parent_name:serde_json::from_value(row.get("parent_name")).unwrap(),
            is_directory: row.get("is_directory"),
            size: row.get("size"),
            content_type: row.get("content_type"),
            md5: row.get("md5"),
            created_at: row.get("created_at"),
            modified_at: row.get("modified_at"),
            creator:row.get("creator"),
            last_modifier:row.get("last_modifier"),
            zone:row.get("zone")})
            },
            _ => Err(FVErrors::NotFound)
        }


    }

    pub async fn upload_files(&self,file:FileEntry)->Result<FileEntry,FVErrors>{
        println!("Uploading files {:?}",&file);

        let sql = "INSERT INTO files (name,parent_name, is_directory, size, content_type, md5, created_at, modified_at, creator, last_modifier,zone)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,$11)
        RETURNING *";
        
        match sqlx::query(sql)
        .bind(serde_json::json!(file.name))
        .bind(serde_json::json!(file.parent_name))
        .bind(file.is_directory)
        .bind(file.size)
        .bind(file.content_type)
        .bind(file.md5)
        .bind(file.created_at)
        .bind(file.modified_at)
        .bind(file.creator)
        .bind(file.last_modifier)
        .bind(file.zone)
        .map(|row:PgRow|{
            FileEntry{
            id: row.get("id"),
            name: serde_json::from_value(row.get("name")).unwrap(),
            parent_name: serde_json::from_value(row.get("parent_name")).unwrap(),
            is_directory: row.get("is_directory"),
            size: row.get("size"),
            content_type: row.get("content_type"),
            md5: row.get("md5"),
            created_at: row.get("created_at"),
            modified_at: row.get("modified_at"),
            creator:row.get("creator"),
            last_modifier:row.get("last_modifier"),
            zone:row.get("zone")}
        })
        .fetch_one(&self.connection)
        .await{
             Ok(v)=>Ok(v),
            Err(e)=>Err(FVErrors::from(DbError(e)))
        }
    }
    
    pub async fn delete_files(&self,name:PathBuf,zone:String)->Result<(),FVErrors>{
        let sql = "DELETE FROM files WHERE name = $1 AND zone = $2";
        sqlx::query(sql)
        .bind(serde_json::json!(name))
        .bind(zone)
        .execute(&self.connection)
        .await
        .map_err(|e| FVErrors::from(DbError(e)))?;
    Ok(())
    }

    //返回下级所有文件与目录
    pub async fn list_dir_all(&self,dir:PathBuf,zone:String)->Result<Vec<FileEntry>,FVErrors>{
        let mut queue = VecDeque::new();
        let mut result:Vec<FileEntry> = Vec::new();

        queue.push_back(dir.clone());
        while let Some(current_dir) = queue.pop_front() {

            let entries:Vec<FileEntry> = self.list_dir(current_dir,zone.clone()).await?;

            for entry in entries {                
                result.push(entry.clone());
                if entry.is_directory {
                    queue.push_back(entry.name.clone());
                }
            }
        }
        Ok(result)
    }

    pub async fn delete_dir_all(&self,dir:PathBuf,zone:String)->Result<(),FVErrors>{
        // let mut tmp =Vec::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        result.push(dir.clone());
        queue.push_back(dir.clone());
        while let Some(current_dir) = queue.pop_front() {

            let entries:Vec<(PathBuf, bool)> = self.list_dir(current_dir,zone.clone())
                .await?
                .into_iter()
                .map(|f|(f.name,f.is_directory))
                .collect();

            for entry in entries {                
                result.push(entry.0.clone());
                if entry.1 {
                    queue.push_back(entry.0);
                }
            }
        }
        for i in result{
            self.delete_files(i,zone.clone()).await?;
        }
        Ok(())

    }


    pub async fn rename_dir(&self,dir:PathBuf,new_dir:PathBuf,zone:String)->Result<(),FVErrors>{
        let all_files:Vec<FileEntry> =
        sqlx::query("SELECT * FROM files WHERE zone = $1;")
        .bind(zone.clone())
        .map(|row:PgRow|{
            FileEntry{
            id: row.get("id"),
            name: serde_json::from_value(row.get("name")).unwrap(),
            parent_name:serde_json::from_value(row.get("parent_name")).unwrap(),
            is_directory: row.get("is_directory"),
            size: row.get("size"),
            content_type: row.get("content_type"),
            md5: row.get("md5"),
            created_at: row.get("created_at"),
            modified_at: row.get("modified_at"),
            creator:row.get("creator"),
            last_modifier:row.get("last_modifier"),
            zone:row.get("zone")}
        })
        .fetch_all(&self.connection)
        .await
        .map_err(|e| FVErrors::DbError(DbError(e)))?;
        

        let names:Vec<(i64,PathBuf)>=all_files.clone().into_iter()
        .filter(|f| f.name.has_path_prefix(&dir))
        .map(|f| (f.id,f.name))
        .collect();
        dbg!(&names);
        
        for (id,name) in names{
           let new_name = name.replace_prefix(&dir, &new_dir).unwrap();
           let new_pname=new_name.get_parent().unwrap();
           self.update_files_info_pthbuf(id,  "parent_name".to_string(),new_pname,zone.clone()).await?;
           self.update_files_info_pthbuf(id,  "name".to_string(),new_name,zone.clone()).await?;

        }

    Ok(())
    }
    

    pub async fn update_files_info_string(&self,id:i64,col:String,value:String,zone:String)->Result<(),FVErrors>{
        let sql = format!("UPDATE files SET {} = $1 WHERE id = $2 AND zone = $3 ",col);
        sqlx::query(&sql)
        .bind(value)
        .bind(id)
        .bind(zone)
        .execute(&self.connection)
        .await
        .map_err(|e| {println!("{:?}",e);FVErrors::from(DbError(e))})?;

    Ok(())
    }

    pub async fn update_files_info_pthbuf(&self,id:i64,target_col:String,value:PathBuf,zone:String)->Result<(),FVErrors>{
        let sql = format!("UPDATE files SET {} = $1 WHERE id = $2 AND zone = $3",target_col);
        sqlx::query(&sql)
        .bind(serde_json::json!(value))
        .bind(id)
        .bind(zone)
        .execute(&self.connection)
        .await
        .map_err(|e| {println!("{:?}",e);FVErrors::from(DbError(e))})?;
    Ok(())
    }

    pub async fn update_files_info_time(&self,id:i64,col:String,value:DateTime<Utc>,zone:String)->Result<(),FVErrors>{
        let sql = format!("UPDATE files SET {} = $2 WHERE id = $1 AND zone = $3",col);

        sqlx::query(&sql)
        .bind(id)
        .bind(value)
        .bind(zone)
        .fetch_one(&self.connection)
        .await
        .map_err(|e| FVErrors::from(DbError(e)))?;

    Ok(())
    }
    
    pub async fn chdir_creator(&self,dir:PathBuf,value:String,zone:String)->Result<(),FVErrors>{
        let all_files:Vec<FileEntry> = 
        sqlx::query("SELECT * FROM files WHERE zone =$1;")
        .bind(zone.clone())
        .map(|row:PgRow|{
            FileEntry{
            id: row.get("id"),
            name: serde_json::from_value(row.get("name")).unwrap(),
            parent_name:serde_json::from_value(row.get("parent_name")).unwrap(),
            is_directory: row.get("is_directory"),
            size: row.get("size"),
            content_type: row.get("content_type"),
            md5: row.get("md5"),
            created_at: row.get("created_at"),
            modified_at: row.get("modified_at"),
            creator:row.get("creator"),
            last_modifier:row.get("last_modifier"),
            zone:row.get("zone")
            }
        })
        .fetch_all(&self.connection)
        .await
        .map_err(|e| FVErrors::DbError(DbError(e)))?;
        
        let idxs:Vec<i64>=all_files.clone().into_iter()
        .filter(|f| f.name.has_path_prefix(&dir))
        .map(|f| f.id)
        .collect();

        for id in idxs{
            self.update_files_info_string(id,"creator".to_string(),value.clone(),zone.clone()).await?;
            self.update_files_info_string(id,"last_modifier".to_string(),value.clone(),zone.clone()).await?;
        }
    Ok(())
    }
    
    //TABLE log
    pub async fn append_log(&self,event:Event)->Result<(),FVErrors>{
        // let json_event=serde_json::json!(event);
        match sqlx::query("INSERT INTO events (username, action, time, status, filepath, args) 
        VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(&event.user)
        .bind(&event.action)
        .bind(&event.time)
        .bind(&event.status)
        .bind(&event.filepath)
        .bind(&event.args)
        .execute(&self.connection)
        .await{
            Ok(_)=>Ok(()),
            Err(e)=>Err(FVErrors::from(DbError(e)))
        }
    }

    pub async fn list_log(&self,len:i32)->Result<Vec<Event>,FVErrors>{
        match sqlx::query("SELECT * FROM events WHERE action != 'List' ORDER BY id DESC LIMIT $1 ")
        .bind(len)
        .map(|row:PgRow|{            
            Event{user:row.get("username"),
                time:row.get("time"),
                action:row.get("action"),
                status:row.get("status"),
                filepath:row.get("filepath"),
                args:row.get("args")}
            }
        )
        .fetch_all(&self.connection)
        .await{
            Ok(v)=>Ok(v),
            Err(e)=>Err(FVErrors::from(DbError(e)))
        }
    }
 
    pub async fn log_try(&self,session:&Session,action:UserAction,path:&String,args:&Option<String>)-> Result<(),Rejection>{
        self.append_log(Event {
            user: session.username.clone(),
            action: action.to_string(),
            time: Utc::now(),
            status: Attempt::Try.to_string(),
            filepath: path.to_string(),
            args: args.clone().unwrap_or("".to_string()),
        }).await.map_err(|e|error_to_rejection(e))?;
        Ok(())
    }

    pub async fn log_sucess(&self,session:&Session,action:UserAction,path:&String,args:&Option<String>)-> Result<(),Rejection>{
        self.append_log(Event {
            user: session.username.clone(),
            action: action.to_string(),
            time: Utc::now(),
            status: Attempt::Success.to_string(),
            filepath: path.to_string(),
            args: args.clone().unwrap_or("".to_string()),
        }).await.map_err(|e|error_to_rejection(e))?;
        Ok(())
    }
    

    //Zone
    pub async fn all_zones(&self)->Result<Vec<FileZone>,FVErrors>{
        let all_files:Vec<FileZone> =
        sqlx::query("SELECT * FROM zones;")
        .map(|row:PgRow|{
            FileZone{
            id: row.get("id"),
            name: row.get("name"),
            lords:serde_json::from_value(row.get("lords")).unwrap(),
            }
        })
        .fetch_all(&self.connection)
        .await
        .map_err(|e| FVErrors::DbError(DbError(e)))?;
    Ok(all_files)
    }
    
    pub async fn tree_zones(&self,name: String)->Result<Vec<Vec<String>>,FVErrors>{
        let all_files:Vec<(String, String)> =
        sqlx::query("SELECT name,parent_name FROM files WHERE zone = $1 AND is_directory = true;")
        .bind(name)
        .map(|row:PgRow| {
            let a= serde_json::from_value::<PathBuf>(row.get("name")).unwrap().to_string_lossy().to_string();
           let b= serde_json::from_value::<PathBuf>(row.get("parent_name")).unwrap().to_string_lossy().to_string();
           (a,b)
        })
        .fetch_all(&self.connection)
        .await
        .map_err(|e| FVErrors::DbError(DbError(e)))?;

    let x:Vec<Vec<String>>= all_files.into_iter().map(|(a,b)|vec![a,b]).collect();
    Ok(x)
    // Ok(all_files)
    }

    pub async fn insert_zone(&self,zone:FileZone)->Result<FileZone,FVErrors>{
        let sql = "INSERT INTO zones (name,lords) VALUES ($1, $2) RETURNING *";
        
        match sqlx::query(sql)
        .bind(zone.name)
        .bind(serde_json::json!(zone.lords))
        .map(|row|
            FileZone{
                id:row.get("id"),
                name:row.get("name"),
                lords:serde_json::from_value(row.get("lords")).unwrap()
        })
        .fetch_one(&self.connection)
        .await {
            Ok(v)=>Ok(v),
            Err(e)=> Err(FVErrors::DbError(DbError(e)))
        }
    }

    pub async fn get_zone_by_name(&self,name: String)->Result<FileZone,FVErrors>{
        let sql = "SELECT * FROM zones WHERE name = $1";
        
        match sqlx::query(sql)
        .bind(name)
        .map(|row: PgRow|
            FileZone{
                id:row.get("id"),
                name:row.get("name"),
                lords:serde_json::from_value(row.get("lords")).unwrap()
        })
        .fetch_one(&self.connection)
        .await {
            Ok(v)=>Ok(v),
            Err(e)=> Err(FVErrors::DbError(DbError(e)))
        }
    }
    
    pub async fn update_zone_name(&self,name:String,newname: String)->Result<(),FVErrors>{
        let sql = "UPDATE zones SET name= $1 WHERE name =$2";
        match sqlx::query(sql)
        .bind(newname)
        .bind(name)
        .execute(&self.connection)
        .await{
            Ok(_)=> Ok(()),
            Err(e)=> Err(FVErrors::DbError(DbError(e)))
        }  
    }
    pub async fn del_zone_by_name(&self,name: String)->Result<(),FVErrors>{
        let sql = "DELETE FROM zones WHERE name = $1";
        match sqlx::query(sql)
        .bind(name)
        .execute(&self.connection)
        .await {
            Ok(_)=>Ok(()),
            Err(e)=> Err(FVErrors::DbError(DbError(dbg!(e))))
        }
    }

    pub async fn update_zone_lords(&self,name:String,lords:HashSet<String>)->Result<(),FVErrors>{
        let sql = "UPDATE zones SET lords= $1 WHERE name =$2";
        match sqlx::query(sql)
        .bind(serde_json::json!(lords))
        .bind(name)
        .execute(&self.connection)
        .await{
            Ok(_)=> Ok(()),
            Err(e)=> Err(FVErrors::DbError(DbError(e)))
        }    // Ok(())
    }

    pub async fn sync_zones(&self,config: Arc<Config>)->Result<(),FVErrors>{

        let db_zones:HashSet<String>=match  sqlx::query("SELECT * FROM zones")
        .map(|row: PgRow|
            FileZone { id: row.get("id"), name: row.get("name"), lords: serde_json::from_value(row.get("lords")).unwrap()}
        )
        .fetch_all(&self.connection)
        .await
        {
            Ok(v)=>v.into_iter().map(|v|v.name).collect::<HashSet<String>>(),
            Err(e)=>{println!("?");return Err(FVErrors::from(DbError(e)));}
        };

        // // 获取磁盘文件
        
        let disk_zones = collect_zones(config.clone()).await?.into_iter().collect::<HashSet<String>>();
        let to_delete: Vec<_> = db_zones
        .difference(&disk_zones)
        .cloned()
        .collect();
        

        let to_add: Vec<_> = disk_zones
        .difference(&db_zones)
        .cloned()
        .collect();

        dbg!(&db_zones,&disk_zones,&to_add,&to_delete);
        for i in to_delete.into_iter(){self.del_zone_by_name(i).await?};
        
        for i in to_add.into_iter(){
            let zone = FileZone { id: 0, 
                name: i.clone(),lords:HashSet::new()};
            self.insert_zone(zone).await?;
        }

        Ok(())


    }

    
}

#[cfg(test)]
mod tests{
use super::*;

    #[tokio::test]
    async fn list_dir_all(){
        let path = PathBuf::from("./web_test/a2new/湾谷海报自建");
        let store=  Store::new("postgres://fileviewer:Fileviewer123@localhost:5432/fviewerdb").await;
        let a = store.list_dir_all(path,"public".to_string());
        dbg!(a.await);
    }
}
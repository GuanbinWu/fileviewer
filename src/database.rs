

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::Config;
// use sqlx::encode::IsNull::No;
use sqlx::postgres::{PgPoolOptions,PgPool,PgRow};
use warp::reject::{Reject, Rejection};
use std::collections::{HashSet,HashMap,VecDeque};
use sqlx::Row;

use crate::auth::{Account,AccountForStore, Event, Session, UserAction,Attempt};
// use crate::errors::FVErrors::DbError;
use crate::errors::{AuthError, DbError, FVErrors, error_to_rejection};
use crate::files::{FileEntry, collect_dirs, collect_files, get_parents, has_path_prefix, replace_prefix};
use chrono::{DateTime,Utc};
#[derive(Debug,Clone)]
pub struct Store{
    pub connection:PgPool
}

//用户表:accounts,字段:username,hashed
//文件表:files,字段[id: u64,parent_id: Option<u64>,name: PathBuf,is_directory: bool,size: i64,content_type: String,md5: Option<String>,created_at: DateTime<Utc>,modified_at: DateTime<Utc>,creator:String,last_modifier:String],
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
    pub async fn sync_disk(&self, config: Arc<Config>) -> Result<(), FVErrors> {
        // let root = config.files_disk_dir.clone();
        // 1. 从数据库加载所有目录和文件
        let dbdirs:Vec<PathBuf>=match  sqlx::query("SELECT name FROM files WHERE is_directory = true;")
        .map(|row: PgRow|
            {let v:PathBuf=serde_json::from_value(row.get("name")).unwrap();
            v}
        )
        .fetch_all(&self.connection)
        .await
        {
            Ok(v)=>v,
            Err(e)=>{println!("?");return Err(FVErrors::from(DbError(e)));}
        };
        println!("{:?}",&dbdirs);
        

        let dbfiles:Vec<PathBuf>=match  sqlx::query("SELECT name FROM files WHERE is_directory = false;")
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

        println!("{:?}",&dbfiles);
        // // 获取磁盘文件
        let disk_dirs = collect_dirs(config.clone()).await?;
        println!("{:?}",&disk_dirs);
        let disk_files = collect_files(config.clone()).await?;
        println!("{:?}",&disk_files);
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

        for i in to_delete_file.into_iter(){self.delete_files(i).await?};
        for i in to_delete_dir.into_iter(){self.delete_files(i).await?};
        
        for i in to_add_dir.into_iter(){
            let file = FileEntry { id: 0, 
                name: i.clone(), 
                parent_name: PathBuf::from(i.parent().unwrap_or(&PathBuf::from("/".to_string()))), 
                is_directory: true, 
                size: 0, 
                content_type: "Dir".to_string(), 
                md5: None, 
                created_at: Utc::now(), 
                modified_at: Utc::now(), 
                creator: "system".to_string(), 
                last_modifier: "system".to_string()};
            self.upload_files(file).await?;
        }

        
        for i in to_add_files.into_iter(){
            let bytes=tokio::fs::read(i.clone()).await.map_err(|e|FVErrors::IOError(e.to_string()))?;
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
                parent_name: PathBuf::from(i.parent().unwrap_or(&PathBuf::from("/".to_string()))), 
                is_directory: false, 
                size: size, 
                content_type: content_type, 
                md5: Some(md5), 
                created_at: Utc::now(), 
                modified_at: Utc::now(), 
                creator: "system".to_string(), 
                last_modifier: "system".to_string()};
            self.upload_files(file).await?;
        }
        self.sync_parent_name().await?;
        
        Ok(())

    }
    
    pub async fn sync_parent_name(&self)->Result<(),FVErrors>{
        let dbdirs:Vec<(i64,PathBuf)>=match  sqlx::query("SELECT id,name FROM files;")
        .map(|row: PgRow|
            {let id = row.get("id");
                let name:PathBuf=serde_json::from_value(row.get("name")).unwrap();
            (id,get_parents(&name))}
        )
        .fetch_all(&self.connection)
        .await
        {
            Ok(v)=>v,
            Err(e)=>{println!("?");return Err(FVErrors::from(DbError(e)));}
        };
        // println!("{:?}",&dbdirs);
        for (id,pname) in dbdirs{
        self.update_files_info_pthbuf(id,  "parent_name".to_string(), pname)
            .await?;}
        Ok(())
    }
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

    pub async fn remove_account(&self,name:String)->Result<(),FVErrors>{

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

    pub async fn list_dir (&self,pname:PathBuf)->Result<Vec<FileEntry>,FVErrors>{
        println!("{:?}",&pname);
        let sql = "SELECT * FROM files WHERE parent_name = $1";
        match sqlx::query(sql)
        .bind(serde_json::json!(pname))
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
            last_modifier:row.get("last_modifier")}
        }

        )
        .fetch_all(&self.connection)
        .await{
            Ok(v)=>Ok(v),
            Err(e)=>Err(FVErrors::from(DbError(e)))
        }
    }
    
    pub async fn get_dir_entry(&self,name:PathBuf)->Result<FileEntry,FVErrors>{
        match sqlx::query(
            "SELECT * FROM files WHERE name = $1 AND is_directory = true"
        )
        .bind(serde_json::json!(name))
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
            last_modifier:row.get("last_modifier")})
            },
            _ => Err(FVErrors::NotFound)
        }
    }
    
    pub async fn get_file_entry(&self,name:PathBuf)->Result<FileEntry,FVErrors>{
        match sqlx::query(
            "SELECT * FROM files WHERE name = $1 AND is_directory = false"
        )
        .bind(serde_json::json!(name))
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
            last_modifier:row.get("last_modifier")})
            },
            _ => Err(FVErrors::NotFound)
        }


    }

    pub async fn upload_files(&self,file:FileEntry)->Result<FileEntry,FVErrors>{
        println!("Uploading files {:?}",&file);

        let sql = "INSERT INTO files (name,parent_name, is_directory, size, content_type, md5, created_at, modified_at, creator, last_modifier)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
            last_modifier:row.get("last_modifier")}
        })
        .fetch_one(&self.connection)
        .await{
             Ok(v)=>Ok(v),
            Err(e)=>Err(FVErrors::from(DbError(e)))
        }
    }
    
    pub async fn delete_files(&self,name:PathBuf)->Result<(),FVErrors>{
        let sql = "DELETE FROM files WHERE name = $1";
        sqlx::query(sql)
        .bind(serde_json::json!(name))
        .execute(&self.connection)
        .await
        .map_err(|e| FVErrors::from(DbError(e)))?;
    Ok(())
    }
    pub async fn list_dir_all(&self,dir:PathBuf)->Result<Vec<FileEntry>,FVErrors>{
        let mut queue = VecDeque::new();
        let mut result:Vec<FileEntry> = Vec::new();
        // let this = self.get_dir_entry(dir.clone()).await?;
        // println!("{:?}",&this);
        // result.push(this);

        queue.push_back(dir.clone());
        while let Some(current_dir) = queue.pop_front() {

            let entries:Vec<FileEntry> = self.list_dir(current_dir).await?;

            for entry in entries {                
                result.push(entry.clone());
                if entry.is_directory {
                    queue.push_back(entry.name.clone());
                }
            }
        }
        Ok(result)
    }

    pub async fn delete_dir_all(&self,dir:PathBuf)->Result<(),FVErrors>{
        // let mut tmp =Vec::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        result.push(dir.clone());        
        queue.push_back(dir.clone());
        while let Some(current_dir) = queue.pop_front() {

            let entries:Vec<(PathBuf, bool)> = self.list_dir(current_dir)
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
            self.delete_files(i).await?;
        }
        Ok(())

    }


    pub async fn rename_dir(&self,dir:PathBuf,new_dir:PathBuf)->Result<(),FVErrors>{
        let all_files:Vec<FileEntry> =
        sqlx::query("SELECT * FROM files;")
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
            last_modifier:row.get("last_modifier")}
        })
        .fetch_all(&self.connection)
        .await
        .map_err(|e| FVErrors::DbError(DbError(e)))?;
        

        let names:Vec<(i64,PathBuf)>=all_files.clone().into_iter()
        .filter(|f|has_path_prefix(&f.name,&dir))
        .map(|f| (f.id,f.name))
        .collect();
        
        for (id,name) in names{
           let new_name = replace_prefix(&name,&dir, &new_dir);
           let new_pname=get_parents(&new_name);
           self.update_files_info_pthbuf(id,  "parent_name".to_string(),new_pname).await?;
            self.update_files_info_pthbuf(id,  "name".to_string(),new_name).await?;
            
        }

        // for name in panmes{
        //     let new_name = replace_prefix(&name,&dir, &new_dir);
        //     self.update_files_info_pthbuf(name, "parent_name".to_string(),"parent_name".to_string(), new_name).await?;
        // }

    Ok(())
    }
    

    pub async fn update_files_info_string(&self,id:i64,col:String,value:String)->Result<(),FVErrors>{
        let sql = format!("UPDATE files SET {} = $1 WHERE id = $2 ",col);
        sqlx::query(&sql)
        .bind(value)
        .bind(id)
        .execute(&self.connection)
        .await
        .map_err(|e| {println!("{:?}",e);FVErrors::from(DbError(e))})?;

    Ok(())
    }



    pub async fn update_files_info_pthbuf(&self,id:i64,target_col:String,value:PathBuf)->Result<(),FVErrors>{
        let sql = format!("UPDATE files SET {} = $1 WHERE id = $2 ",target_col);
        sqlx::query(&sql)
        .bind(serde_json::json!(value))
        .bind(id)
        .execute(&self.connection)
        .await
        .map_err(|e| {println!("{:?}",e);FVErrors::from(DbError(e))})?;
    Ok(())
    }

    pub async fn update_files_info_time(&self,id:i64,col:String,value:DateTime<Utc>)->Result<(),FVErrors>{
        let sql = format!("UPDATE files SET {} = $2 WHERE id = $1",col);

        sqlx::query(&sql)
        .bind(id)
        .bind(value)
        .fetch_one(&self.connection)
        .await
        .map_err(|e| FVErrors::from(DbError(e)))?;

    Ok(())
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
    


    pub async fn chdir_creator(&self,dir:PathBuf,value:String)->Result<(),FVErrors>{
        let all_files:Vec<FileEntry> =
        sqlx::query("SELECT * FROM files;")
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
            last_modifier:row.get("last_modifier")}
        })
        .fetch_all(&self.connection)
        .await
        .map_err(|e| FVErrors::DbError(DbError(e)))?;
        
        let idxs:Vec<i64>=all_files.clone().into_iter()
        .filter(|f|has_path_prefix(&f.name,&dir))
        .map(|f| f.id)
        .collect();
        
        for id in idxs{
            self.update_files_info_string(id,"creator".to_string(),value.clone()).await?;
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
        println!("{:?}",&store);
        println!("{:?}",store.list_dir_all(path).await);
        
        // let prefix = PathBuf::from("./web_test/a2");
        // let new =PathBuf::from("./web_test/ccc/a2_copy");
        // println!("{:?}",replace_prefix(&path, &prefix, &new));

    }
}
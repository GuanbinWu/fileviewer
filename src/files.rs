use chrono::{DateTime, Utc};
use std::path::{self, Path, PathBuf};
use serde::{Serialize,Deserialize};
use crate::{config::Config, errors::FVErrors};
use std::sync::Arc;
use tokio::fs::ReadDir;


#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FileEntry{
    pub id: i64,
    pub name: PathBuf,
    pub parent_name:PathBuf,
    pub is_directory: bool,
    pub size: i64,
    pub content_type: String,
    pub md5: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub creator:String,
    pub last_modifier:String,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FileEntryResponse{
    pub id: i64,
    pub name: PathBuf,
    pub parent_name:PathBuf,
    pub is_directory: bool,
    pub size: i64,
    pub content_type: String,
    pub md5: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub creator:String,
    pub last_modifier:String,
}


impl FileEntry {
    pub fn into_response(&self,config:Arc<Config>)-> FileEntryResponse {
        FileEntryResponse { 
        id: self.id, 
        name: rm_prefix(&self.name, config.clone()), 
        parent_name: rm_prefix(&self.parent_name, config), 
        is_directory: self.is_directory, 
        size: self.size, 
        content_type: self.content_type.clone(), 
        md5: self.md5.clone(), 
        created_at:self.created_at, 
        modified_at: self.modified_at, 
        creator: self.creator.clone(), 
        last_modifier: self.last_modifier.clone()}
    }
}


impl FileEntryResponse {
    pub fn from_file_entry(f: FileEntry,config: Arc<Config>) -> Self {
        FileEntryResponse { 
            id: f.id, 
            name: rm_prefix(&f.name, config.clone()), 
            parent_name: rm_prefix(&f.parent_name, config), 
            is_directory: f.is_directory, 
            size: f.size, 
            content_type: f.content_type, 
            md5: f.md5, 
            created_at:f.created_at, 
            modified_at: f.modified_at, 
            creator: f.creator, 
            last_modifier: f.last_modifier }
    }
}




fn clean(path: PathBuf) -> PathBuf {
    let mut path_str = path.to_string_lossy().replace('\\', "/");
    if path_str.ends_with('/') {
        path_str.pop();
    }
    
    let a = PathBuf::from(path_str);
    // println!("{:?}",&a);
    a
}

pub async fn collect_dirs(conf: Arc<Config>) -> Result<Vec<PathBuf>, FVErrors>{
    
    let root=conf.files_disk_dir.clone();
    // println!("Collecting Files from {:?}",&root);
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let mut read_dir = tokio::fs::read_dir(&dir).await.map_err(|e|FVErrors::IOError(e.to_string()))?;
        while let Some(entry) = read_dir.next_entry().await.map_err(|e|FVErrors::IOError(e.to_string()))? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.clone());
                files.push(path);
            } else {
                // files.push(path);
            }
        }
    }
    // println!("{:?}",&files);
    
    Ok(files.into_iter().map(|v| clean(v)).collect())
}


pub async fn collect_files(conf: Arc<Config>) -> Result<Vec<PathBuf>, FVErrors>{
    
    let root=conf.files_disk_dir.clone();
    // println!("Collecting Files from {:?}",&root);
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let mut read_dir = tokio::fs::read_dir(&dir).await.map_err(|e|FVErrors::IOError(e.to_string()))?;
        while let Some(entry) = read_dir.next_entry().await.map_err(|e|FVErrors::IOError(e.to_string()))? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.clone());
                // files.push(path);
            } else {
                files.push(path);
            }
        }
    }
    // println!("{:?}",&files);
    Ok(files.into_iter().map(|v| clean(v)).collect())
}


pub fn has_path_prefix(path: &PathBuf, prefix: &PathBuf) -> bool {
    let path = path.components();
    let prefix = prefix.components();
    let mut path_iter = path.peekable();
    let mut prefix_iter = prefix.peekable();
    while let Some(p) = prefix_iter.next() {
        match path_iter.next() {
            Some(x) if x == p => continue,
            _ => return false,
        }
    }
    true
}

pub fn replace_prefix(path:&PathBuf,prefix: &PathBuf,new_prefix:&PathBuf)->PathBuf{
    
    // println!("{:?}",path.clone().strip_prefix(prefix.clone()));
    
    
    if let Ok(rest) = path.strip_prefix(prefix) {
        // println!("{:?}",&rest);
        let mut out = new_prefix.clone();
        // println!("{:?}",&out);
        out.push(rest);
        // println!("Replacing prefix : from {:?} to {:?}",&path,&out);
        clean(out)
         
    } else {
        // println!("Replacing prefix : from {:?} to {:?}",&path,&path);
        clean(path.clone())
    }
   
}

pub fn rm_prefix(path:&PathBuf,config: Arc<Config>)->PathBuf{

    let stripped = path
    .strip_prefix(config.files_disk_dir.to_str().unwrap())
    .unwrap_or(&path)
    .to_path_buf();
    clean(stripped)
}

pub fn add_disk_base_prefix(path:&PathBuf,config: Arc<Config>)->PathBuf{
    clean(config.files_disk_dir.join(&path))
    
}

pub fn get_parents(path:&PathBuf)->PathBuf{
    match  path.clone().parent(){
        Some(v)=>clean(PathBuf::from(v)),
        None => PathBuf::from("/")
    }

}

#[cfg(test)]
mod tests{
use super::*;

    #[test]
    fn replace_prefix_test(){
        let path = PathBuf::from("./web_test/a2/a4");
        let prefix = PathBuf::from("./web_test/a2");
        let new =PathBuf::from("./web_test/ccc/a2_copy");
        // println!("{:?}",replace_prefix(&path, &prefix, &new));

    }
}

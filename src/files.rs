use chrono::{DateTime, Utc};
use std::{collections::HashSet, path::{self, Path, PathBuf}, str::FromStr};
use serde::{Serialize,Deserialize};
use crate::{auth, config::Config, errors::{FVErrors, PathError}};
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
    pub zone:String,
}

// pub type FileEntryResponse = FileEntry;

// impl FileEntry {
//     pub fn into_response(&self,config:Arc<Config>)-> FileEntryResponse {
//         FileEntryResponse { 
//         id: self.id, 
//         name: rm_prefix(&self.name, config.clone()), 
//         parent_name: rm_prefix(&self.parent_name, config), 
//         is_directory: self.is_directory,
//         size: self.size, 
//         content_type: self.content_type.clone(), 
//         md5: self.md5.clone(), 
//         created_at:self.created_at, 
//         modified_at: self.modified_at, 
//         creator: self.creator.clone(), 
//         last_modifier: self.last_modifier.clone(),
//         zone:self.zone.clone()
//         }
//     }
//     pub fn from_file_entry(f:FileEntryResponse,config: Arc<Config>)->Self{
//         FileEntry { 
//             id: f.id,
//             name: rm_prefix(&f.name, config.clone()), 
//             parent_name: rm_prefix(&f.parent_name, config), 
//             is_directory: f.is_directory, 
//             size: f.size, 
//             content_type: f.content_type, 
//             md5: f.md5, 
//             created_at:f.created_at, 
//             modified_at: f.modified_at, 
//             creator: f.creator, 
//             last_modifier: f.last_modifier,
//             zone: f.zone}
//     }
// }

#[derive(Debug,Serialize,Deserialize,Eq,PartialEq)]
pub struct FileZone{
    pub id:i32,
    pub name:String,
    pub lords:HashSet<String>,
}


impl FileZone {
    pub fn new(id:i32,name: String,)->Self{
        FileZone {id, name,lords:HashSet::new()}
    }
}


pub trait PathBehavior {
    fn to_web_path(&self,sys_disk_dir:&PathBuf,zone:&str)->Result<PathBuf,PathError>;
    fn to_sys_path(&self,sys_disk_dir:&PathBuf,zone:&str)->Result<PathBuf,PathError>;
    fn to_unix_path(&self) -> Result<PathBuf,PathError>;
    fn has_path_prefix(&self, prefix: &PathBuf) -> bool;
    fn replace_prefix(&self,prefix: &PathBuf,new_prefix:&PathBuf)->Result<PathBuf,PathError>;
    fn rm_prefix(&self,prefix: &PathBuf)->Result<PathBuf,PathError>;
    fn add_prefix(&self,prefix: &PathBuf)->Result<PathBuf,PathError>;
    fn get_parent(&self)->Result<PathBuf,PathError>;
    fn concat(&self,path:&PathBuf)->Result<PathBuf,PathError>;
    fn to_absolute(&self)->Result<PathBuf,PathError>;
}


impl PathBehavior for PathBuf {

    fn to_web_path(&self,sys_disk_dir:&PathBuf,zone:&str)->Result<PathBuf,PathError>{
        self.rm_prefix(sys_disk_dir)
        .and_then(|x| x.rm_prefix(&PathBuf::from(zone.to_string())))
        .and_then(|v| v.to_unix_path())
        // .and_then(|v|v.add_prefix(&PathBuf::from("/")))     
    }

    fn to_sys_path(&self,sys_disk_dir:&PathBuf,zone:&str)->Result<PathBuf,PathError>{
        self.add_prefix(&PathBuf::from(zone.to_string()))
        .and_then(|v | v .add_prefix(sys_disk_dir))
        .and_then(|v| v.to_unix_path())
    }

    fn to_unix_path(&self) -> Result<PathBuf,PathError>{
        let mut path_str = self.to_string_lossy().replace('\\', "/");
        if path_str.ends_with('/') {
            path_str.pop();
        }
        match PathBuf::from_str(&path_str){
            Ok(v) =>  Ok(v),
            Err(_) =>Err(PathError::E101)
        }
    }

    fn has_path_prefix(&self, prefix: &PathBuf) -> bool{
        let path = self.components();
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

    fn replace_prefix(&self,prefix: &PathBuf,new_prefix:&PathBuf)->Result<PathBuf,PathError>{
        if let Ok(rest) = self.strip_prefix(prefix) {
            let mut out = new_prefix.clone();
            out.push(rest);
            return out.to_unix_path()
        } else {
            return Err(PathError::E101)
        }
    }

    fn rm_prefix(&self,prefix: &PathBuf)->Result<PathBuf,PathError>{
        match self.strip_prefix(prefix){
            Ok(v)=> Ok(v.to_path_buf()),
            Err(_)=> Err(PathError::E101)
        }
    }

    fn add_prefix(&self,prefix: &PathBuf)->Result<PathBuf,PathError>{
        let mut tmp = prefix.clone();
        tmp.push(self);
        Ok(tmp)
    }

    fn get_parent(&self)->Result<PathBuf,PathError>{
        match  self.parent(){
        Some(v)=>Ok(v.to_path_buf()),
        None => Ok(PathBuf::from("/"))
        }
    }
    
    fn concat(&self,path:&PathBuf)->Result<PathBuf,PathError>{
        // dbg!(self);
        let mut tmp = self.clone();
        for i in  path.components().peekable(){
            tmp.push(i);
            // dbg!(&tmp);
            // dbg!(&i);
        }
        Ok(tmp.to_unix_path().unwrap())
    }

    fn to_absolute(&self)->Result<PathBuf,PathError>{
        let path:&Path = self.as_ref();
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            Ok(std::env::current_dir().map_err(|_|PathError::E101)?.join(path))
        }
    }
}



// fn to_unix_path(path: PathBuf) -> PathBuf {
//     let mut path_str = path.to_string_lossy().replace('\\', "/");
//     if path_str.ends_with('/') {
//         path_str.pop();
//     }
//     PathBuf::from(path_str)
// }



pub async fn collect_dirs(conf: Arc<Config>,zone:&str) -> Result<Vec<PathBuf>, FVErrors>{
    
    let root=conf.sys_disk_dir.clone().concat(&PathBuf::from(zone)).unwrap();
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
    Ok(files.into_iter().map(|v| v.to_unix_path().unwrap()).collect())
}

pub async fn collect_zones(conf:Arc<Config>) -> Result<Vec<String>,FVErrors>{
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&conf.sys_disk_dir).map_err(|e|FVErrors::IOError(e.to_string()))? {
        let entry = entry.map_err(|e|FVErrors::IOError(e.to_string()))?;
            if entry.file_type().map_err(|e|FVErrors::IOError(e.to_string()))?.is_dir() {
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|e|FVErrors::IOError(format!("{:?}",e)))?;
                names.push(name);
            }
        }
    Ok(names)
}

pub async fn collect_files(conf: Arc<Config>,zone:&str) -> Result<Vec<PathBuf>, FVErrors>{
    
    let root=conf.sys_disk_dir.clone().concat(&PathBuf::from(zone)).unwrap();
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
    Ok(files.into_iter().map(|v| v.to_unix_path().unwrap()).collect())
}

pub fn folder_size(path: impl AsRef<Path>) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            if metadata.is_file() {
                Some(metadata.len())
            } else {
                None
            }
        })
        .sum()
}

pub fn disk_capacity(path: impl AsRef<Path>) -> Option<(u64,u64,u64)> {
    let path = path.as_ref();
    dbg!(&path);
    let disks = sysinfo::Disks::new_with_refreshed_list();
    dbg!(&disks);
    let mut best = None;
    let mut best_len = 0;
    for disk in disks.list() {
        let mount = disk.mount_point();
        if path.starts_with(mount) {
            let len = mount.as_os_str().len();
            if len > best_len {
                best_len = len;
                best = Some((
                    disk.total_space(),
                    disk.available_space(),
                    disk.total_space() - disk.available_space(),
                ));
            }
        }
    }
    dbg!(best)
}

pub fn zip_dir(dir: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    let dir = dir.as_ref();
    let mut buffer = Vec::new();
    let cursor = std::io::Cursor::new(&mut buffer);
    let mut zip = zip::ZipWriter::new(cursor);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    #[cfg(unix)]
    let options = options.unix_permissions(0o644);
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();
        let rel_path = path.strip_prefix(dir).unwrap();
        if rel_path.as_os_str().is_empty() {
            continue;
        }
        let name = rel_path
            .to_string_lossy()
            .replace('\\', "/");
        if entry.file_type().is_dir() {
            zip.add_directory(name, options)?;
        } else {
            zip.start_file(name, options)?;
            let mut f = std::fs::File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        }
    }
    let cursor = zip.finish()?;
    Ok(cursor.into_inner().to_vec())
}


#[cfg(test)]
mod tests{
use std::str::FromStr;

use super::*;

    #[test]
    fn replace_prefix(){
        let path = PathBuf::from("./web_test/a2/a4/test.txt");
        let prefix = PathBuf::from("./web_test/a2");
        let new =PathBuf::from("./web_test/ccc/a2_copy");
        // println!("{:?}",replace_prefix(&path, &prefix, &new));
        let y= PathBuf::from("./web_test/ccc/a2_copy/a4/test.txt");
        assert_eq!(path.replace_prefix(&prefix, &new).unwrap(),y);
    }
    #[test]
    fn to_unix_style(){
        assert_eq!(PathBuf::from(r"C:\Users\me\file.txt").to_unix_path().unwrap(), PathBuf::from("C:/Users/me/file.txt").to_unix_path().unwrap());
        assert_eq!(PathBuf::from(r"..\data\input.csv").to_unix_path().unwrap(), PathBuf::from("../data/input.csv").to_unix_path().unwrap());
        assert_eq!(PathBuf::from(r"/usr/local/bin").to_unix_path().unwrap(), PathBuf::from("/usr/local/bin").to_unix_path().unwrap());
        assert_eq!(PathBuf::from(r"\\server\share\dir").to_unix_path().unwrap(), PathBuf::from("//server/share/dir").to_unix_path().unwrap());
    }
    
    #[test]
    fn web_to_sys(){
        let sys=PathBuf::from("./web_test");
        let zone = "public";
        let a = PathBuf::from("./web_test/public/1.txt");
        let b = PathBuf::from("1.txt");
        assert_eq!(a.to_web_path(&sys, zone).unwrap(),b);
        assert_eq!(b.to_sys_path(&sys, zone).unwrap(),a);

        let a = PathBuf::from("");
        let b = PathBuf::from("./web_test/测试");

        assert_eq!(a.to_sys_path(&sys, "测试").unwrap(),b);
    }
    
    #[test]
    fn concat(){
        let a=PathBuf::from("./web_test");
        let b = PathBuf::from("public/name.txt");
        let c = PathBuf::from("./web_test/public/name.txt");
        // dbg!(&a);
        assert_eq!(a.concat(&b).unwrap(),c);
    }
}

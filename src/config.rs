use std::path::PathBuf;

#[derive(Debug,Clone)]
pub struct Config{
    pub ip:[u8;4],
    pub port:u16,
    pub sys_disk_dir:PathBuf,
    pub config_dir:PathBuf,
    pub web_resources:PathBuf,
    pub max_username_len:u64,
    pub min_pwd_len:u8,
    pub default_pwd:String,
    pub db_url:String,

}

impl Config {
    pub fn new()->Self{
        Config{
            ip:[0u8,0u8,0u8,0u8],
            port:3344,
            sys_disk_dir:PathBuf::from("./web_test".to_string()),
            config_dir:PathBuf::from("./conf".to_string()),
            web_resources:PathBuf::from("./web_resources".to_string()),
            max_username_len:20,
            min_pwd_len:6,
            default_pwd:"123456abcd".to_string(),
            db_url:"postgres://fileviewer:Fileviewer123@localhost:5432/fviewerdb".to_string(),
            // token_key:,
        }
    }
}

use thiserror::Error;




#[derive(Error,Debug)]
pub enum FVErrors{
    #[error("DbError")]
    DbError(#[from] DbError),
    #[error("AccountError")]
    AuthError(#[from] AuthError),
    // #[error("FileError")]
    // FileError(#[from])
    #[error("Notfound")]
    NotFound,
    #[error("IO Error")]
    IOError(String),
    #[error("Path Error")]
    PathError(#[from] PathError)
}

#[derive(Debug,Error)]
pub struct DbError(pub sqlx::Error);

// impl warp::reject::Reject for DbError {}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}



#[derive(Debug,Error)]
pub enum AuthError{
    #[error("加密失败")]
    EncodeError,
    #[error("解密失败")]
    DecodeError,
    #[error("无效用户名")]
    InvalidUsername,
    #[error("无效用户名已被使用")]
    UsernameUsed,
    #[error("无效密码")]
    InvalidPassword,
    #[error("无此用户")]
    NoSuchUser,
}

#[derive(Error,Debug)]
pub enum PathError {
    #[error("路径处理失败")]
    E101
}

impl warp::reject::Reject for DbError {}
impl warp::reject::Reject for FVErrors {}
impl warp::reject::Reject for AuthError {}

pub fn error_to_rejection(e:FVErrors)->warp::reject::Rejection{
    match e {
        FVErrors::DbError(v) => warp::reject::custom(v),
        FVErrors::AuthError(v) =>warp::reject::custom(v),
        FVErrors::NotFound =>warp::reject::reject(),
        FVErrors::IOError(e)=>warp::reject::reject(),
        FVErrors::PathError(e)=>warp::reject::reject(),
    }
}
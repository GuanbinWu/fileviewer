
# API
## no auth
- GET /portal/login/ 登录界面
- GET /protal/files/ 文件门户界面
  

- POST /api/accounts/regist/ 注册账户
- POST /api/accounts/login/ 上传登录账户密码
- PATCH /api/accounts/updatepwd/ 更新密码

- DELETE /api/accounts/delete/ 注销账户

## need auth
- GET /api/accounts/list/ 获取所有用户
- POST /api/accounts/logout/ 登出
- 
- GET /api/zone/list 获取所有的文件Zone信息
- PATCH /api/zone/rename
- POST /api/zone/create
- PATCH /api/zone/newlords
- GET /api/zone/tree?zone=xxx 
  
- POST /api/auth/verify/ 验证会话有效期

- POST /api/files/list 获取目录
- POST /api/files/upload 上传文件或目录
- POST /api/files/download 下载文件或目录
- POST /api/files/delete 删除文件或目录
- PATCH /api/files/rename 移动（重命名）文件或目录
- POST /api/files/copy 复制文件或目录
- PATCH /api/files/chown 修改归属人

- GET /api/log?len=100 获取日志

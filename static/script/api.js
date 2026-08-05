



//Auth

export async function login(BASE,body,token) {
    res = await fetch(`${BASE}/login`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body)
        });
    return res
}

export async function logout(BASE,token) {
    const res = await fetch(`${BASE}/login/logout`, {
          method: "POST",
          headers: { "Content-Type": "application/json","Authorization":token },
          // body: JSON.stringify(body)
        });
    return res
}


// list_dir
export async function list_dir(BASE,body,token) {
  // console.log(body)

  const res = await fetch(`${BASE}/list`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `${token}`,
    },
    body: JSON.stringify(body),
  });
  return res;
}

// 4. 删除文件/目录
export async function deleteFile(BASE,body,token) {
  //文件夹 body：{filename:被删除的文件目录,is_dir:true,args:null,bytes:any}
  //文件 body：{filename:被删除的文件全名,is_dir:false,args:null,bytes:null}
  const res = await fetch(`${BASE}/delete`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token
    },
    body: JSON.stringify(body),
  });
  items = await res.json();

}

// 5. 上传文件
export async function uploadFile(BASE,body,token) {
  //文件夹 body：{filename:当前工作目录,is_dir:true,args:新文件夹名称全称,bytes:any}
  //文件 body：{filename:新文件名全称,is_dir:false,args:md5值,bytes：文件字节流}
  const res = await fetch(`${BASE}/upload`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' ,"Authorization":token},
    body: JSON.stringify(body),
  });
  return res;
}

// 6. 下载文件
export async function downloadFile(BASE,body,token) {
    const res = await fetch(`${BASE}/download`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token
    },
    body: JSON.stringify(body),
  });
  return res;
}

// 7. 获取文件详情

export async function renameFile(BASE,body,token) {
  const res = await fetch(`${BASE}/rename`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token
    },
    body: JSON.stringify(body),
  });
  res
}


// 10. 复制文件
export async function copyFile(BASE,body,token) {
  const res = await fetch(`${BASE}/copy`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token
    },
    body: JSON.stringify(body),
  });
  res
}

// 11. 预览文件
export async function previewFile(BASE,dir,name,args,token) {

}


//Done
export async function changeCreator(BASE,body,token) {

  const res = await fetch(`${BASE}/chown`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token
    },
    body: JSON.stringify(body),
  });
  res;

}

export async function list_log(BASE,len,token) {
    const res = await fetch(`${BASE}/log?len=${len}`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token
    },
  });
  return res;
}
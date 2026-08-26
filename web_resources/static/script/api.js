
class Request{
  constructor(){}
  file_list(zone,dir){
    return JSON.stringify({
      "zone":zone,
      "dir":dir})
  }
  file_upload(zone,is_dir,filename,md5,bytes){
    return JSON.stringify({
      "zone":zone,
      "is_dir":is_dir,
      "filename":filename,
      "md5":md5,
      "bytes":bytes
    })
  }
  file_rename(zone,is_dir,filename,new_filename){
    return JSON.stringify({
      "zone":zone,
      "is_dir":is_dir,
      "filename":filename,
      "new_filename":new_filename,
    })
  }
  file_download(zone,is_dir,filename){
    return JSON.stringify({
      "zone":zone,
      "is_dir":is_dir,
      "filename":filename,
    })
  }
  file_copy(zone,is_dir,filename,new_filename){
    return JSON.stringify({
      "zone":zone,
      "is_dir":is_dir,
      "filename":filename,
      "new_filename":new_filename,
    })
  }
  file_delete(zone,is_dir,filename){
    return JSON.stringify({
      "zone":zone,
      "is_dir":is_dir,
      "filename":filename,
    })
  }
  file_chown(zone,is_dir,filename,creator){
    return JSON.stringify( {
      "zone":zone,
      "is_dir":is_dir,
      "filename":filename,
      "creator":creator,
    })
  }
  account_regular(username,password){
    return JSON.stringify({
      "username":username,
      "password":password
    })
  }
  account_withNewpwd(username,password,newpwd){
    return JSON.stringify({
      "username":username,
      "password":password,
      "newpwd":newpwd
    })
  }
  zone_withLords(name,lords){
    return JSON.stringify({
      "name":name,
      "lords":lords
    })
  }
  zone_withNewName(name,newname){
    return JSON.stringify({
      "name":name,
      "new_name":newname,
    })
  }
}




const AccountsBase="/api/accounts"
const FileBase="/api/files"
const ZoneBase="/api/zone"
const AuthBase="/api/auth"
const LogBase= "/api/log"
export async function accounts_regist(username,password) {
  const res = await fetch(`${AccountsBase}/regist`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: new Request().account_regular(username,password) 
  });
  return res
}

export async function accounts_login(username,password) {
  const res = await fetch(`${AccountsBase}/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: new Request().account_regular(username,password) });
  return res
}

export async function accounts_updatepwd(username,password,newpwd) {
  const res = await fetch(`${AccountsBase}/updatepwd`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: new Request().account_withNewpwd(username,password,newpwd) });
  return res
}


export async function accounts_delete(username,password) {
  const res = await fetch(`${AccountsBase}/logout`, {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
    body: new Request().account_regular(username,password) });
  return res
}

//need token
export async function accounts_list(token) {
  const res = await fetch(`${AccountsBase}/list`, {
    method: "GET",
    headers: { "Content-Type": "application/json", 'Authorization': token }
  });
  // console.log
  return await res.json()
}

export async function accounts_logout(token) {
  const res = await fetch(`${AccountsBase}/logout`, {
    method: "POST",
    headers: { "Content-Type": "application/json",
      'Authorization': token
     }
    });
  return res
}

export async function zone_list(token) {
  const res = await fetch(`${ZoneBase}/list`, {
    method: "GET",
    headers: { "Content-Type": "application/json",
      'Authorization': token
    }
    });
  return await res.json()
}

export async function zone_delete(token,name) {
  const res = await fetch(`${ZoneBase}/delete?zone=${name}`, {
    method: "DELETE",
    headers: { "Content-Type": "application/json",
      'Authorization': token
    }
    });
  return res
}

export async function zone_tree(token,zoneName) {
  const params = new URLSearchParams({
    zone: zoneName,
    });
  const res = await fetch(`${ZoneBase}/tree?${params.toString()}`, 
    {
    method: "GET",
    headers: { "Content-Type": "application/json",
      'Authorization': token
    }});
  return await res.json()
}

export async function zone_rename(token,name,newName) {
  const res = await fetch(`${ZoneBase}/rename`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json",
      'Authorization': `${token}`
    },
    body:new Request().zone_withNewName(name,newName)
    });
  return res
}

export async function zone_create(token,name,lords) {
  const res = await fetch(`${ZoneBase}/create`, {
    method: "POST",
    headers: { "Content-Type": "application/json",
      'Authorization': token,
    },
    body:new Request().zone_withLords(name,lords)
    });
  return res
}
export async function zone_newlords(token,name,newLords) {
  const res = await fetch(`${ZoneBase}/newlords`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json",
      'Authorization': token,
    },
    body:new Request().zone_withLords(name,newLords)
    });
  return res
}

export async function auth_verify(token) {
  const res = await fetch(`${AuthBase}/verify?token=${token}`,{
    method:"POST",
    headers: { "Content-Type": "application/json" },
  });
  return res;
}

export async function files_list(token,zone,dir) {
  const res = await fetch(`${FileBase}/list`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token,
    },
    body: new Request().file_list(zone,dir),
  });
  return await res.json();
}
export async function files_upload(token,zone,is_dir,filename,md5,bytes) {
  const res = await fetch(`${FileBase}/upload`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token,
    },
    body: new Request().file_upload(zone,is_dir,filename,md5,bytes),
  });
  return res;
}
export async function files_download(token,zone, is_dir, filename) {
  const res = await fetch(`${FileBase}/download`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token,
    },
    body: new Request().file_download(zone, is_dir, filename),
  });
  return res;
}
export async function files_delete(token,zone,is_dir, filename) {
  const res = await fetch(`${FileBase}/delete`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token,
    },
    body: new Request().file_delete(zone, is_dir, filename),
  });
  return res;
}
export async function files_rename(token,zone, is_dir, filename, new_filename) {
  const res = await fetch(`${FileBase}/rename`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token,
    },
    body: new Request().file_rename(zone, is_dir, filename, new_filename),
  });
  return res;
}
export async function files_copy(token,zone, is_dir, filename, new_filename) {
  const res = await fetch(`${FileBase}/copy`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token,
    },
    body: new Request().file_copy(zone, is_dir, filename, new_filename),
  });
  return res;
}
export async function files_chown(token,zone,is_dir,filename,creator) {
  const res = await fetch(`${FileBase}/chown`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token,
    },
    body: new Request().file_chown(zone, is_dir, filename, creator),
  });
  return res;
}

export async function log(token,len=100) {
  const res = await fetch(`${LogBase}?len=${len}`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token,
    },
  });
  return await res.json();
}

export async function zone_size(token) {
  const res = await fetch(`${ZoneBase}/size`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': token,
    },
  });
  return res;
}
import * as API from "./api.js";
import { Path,FMT } from "./utils.js";
import {Render} from "./render.js"
let currentPath = new Path();
let TOKEN = localStorage.getItem("token");
const BASE=""
const FILEBASE="/files"
let currentItems=[];
let currentMod ="files";


async function refreshDir(dir = currentPath) {
  const body =construct_rq_body(dir.to_string_no_root(),true,null,null)
  const items = await API.list_dir(FILEBASE,body,TOKEN);
  // console.log(items)
  document.getElementById("currentPath").textContent = dir.to_string_with_root();
  try {
    currentItems =await items.clone().json();
    let sortedItems =await items.json();
    sortedItems.sort((a,b) => new Path().from_string(a.name).peek_filename().localeCompare(new Path().from_string(b.name).peek_filename()))
    render_files(sortedItems);
  } catch(e) {
    console.error('Render error:', e);
  }

}

// FrontEnd

function render_files(items) {
  const fileBody = document.getElementById('fileBody');
  const thead =document.getElementById("fileTableThead")
  thead.innerHTML="";
  //Creat head
  const headtr = document.createElement('tr');

    const cbTd = document.createElement('th');
    cbTd.className = 'check-col';
    headtr.appendChild(cbTd);
    
    const nameTd = document.createElement('th');
    nameTd.textContent = "文件名";
    nameTd.style.width ="auto";
    nameTd.className = "th";
    headtr.appendChild(nameTd);

    const sizeTd = document.createElement('th');
    sizeTd.textContent = "大小";
    sizeTd.style.width ="160px";
    sizeTd.className = "th";
    headtr.appendChild(sizeTd);

    const createdatTd=document.createElement('th');
    createdatTd.textContent ="创建时间";
    createdatTd.style.width ="160px";
    createdatTd.className = "th";
    headtr.appendChild(createdatTd);

    const modifiedatTd=document.createElement('th');
    modifiedatTd.textContent ="最后修改于";
    modifiedatTd.style.width ="160px";
    modifiedatTd.className = "th";
    headtr.appendChild(modifiedatTd);

    const creatorTd=document.createElement("th");
    creatorTd.style.width ="160px";
    creatorTd.className = "th";
    creatorTd.textContent="创建者";
    headtr.appendChild(creatorTd);

    const modifierTd=document.createElement("th");
    modifierTd.textContent="最后修改者";
    modifierTd.style.width ="160px";
    modifierTd.className = "th";
    headtr.appendChild(modifierTd);

  thead.appendChild(headtr);

    fileBody.innerHTML = '';
    // console.log('items:', items); //
    
    if (!items || items.length === 0) {
      fileBody.innerHTML = '<tr><td colspan="7" style="text-align:center;padding:40px;color:#999;">此目录为空</td></tr>';
      return;
    }
  
  items.forEach(item => {
    const tr = document.createElement('tr');

    const cbTd = document.createElement('td');
    cbTd.className = 'check-col';
    tr.appendChild(cbTd);

    const nameTd = document.createElement('td');

    const iconContainer = document.createElement('img');
    iconContainer.width =18;
    iconContainer.height=18; 
    iconContainer.src = getIcon(item);
    const fileName = new Path().from_string(item.name).peek_filename();
    const textSpan = document.createElement('span');
    textSpan.textContent = ' ' + fileName; // 加个空格
    // wrapper.append(iconContainer, textSpan);
    iconContainer.style.verticalAlign = 'middle';
    textSpan.style.verticalAlign = 'middle';
    nameTd.append(iconContainer,textSpan);
    // nameTd.textContent = `${icon} ${new Path().from_string(item.name).peek_filename()}`;
    nameTd.addEventListener('contextmenu',(e)=>{e.preventDefault();handleItemMenu(item,e)});
    nameTd.addEventListener('click', () => handleItemClick(item));
    nameTd.addEventListener('dblclick', () => handleItemDblClick(item));
    tr.appendChild(nameTd);


    const sizeTd = document.createElement('td');
    if (item.is_directory) {sizeTd.textContent = "";}
    else {sizeTd.textContent = FMT.fmt_size(item.size);};
    
    tr.appendChild(sizeTd);

    const createdatTd=document.createElement('td');
    createdatTd.textContent =FMT.fmt_time(item.created_at);
    tr.appendChild(createdatTd);

    const modifiedatTd=document.createElement('td');
    modifiedatTd.textContent =FMT.fmt_time(item.modified_at);
    tr.appendChild(modifiedatTd);

    const creatorTd=document.createElement("td");
    creatorTd.textContent=item.creator;
    tr.appendChild(creatorTd);

    const modifierTd=document.createElement("td");
    modifierTd.textContent=item.last_modifier;
    tr.appendChild(modifierTd);


    // tr.addEventListener('click', () => handleItemClick(item));
    
    if (!item.is_directory){tr.draggable = true;}

    fileBody.appendChild(tr);
  });
}

// Handler

function handleItemClick(item){
  if (item.is_directory){
    currentPath = currentPath.push_self(new Path().from_string(item.name).peek_filename())
    refreshDir();
  }else{
    return;
  }

}


function handleItemDblClick(item){
  if (item.is_directory){
    return;
  }else{
    previewHandler(item);
  }

}


async function previewHandler(item) {
  
  
  
  const body = construct_rq_body(item.name, item.is_directory, null, null);
  const res = await API.downloadFile(FILEBASE, body, TOKEN);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  const blob = await res.blob();
  Render.previewFile(blob, new Path().from_string(item.name).peek_filename());
}


function handleItemMenu(item,event){
  const existing = document.querySelector('.context-menu');
  existing?.remove();
  const menu = document.createElement('div');
  menu.className = 'context-menu';
  menu.style.left = event.clientX + 'px';
  menu.style.top = event.clientY + 'px';
  const actions = [
    { label: '重命名', action: () => renameHandler(item) },
    { label: '复制到', action: () => copyHandler(item) },
    { label: '移动到', action: () => moveHandler(item) },
    { label: '详细信息', action: () => fileDetailHandler(item) },
    { label: '下载', action: () => downloadHandler(item)},
    { label: '修改创建人', action: () => chown(item) },
    { label: '删除', action: () => DeleteHandler(item)},
  ];

  actions.forEach(({ label, action }) => {
    const btn = document.createElement('button');
    btn.className = "menu-item"
    btn.textContent = label;
    btn.onclick = () => {
      action();
      menu.remove();
    };
    menu.appendChild(btn);
  });
  document.body.appendChild(menu);
  const closeOnClick = (e) => {
    if (!menu.contains(e.target)) {
      menu.remove();
      document.removeEventListener('click', closeOnClick);
    }
  };
  setTimeout(() => document.addEventListener('click', closeOnClick), 0);
}




function goUp(){
  currentPath=currentPath.pop_self();
  refreshDir();
}


function inputDialog(title = '输入', placeholder = '',validate_path,is_dir=true,suffix) {
  return new Promise(resolve => {
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    const box = document.createElement('div');
    box.className = 'modal-box';
    box.innerHTML = `
      <h3 class="modal-title">${title}</h3>
      
      <div style="display: flex; align-items: baseline;flex-direction:row;">
        <input type="text" class="modal-input" placeholder="${placeholder}" autofocus>
        <div id="file_suffix"></div>
      </div>
      <div class="modal-btn-container">
        <button class="modal-btn" data-action="cancel">取消</button>
        <button class="modal-btn" data-action="confirm">确认</button>
      </div>
      <div class="modal-error" id = "input-dialog-err"> </div>
    `;
    overlay.appendChild(box);
    document.body.appendChild(overlay);
    const input = box.querySelector('.modal-input');
    const err = box.querySelector('#input-dialog-err');
    const confirmBtn = box.querySelector('[data-action="confirm"]');
    const cancelBtn = box.querySelector('[data-action="cancel"]');
    const suffix_div = box.querySelector('#file_suffix')
    suffix_div.textContent=suffix
    
    const close = (value) => {
      overlay.remove();
      resolve(value);
    };

    confirmBtn.addEventListener('click', () => {
    const val = input.value.trim();
    
    if (validate_path){    
        const errMsg = validate_path(val,is_dir);
        if (!errMsg.valid) {
          err.textContent=errMsg.msg;
          return;
        }
      }
    close(val);
    });

    cancelBtn.addEventListener('click', () => close(null));
    overlay.addEventListener('click', e => {
      if (e.target === overlay) close(null);
    });
    input.addEventListener('keydown', e => {
      if (e.key === 'Enter') confirmBtn.click();
      if (e.key === 'Escape') cancelBtn.click();
    });
    input.focus();
  });
}

function fileDetailHandler(item){
const overlay = document.createElement('div');
  // overlay.style.cssText = `
  //   position:fixed;inset:0;background:rgba(0,0,0,0.4);
  //   display:flex;align-items:center;justify-content:center;z-index:9999
  // `;
  overlay.className="modal-overlay"
  const modal = document.createElement('div');
  // modal.style.cssText = `
  //   background:#fff;border-radius:8px;padding:24px;min-width:300px;max-width:500px;
  //   box-shadow:0 4px 20px rgba(0,0,0,0.2)
  // `;
  modal.className="modal-box"
  const list = document.createElement('dl');
  list.style.cssText = 'margin:0';
  const keyMap = {
      id:"索引",
      name: '文件名',
      parent_name:"父文件夹",
      is_directory:"是否为文件夹",
      size: '大小',
      content_type:"文件类型",
      md5:"MD5",
      created_at: '创建时间',
      modified_at: '最后修改时间',
      creator: '创建者',
      last_modifier: '最后修改者'
    };

  for (let [key, value] of Object.entries(item)) {
    const dd = document.createElement('dd');
    if (key == "parent_name"){
      value=`/${value}`
    }
    
    dd.textContent = `${keyMap[key]} : ${value}`;
    dd.style.cssText = 'margin:0 0 8px 0';
    list.appendChild(dd);
  }
  modal.appendChild(list);
  const closeBtn = document.createElement('button');
  closeBtn.textContent = '关闭';
  closeBtn.className = "modal-btn";
  // closeBtn.style.cssText = 'margin-top:16px;padding:6px 16px;border:1px solid var(--white);border-radius:4px;cursor:pointer;width:100%;';
  closeBtn.onclick = () => overlay.remove();
  modal.appendChild(closeBtn);
  overlay.appendChild(modal);
  // overlay.appendChild(closeBtn);
  document.body.appendChild(overlay);
  overlay.onclick = (e) => { if (e.target === overlay) overlay.remove(); };

}


async function mkdir(){
  const name = await inputDialog("请输入名称"," ",is_path_valid,true);
  let newname = currentPath.push_clone(name).to_string_no_root()
  const body=construct_rq_body(currentPath.slice(1),true,newname,null);
  console.log(body);
  try {
    const res =await API.uploadFile(FILEBASE,body, TOKEN);
    // console.log(res);
    refreshDir(); 
  } catch (err) {
    console.error('mkdir failed', err);
    alert('创建失败: ' + (err.message || '未知错误'));
  }
  
}

async function logout() {
  // const body=construct_rq_body(localStorage.getItem("username"),TOKEN);
  const res = await API.logout(BASE,localStorage.getItem("token"));
  if (res.ok){
    localStorage.setItem("token" , res.token);
    window.location.href="/login";
  }else{
    console.log("Fail")
  }

}


async function list_log(){
  currentMod = "log";
  const res = await API.list_log(BASE,100,localStorage.getItem("token"));
  if (res.ok) {
    render_log(await res.json())
  }else{
    console.log("Fail")
  }
}


async function  render_log(items) {
  const fileBody = document.getElementById('fileBody');
  fileBody.innerHTML = '';
  console.log('items:', items); //
  
  const thead =document.getElementById("fileTableThead")
  thead.innerHTML="";
  //Creat head
  const headtr = document.createElement('tr');

    const nameTd = document.createElement('th');
    nameTd.textContent = "用户";
    // nameTd.style ="width:;"
    headtr.appendChild(nameTd);

    const actionTd = document.createElement('th');
    actionTd.textContent = "动作";
    // nameTd.style ="width:160px;"
    headtr.appendChild(actionTd);

    const statusTd=document.createElement('th');
    statusTd.textContent ="执行状态";
    // createdatTd.style ="width:160px;"
    headtr.appendChild(statusTd);

    const filepathTd=document.createElement('th');
    filepathTd.textContent ="文件路径";
    // modifiedatTd.style ="width:160px;"
    headtr.appendChild(filepathTd);

    const argsTd=document.createElement("th");
    // createdatTd.style ="width:160px;"
    argsTd.textContent="传入参数";
    headtr.appendChild(argsTd);

    const timeTd=document.createElement("th");
    timeTd.textContent="时间";
    // modifierTd.style ="width:160px;"
    headtr.appendChild(timeTd);

  thead.appendChild(headtr);

  if (!items || items.length === 0) {
    fileBody.innerHTML = '<tr><td colspan="7" style="text-align:center;padding:40px;color:#999;">日志为空</td></tr>';
    return;
  }

  items.forEach(item => {
    const tr = document.createElement('tr');

    const  userTd = document.createElement('td');
     userTd.textContent = `${item.user}`;

    const actionTd = document.createElement('td');
    actionTd.textContent = item.action;
    
    const timeTd=document.createElement('td');
    timeTd.textContent =FMT.fmt_time(item.time);
    
    const statusTd=document.createElement('td');
    statusTd.textContent =item.status;
    
    const filepathTd=document.createElement("td");
    filepathTd.textContent=`/${item.filepath}`;
    
    const argsTd=document.createElement("td");
    argsTd.textContent=item.args || "无";

    tr.appendChild( userTd);
    tr.appendChild(actionTd);
    tr.appendChild(statusTd);
    tr.appendChild(filepathTd);
    tr.appendChild(argsTd);
    tr.appendChild(timeTd);

    fileBody.appendChild(tr);
  });
}


async function renameHandler(item) {
  const suffix = new Path().from_string(item.name).get_suffix();
  const newname = await inputDialog("请输入新名称"," ",is_path_valid,item.is_directory,suffix);
  

  const body = construct_rq_body(new Path().from_string(item.name).to_string_no_root(),item.is_directory,currentPath.push_clone(newname).add_suffix(suffix).to_string_no_root(),null)
  console.log(body)
  const res = await API.renameFile(FILEBASE,body,TOKEN);
  refreshDir();
}


async function downloadHandler(item){
  const body = construct_rq_body(item.name, item.is_directory, null, null);
  const res = await API.downloadFile(FILEBASE, body, TOKEN);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  // 从响应头取 MD5（后端自定义头）
  const md5Base64 = await res.headers.get('X-Content-MD5');
  const blob = await res.blob();
  const buffer = await blob.arrayBuffer();
  // 验证 MD5
  let tmp;
  if (md5Base64) {
    const md5Compute = await calcFileMD5(buffer);
    // console.log(md5Base64);
    // console.log(md5Compute);
    if (md5Compute !== md5Base64) {
        tmp = window.confirm(`检测到MD5值不匹配，服务器记录的MD5为：${md5Base64}，实际收到的文件MD5值为：${md5Compute}。这说明可能存在信息损失。是否放弃本次下载？`)
    }
  }
  if(tmp){return;}
  
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = new Path().from_string(item.name).peek_filename() || 'download';
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function base64ToHex(b64) {
  const raw = atob(b64);
  let hex = '';
  for (let i = 0; i < raw.length; i++) {
    hex += raw.charCodeAt(i).toString(16).padStart(2, '0');
  }
  return hex;
}

function SelectDirDialog(title="选择目标文件夹"){
  return new Promise(resolve => {
    
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    const box = document.createElement('div');
    box.className = 'modal-box';
    box.innerHTML = `
      <h3 class="modal-title">${title}</h3>
      <div class="current-path" id="currentPathDisplay">当前已选择的文件夹：/</div>
      <div class="file-container" id="semiFileContainer">
        <table class="file-table" id="semiFileTable">
          <tbody id="semiFileBody"></tbody>
        </table>
      </div>
      <div class="modal-btn-container">
        <button class="modal-btn" data-action="cancel">取消</button>
        <button class="modal-btn" data-action="confirm">确认</button>
      </div>
      <div class="modal-error" id="input-dialog-err"> </div>
    `;
    overlay.appendChild(box);
    document.body.appendChild(overlay);
    const semiFileBody = box.querySelector('#semiFileBody');
    const pathDisplay = box.querySelector('#currentPathDisplay');
    const errorDiv = box.querySelector('#input-dialog-err');
    const confirmBtn = box.querySelector('[data-action="confirm"]');
    const cancelBtn = box.querySelector('[data-action="cancel"]');
    let semi_currentPath = new Path();
    let existingFiles;
    // 加载指定路径下的文件夹列表
    //path:Path
    async function loadDir(path) {
      try {
        errorDiv.textContent = '';
        semiFileBody.innerHTML = '<tr><td>加载中...</td></tr>';
        pathDisplay.textContent = `当前已选择的文件夹为 ${path.to_string_with_root()}`;
        const body = construct_rq_body(path.to_string_no_root(), true, null, null);
        const response = await API.list_dir(FILEBASE, body, TOKEN);
        const data = await response.json();
        // 假设 data.items 或 data 本身就是数组
        const items = Array.isArray(data) ? data : data.items || [];
        existingFiles = items.filter(item => !item.is_directory).map(item => new Path().from_string(item.name).peek_filename());
        const dirs = items.filter(item => item.is_directory);
        // 构建表格行
        
        let html = '';
        // 如果不是根目录，添加 "返回上级" 行
        if (path !== "/") {
          html += `<tr class="dir-row parent-dir">
            <td>⬆ 返回上级</td>
          </tr>`;
        }
        dirs.forEach(item => {
          html += `<tr class="dir-row" data-name="${item.name}">
            <td>📁 ${new Path().from_string(item.name).peek_filename()}</td>
          </tr>`;
        });
        semiFileBody.innerHTML = html || '<tr><td>空目录</td></tr>';
      } catch (err) {
        errorDiv.textContent = '加载失败: ' + err.message;
        semiFileBody.innerHTML = '';
      }
    }

    semiFileBody.addEventListener('click', async (e) => {
      const row = e.target.closest('tr');
      if (!row) return;
      const isParent = row.classList.contains('parent-dir');
      if (isParent) {
        semi_currentPath = semi_currentPath.pop_self()
        loadDir(semi_currentPath);
        
      } else {

        semi_currentPath = new Path().from_string(row.dataset.name);//????????????????
        loadDir(semi_currentPath);
      }
    });
    // 确认：返回最后选中的文件夹名字
    confirmBtn.addEventListener('click', () => {
     const name = semi_currentPath.to_string_no_root();
      close([name,existingFiles]);
    });
    cancelBtn.addEventListener('click', () => close(null));
    overlay.addEventListener('click', e => {
      if (e.target === overlay) close(null);
    });
    // 键盘事件
    document.addEventListener('keydown', function handler(e) {
      if (e.key === 'Enter') {
        confirmBtn.click();
        e.preventDefault();
      } else if (e.key === 'Escape') {
        cancelBtn.click();
        e.preventDefault();
      }
    }, { once: true });
    // 关闭并清理
    function close(value) {
      overlay.remove();
      resolve(value);
    }
    // 初始加载根目录
    loadDir(semi_currentPath);
  });
}

async function moveHandler(item){
  // console.log(item.name)
  let rx = await SelectDirDialog();
  
  const target_dir =rx[0];
  const existingFiles = rx[1];
  
  // if ()
  // console.log(target_dir)
  // console.log(existingFiles)
  
  if (existingFiles.includes(new Path().from_string(item.name).peek_filename())){
    alert(`目标文件夹 ${target_dir} 下已有一个名为 ${new Path().from_string(item.name).peek_filename()}的文件！本次移动已中止！请尝试重命名后再重新操作。`)
    return;
  }

  let tmp;
  if (item.is_directory){
    tmp = window.confirm(`你正在把文件夹 ${new Path().from_string(item.name).peek_filename()} 移动到 ${new Path().from_string(target_dir).to_string_with_root()} ，子目录与文件都会移动！`)
  } else{
    tmp = window.confirm(`你正在把文件 ${new Path().from_string(item.name).peek_filename()} 移动到 ${new Path().from_string(target_dir).to_string_with_root()} ，确定吗？`)
  }
  if (tmp){
  let newname= new Path().from_string(target_dir).push_self(new Path().from_string(item.name).peek_filename()).to_string_no_root()
  const body = construct_rq_body(item.name,item.is_directory,newname,null)
  console.log(body)
  const res = await API.renameFile(FILEBASE,body,TOKEN);
  console.log(res)
  refresh()
  }
}

async function copyHandler(item) {
  let rx = await SelectDirDialog("选择要复制到哪个文件夹下");
  const target_dir =rx[0];
  const existingFiles = rx[1];
  
  // if ()
  // console.log(target_dir)
  // console.log(existingFiles)
  if (existingFiles.includes(new Path().from_string(item.name).peek_filename())){
    alert(`目标文件夹 ${target_dir} 下已有一个名为 ${new Path().from_string(item.name).peek_filename()}的文件！本次复制已中止！请尝试重命名后再重新操作。`)
    return;
  }

  let tmp;
  if (item.is_directory){
    tmp = window.confirm(`你正在把文件夹 ${new Path().from_string(item.name).peek_filename()} 复制到 ${new Path().from_string(target_dir).to_string_with_root()} ，子目录与文件都会复制！`)
  } else{
    tmp = window.confirm(`你正在把文件 ${new Path().from_string(item.name).peek_filename()} 复制到 ${new Path().from_string(target_dir).to_string_with_root()} ，确定吗？`)
  }
  
  if (tmp){
  let newname= new Path().from_string(target_dir).push_self(new Path().from_string(item.name).peek_filename()).to_string_no_root()
  const body = construct_rq_body(item.name,item.is_directory,newname,null)
  console.log(body)
  const res = await API.copyFile(FILEBASE,body,TOKEN);
  console.log(res)
  refresh()
  }
  
}


async function DeleteHandler(item){
  let tmp;
  if (item.is_directory){
    tmp = window.confirm("确定要删除此文件夹吗？所有子目录和文件都会被删除！文件不可找回，你的操作会被日志记录！")
  } else{
    tmp = window.confirm("确定要删除此文件吗？文件不可找回，你的操作会被日志记录！")
  }
  if (tmp){
    const body = construct_rq_body(item.name,item.is_directory,null,null)
    const res = await API.deleteFile(FILEBASE,body,TOKEN);
    console.log(res)
  }else{
    return;
  }
  refresh();
  
}

function getUniqueFileName(originalName) {
  
  // console.log(currentItems)

  let uploadedFileNames = currentItems.map(item => item.name);
  // console.log("UploadedNames")
  // console.log(uploadedFileNames)
  let name = originalName;
  let counter = 1;
  while (uploadedFileNames.includes(name)) {
    const dotIndex = originalName.lastIndexOf('.');
    if (dotIndex === -1) {
      name = `${originalName}_${counter}`;
    } else {
      name = `${originalName.slice(0, dotIndex)}_${counter}${originalName.slice(dotIndex)}`;
    }
    counter++;
  }
  // console.log(originalName)
  // console.log(name)
  return name;
}

async function uploadFile(current_path,is_dir) {
  console.log(current_path)
  // 1. 创建隐藏的 file input 并触发选择
  const fileInput = document.createElement('input');
  fileInput.type = 'file';
  fileInput.multiple = true;
  fileInput.style.display = 'none';
  
  if (is_dir){
    fileInput.directory = true;
    fileInput.webkitdirectory = true;}

  document.body.appendChild(fileInput);
  const files = await new Promise((resolve) => {
      fileInput.addEventListener('change', (e) => {
      resolve(e.target.files);
      fileInput.remove();
      });
      fileInput.click();
  });
  const selectedDir = current_path;
  if (!files.length) return;

  const fileList=Array.from(files);
  //文件夹模式
  if(is_dir){
    const dirSet = new Set();
    // const fileList = Array.from(files);
    
    //Colleting all dir names
    for (const file of fileList) {
        const relPath = file.webkitRelativePath;  // 此时是 "trip/photo.jpg"
        // const fullPath =`${selectedDir}/${relPath}`  // 变成 "a/trip/photo.jpg"
        // console.log(fullPath);
        const parts = relPath.split('/');
        for (let i = 0; i < parts.length - 1; i++) {
            dirSet.add(parts.slice(0, i + 1).join('/'));
        }
        // console.log(dirSet)
      }
    // for dir in dirSet{}
    
    
    const sortedDirs = Array.from(dirSet).sort((a, b) => a.split('/').length - b.split('/').length);
    console.log(sortedDirs)
    console.log(files)
    for (const dir of sortedDirs) {
            
        const dirPath = current_path.push_clone(dir).to_string_no_root();
        console.log(dirPath)
        let uploadedFileNames = currentItems.map(item => item.name);
        if (uploadedFileNames.includes(dirPath)){
          alert(`当前目录下已有一个有 ${dirPath} 文件夹！`)
          return;
        }
        const body = construct_rq_body(current_path,true,dirPath,null)

        const res = await API.uploadFile(FILEBASE, body, TOKEN);
        // console.log(res)
      }
    
    for (const file of files) {
    const [contentBase64, arrayBuffer] = await Promise.all([
      readFileAsBase64(file),
      readFileAsArrayBuffer(file)
    ]);
    const md5 = await calcFileMD5(arrayBuffer);
    const uniqueName = getUniqueFileName( current_path.push_clone(file.webkitRelativePath).to_string_no_root());
    const body =construct_rq_body(uniqueName,false,md5,contentBase64)

    const res = await API.uploadFile(FILEBASE,body,TOKEN);
    // 调用 API 上传
    // console.log(uniqueName)
    }
  

  //文件模式
  }else{
    console.log("Uploading files")
    for (const file of files) {
    const [contentBase64, arrayBuffer] = await Promise.all([
      readFileAsBase64(file),
      readFileAsArrayBuffer(file)
    ]);
    const md5 = await calcFileMD5(arrayBuffer);
    const uniqueName = getUniqueFileName(current_path.push_clone(file.name).to_string_no_root());
    const body =construct_rq_body(uniqueName,false,md5,contentBase64)
    // console.log(uniqueName)
    const res =await API.uploadFile(FILEBASE,body,TOKEN)
    console.log(res)

  }}

  refresh()
}

async function chown(item){
  const new_creator = await inputDialog("输入新的创建人"," ", is_path_valid,true);
  const body = construct_rq_body(item.name,item.is_directory,new_creator,null);
  const res = await API.changeCreator(FILEBASE,body,TOKEN)
  refresh();
}




//Utils

function readFileAsBase64(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = reader.result;
      const base64 = dataUrl.split(',')[1];
      resolve(base64);
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

function readFileAsArrayBuffer(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = () => reject(reader.error);
    reader.readAsArrayBuffer(file);
  });
}

async function calcFileMD5(arrayBuffer) {
  const spark = new SparkMD5.ArrayBuffer();
  spark.append(arrayBuffer);
  return spark.end(); 
}

function getIcon(item) {
  if (item.is_directory) return '/static/icons/dir.svg';
  const map = {
    "csv":"/static/icons/csv.svg",
    "dir":"/static/icons/dir.svg",
    "docx":"/static/icons/docx.svg",
    "other":"/static/icons/other.svg",
    "pdf":"/static/icons/pdf.svg",
    "png":"/static/icons/png.svg",
    "pptx":"/static/icons/pptx.svg",
    "rar":"/static/icons/rar.svg",
    "txt":"/static/icons/txt.svg",
    "xlsx":"/static/icons/xlsx.svg",
    "zip":"/static/icons/zip.svg",
    "mp3":"/static/icons/audio.svg",
  }
  const ext = (item.name || '').split('.').pop().toLowerCase();
  return map[ext] || "/static/icons/other.svg";
}

function show_username(){
  const uname = localStorage.getItem("username");
  document.getElementById("sidebar-username").textContent = uname ;
}

function is_path_valid(name,is_dir){
  if (!name ){
    return {valid:false,msg:"名称不能为空"}
  }
  const forbiddenChars = /[\\/:*?"<>|]/;


  if (forbiddenChars.test(name)) {
    return {valid:false,msg:'名称不能包含字符：\\ / : * ? " < > |'};
  }
  if (is_dir && /[. ]$/.test(name)) {
    return { valid: false, msg: '目录名不能以点或空格结尾' };
  }

  return {valid:true,msg:'有效的新名称'}
}

function construct_rq_body(filename,is_dir,args,bytes){
  const body = {
    filename:filename,
    is_dir:is_dir,
    args:args,
    bytes:bytes,
  }
  return body;
}


function refresh(dir=currentPath){
  if (currentMod == "log"){
    list_log()
  }
  if (currentMod == "files"){
    refreshDir(dir)
  }
}

function check_login(){
  const token = localStorage.getItem("token");
  if (!token|| token === "undefined" || token === "null"){
    window.location.href = "/login";
  }
}


// ===== 初始化 =====
check_login()
show_username()
await refreshDir();

document.getElementById('parentBtn').addEventListener('click', goUp);
document.getElementById('refreshBtn').addEventListener('click', ()=>refresh(currentPath));
document.getElementById('mkdir').addEventListener('click', mkdir);
document.getElementById('logout').addEventListener('click',logout);
document.getElementById('log').addEventListener('click',list_log);
document.getElementById('mainPage').addEventListener('click',()=> {currentPath = new Path();currentMod="files";refresh();})
document.getElementById('upLoadFiles').addEventListener('click',()=>uploadFile(currentPath,false))
document.getElementById("upLoadFolder").addEventListener('click', () => uploadFile(currentPath,true))
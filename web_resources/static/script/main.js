import * as API from "./api.js";
import { Path,FMT } from "./utils.js";
import {Render} from "./render.js";
import "./spark_md5.js";
const appState = {
    username:localStorage.getItem("username"),
    state:"zone",
    currentZone:"",
    currentDir:"",
    token:localStorage.getItem("token"),
    allZones:null,
    allUsers:null,
    currentFiles: null,
    currentLords:null, 
    
    update_token:function(token){
        localStorage.setItem("token",token);
        this.token=token;
    },

    dir_pop:function(){
        const segment = this.currentDir.split("/");
        this.currentDir = segment.slice(0,-1).join("/");
    },
    dir_push:function(name){
        const segment = this.currentDir.split("/");
        this.currentDir = segment.push(name).join("/");
    }
    


}



const iconMap = {
    csv:"/static/icons/csv.svg",
    dir:"/static/icons/dir.svg",
    docx:"/static/icons/docx.svg",
    other:"/static/icons/other.svg",
    pdf:"/static/icons/pdf.svg",
    png:"/static/icons/png.svg",
    pptx:"/static/icons/pptx.svg",
    rar:"/static/icons/rar.svg",
    txt:"/static/icons/txt.svg",
    xlsx:"/static/icons/xlsx.svg",
    zip:"/static/icons/zip.svg",
    mp3:"/static/icons/audio.svg",
}



function FviwerInit() {
    const layout =  document.createElement("div");
    layout.className ="layout";
    
    const sidebar =  document.createElement("div");
    sidebar.className ="sidebar";
    
    const sidebarTitle =  document.createElement("div");
    sidebarTitle.className = "sidebar-title";
    
    const logo =  document.createElement("a");
    logo.href = "https://guanbinwu.github.io";
    logo.target = "_blank";
    logo.rel = "noopener";
    logo.style= "color: inherit; text-decoration: none;";
    logo.textContent ="FileViewer v0.1.0";
    sidebarTitle.append(logo);
    
    const welcome =  document.createElement("div");
    welcome.className="sidebar-info";
    welcome.textContent= "欢迎,";
    
    const username =  document.createElement("div");
    username.className ="sidebar-info";
    username.id ="sidebar-username";
    username.textContent = localStorage.getItem("username");

    const sidebar_item_container =  document.createElement("div");
    sidebar_item_container.className="sidebar-item-container";
    sidebar_item_container.id = "sidebar-item-container";
    
    const sidebar_footer =  document.createElement("div");
    sidebar_footer.className = "sidebar-footer";
    sidebar_footer.innerHTML = "<p>单击：进入目录</p><p>双击：预览文件</p><p>右键：打开菜单</p>"
    
    const bless = document.createElement("p");
    bless.style.paddingTop="20px";
    bless.textContent = "🖖Live long and prosper🖖"

    sidebar_footer.append(bless)
    const main =  document.createElement("div");
    main.className="main";
    main.id="main";
    
    sidebar.append(sidebarTitle);
    sidebar.append(welcome);
    sidebar.append(username);
    sidebar.append(sidebar_item_container);
    sidebar.append(sidebar_footer);

    layout.append(sidebar);
    layout.append(main);
    return layout;
}

function sidebarFixedItems(){
    const btn1 = document.createElement("div")
    btn1.className = "sidebar-item";
    btn1.id = "setting";
    btn1.textContent = "设置";
    btn1.addEventListener("click",(e)=>{
        console.log("设置");
    });

    const btn2 = document.createElement("div")
    btn2.className = "sidebar-item";
    btn2.id = "log";
    btn2.title="查看最新的100条用户记录"
    btn2.textContent = "查看日志";
    btn2.addEventListener("click",async (e)=>{
        // console.log("日志");
        await logState();
    });

    const btn3 = document.createElement("div")
    btn3.className = "sidebar-item";
    btn3.id = "logout";
    btn3.textContent = "登出";
    btn3.addEventListener("click",async(e)=>{
        // console.log("登出");
        const res = await API.accounts_logout(appState.token);
        const tmp = await res.text();
        appState.update_token(tmp);
        // console.log(await res.text());
        await check_session();
        
    });
    return [btn1,btn2,btn3]
}

async function zoneState(){
    const zones =await API.zone_list(appState.token);
    appState.allZones=zones;

    

    const mainBody = document.getElementById('main');
    const sidebarItems =document.getElementById("sidebar-item-container");
    sidebarItems.innerHTML=``;
    mainBody.innerHTML= ``;
    mainBody.append(renderZones(zones));

    const createZone = document.createElement("div");
    createZone.className = "sidebar-item"
    createZone.id = "createZone"
    createZone.textContent="新建仓库";

    createZone.addEventListener("click",async ()=>{
        const tmp = await ZoneDialog("请输入新的仓库名称",null,[],null);
        if (!tmp){return;}
        const res = await API.zone_create(appState.token,tmp.name,tmp.lords);
        await zoneState()
    })

    sidebarItems.append(createZone,...sidebarFixedItems());
    const allSize = await API.zone_size(appState.token);
    
    if (allSize.ok){
        const sizeInfoContainer = document.createElement("div");
        sizeInfoContainer.width="100%"
        const sizes =await allSize.json();   
        const t = document.createElement("p");
        const p = document.createElement("progress");
        p.flex=1;
        t.width="100%";
        p.width="100%";
        t.textContent = `${FMT.fmt_size(sizes[0])}/${FMT.fmt_size(sizes[2])}`
        p.max=1;
        // p.value=sizes[0]/sizes[2];
        p.value=0.5;
        sizeInfoContainer.append(p,t)
        sidebarItems.append(sizeInfoContainer);
    }
    

}

async function logState() {
    
    const items = await API.log(appState.token);
    const mainBody = document.getElementById('main');
    const sidebarItems =document.getElementById("sidebar-item-container");
    mainBody.innerHTML="";
    sidebarItems.innerHTML = "";

    const table = document.createElement('table');
    table.className = "file-table";
    table.id = "logTable";

    const thead = document.createElement("thead");
    thead.id = "logTableThead";

    const tbody =document.createElement("tbody");
    tbody.id = "logTableBody";
    
    const headtr = document.createElement('tr');

    const nameTd = document.createElement('th');
    nameTd.textContent = "用户";
    nameTd.style.width ="160px";
    nameTd.className = "th";
    
    const actionTd = document.createElement('th');
    actionTd.textContent = "动作";
    actionTd.style.width ="160px";
    actionTd.className = "th";
    
    const filenameTd=document.createElement('th');
    filenameTd.textContent ="操作的文件对象";
    filenameTd.style.width ="auto";
    filenameTd.className = "th";
    
    const timeTd=document.createElement('th');
    timeTd.textContent ="时间";
    timeTd.style.width ="360px";
    timeTd.className = "th";
    
    const argsTd=document.createElement("th");
    argsTd.style.width ="auto";
    argsTd.className = "th";
    argsTd.textContent="参数";

    const resultTd=document.createElement("th");
    resultTd.style.width ="auto";
    resultTd.className = "th";
    resultTd.textContent="结果";

    headtr.append(nameTd,actionTd,filenameTd,timeTd,argsTd,resultTd);

    thead.appendChild(headtr);

    items.forEach(item => {
        const tr = document.createElement('tr');
        const nameTd = document.createElement('td');
        nameTd.textContent = item.user;        
        const actionTd = document.createElement('td');
        actionTd.textContent = item.action;
        
        const filenameTd=document.createElement('td');
        filenameTd.textContent = item.filepath;
        
        const timeTd=document.createElement('td');
        timeTd.textContent =item.time;
        
        const argsTd=document.createElement("td");
        argsTd.textContent =item.args;

        const resultTd=document.createElement("td");
        resultTd.textContent =item.status;

        tr.append(nameTd,actionTd,filenameTd,timeTd,argsTd,resultTd);
        tbody.append(tr);
    });
  
    table.append(thead,tbody);
    mainBody.append(table,noMoreContent());
    sidebarItems.append(backZone());
    sidebarItems.append(...sidebarFixedItems());

}


function backZone(){
    const btn = document.createElement("div");
    btn.className = "sidebar-item";
    btn.id = "backZone";
    btn.textContent = "返回仓库";
    btn.addEventListener("click",async (e)=>{
        await zoneState();
    })
    return btn

}

function backTree(){
    const btn = document.createElement("div");
    btn.className = "sidebar-item";
    btn.id = "backTree";
    btn.textContent = "查看目录树";
    btn.addEventListener("click",async (e)=>{
        await TreeState(appState.currentZone)
    });
    return btn
}

async function TreeState(zoneName) {
    appState.currentZone=zoneName;
    const mainBody = document.getElementById('main');
    const sidebarItems =document.getElementById("sidebar-item-container");
    sidebarItems.innerHTML=``;
    sidebarItems.append(backZone(),...sidebarFixedItems())
    
    const items = await API.zone_tree(appState.token,appState.currentZone);

    mainBody.innerHTML = ``;
    if (items.length ==0){
        mainBody.append(emptyTree())
    }else{
        const tree = itemsToTree(items);
        mainBody.append(renderTree(tree));
    }

    const treeContainer = document.getElementById('fileDirTree');

    treeContainer.addEventListener("click",async (e)=>{
        const El = event.target.closest(".tree-node");
        if (!El) return;
        const name = El.dataset.name;
        await FileState(name);
    })
}



function toolBar(refreshfn,cwd){
    const toolBar = document.createElement("div");
    toolBar.className = "toolbar";
    toolBar.id = "toolbar";
    
    const newDir = document.createElement("button");
    newDir.className = "btn";
    newDir.id = "mkdir";
    newDir.textContent ="📁 新建目录";

    const span = document.createElement("span");
    span.textContent =" 你目前位于➤";
    span.style.color = "var(--white)"

    const refresh = document.createElement("button");
    refresh.className = "btn";
    refresh.id = "refreshBtn";
    refresh.textContent ="⟳ 刷新页面";

    const goUp = document.createElement("button");
    goUp.className = "btn";
    goUp.id = "parentBtn";
    goUp.textContent ="⏎ 返回上级";

    const upload = document.createElement("button");
    upload.className = "btn";
    upload.id = "upload";
    upload.textContent ="⬆ 上传文件";
    upload.title ="你可以上传一个或多个文件到当前目录下，但不能上传文件夹";

    const uploadDir = document.createElement("button");
    uploadDir.className = "btn";
    uploadDir.id = "uploadDir";
    uploadDir.textContent ="⬆ 上传目录";
    uploadDir.title ="你可以上传一个文件夹到当前目录，他的子文件/目录都会跟随上传。";

    const multiSelect = document.createElement("button");
    multiSelect.className = "btn";
    multiSelect.id = "multiSelectBtn";
    multiSelect.textContent ="☐ 批量下载";

    const confirmSelect = document.createElement("button");
    confirmSelect.style.backgroundColor="var(--blue)"
    confirmSelect.className = "btn";
    confirmSelect.id = "cancelSelectBtn";
    confirmSelect.textContent ="✔ 确认下载";
    confirmSelect.hidden = true;

    goUp.addEventListener("click",async(e)=>{
        appState.dir_pop();
        await refreshfn(appState.currentDir);
    })

    upload.addEventListener("click",async(e)=>{
        const res = await uploadFile(refreshfn);
    })
    uploadDir.addEventListener("click",async(e)=>{
        const res = await uploadDirFn(refreshfn);
    })

    refresh.addEventListener("click",async(e)=>{
        await refreshfn(appState.currentDir);
    })

    newDir.addEventListener("click",async (e)=>{
        const name =await renameDialog("请输入新文件夹名称","",true,null);
        const allName = new Path().from_string(appState.currentDir).push_clone(name).to_string_no_root()
        const res = await API.files_upload(appState.token,appState.currentZone,true,allName,"","");
        refreshfn(appState.currentDir)
    })

    function setCheckColVisible(visible) {
        document.querySelectorAll('.file-cb').forEach(el => {
            el.style.display = visible ? "" :"none" ;
        });
    }

    function clearChecks() {
        document.querySelectorAll('.file-cb').forEach(cb => {
            cb.checked = false;
        });
    }

    multiSelect.addEventListener("click",()=>{
        // if (is_lord())
        confirmSelect.hidden=false;
        multiSelect.hidden=true;
        setCheckColVisible(true);
    })

    confirmSelect.addEventListener("click",async()=>{
        multiSelect.hidden=false;
        confirmSelect.hidden=true;
        setCheckColVisible(false);
        const selectedNames = [...document.querySelectorAll('.file-cb:checked')].map(item => item.dataset.name)

        for (const name of selectedNames){
            const tmp = {name:name,is_directory:false}
            const res = await downloadFile(tmp);
        }
        clearChecks()
    })

    toolBar.append(refresh,goUp,newDir,upload,uploadDir,span,cwd,multiSelect,confirmSelect)
    return toolBar
}


async function uploadFile(refreshfn) {
    const fileInput = document.createElement('input');
    fileInput.type = 'file';
    fileInput.multiple = true;
    fileInput.style.display = 'none';

    document.body.appendChild(fileInput);
    const files = await new Promise((resolve) => {
        fileInput.addEventListener('change', (e) => {
        resolve(e.target.files);
        fileInput.remove();
        });
        fileInput.click();
    });

    if (!files.length) return;
    console.log(files);

    for (const file of files) {
        const uniqueName = new Path().from_string(appState.currentDir).push_clone(file.name).to_string_no_root();
        console.log(uniqueName);
        if(appState.currentFiles.some(item=>item.name == uniqueName)){
            window.alert(`当前文件夹下已经有一个${uniqueName}文件,不能重名！`)
            return;
        }
    }

    for (const file of files) {
        const [contentBase64, arrayBuffer] = await Promise.all([
            readFileAsBase64(file),
            readFileAsArrayBuffer(file)
        ]);
        const md5 = await calcFileMD5(arrayBuffer);
        const uniqueName = new Path().from_string(appState.currentDir).push_clone(file.name).to_string_no_root();
        const res = await API.files_upload(appState.token,appState.currentZone,false,uniqueName,md5,contentBase64)
    }

    refreshfn(appState.currentDir)

}

async function uploadDirFn(refreshfn) {
    const fileInput = document.createElement('input');
    fileInput.type = 'file';
    fileInput.multiple = true;
    fileInput.style.display = 'none';
    fileInput.directory = true;
    fileInput.webkitdirectory = true;

    document.body.appendChild(fileInput);
    const files = await new Promise((resolve) => {
        fileInput.addEventListener('change', (e) => {
            resolve(e.target.files);
            fileInput.remove();
        });
        fileInput.click();
    });

    const fileList=Array.from(files);
    // console.log(fileList);

    const dirSet = new Set();

    for (const file of fileList) {
        const relPath = file.webkitRelativePath;  // 此时是 "trip/photo.jpg"
        const parts = relPath.split('/');
        for (let i = 0; i < parts.length - 1; i++) {
            dirSet.add(parts.slice(0, i + 1).join('/'));
        }
    }
    
    const sortedDirs = Array.from(dirSet).sort((a, b) => a.split('/').length - b.split('/').length);
    console.log(sortedDirs);
    for (const dir of sortedDirs) {
        const dirPath = new Path().from_string(appState.currentDir).push_clone(dir).to_string_no_root();
        const existing = appState.currentFiles.filter(item => item.is_directory).map(item => item.name);
        if (existing.includes(dirPath)){
            alert(`当前目录下已有一个有 ${dirPath} 文件夹！本次操作取消!`)
            return;
        }
        const res = await API.files_upload(appState.token,appState.currentZone,true,dirPath,"","");
    }


    for (const file of files) {
        const [contentBase64, arrayBuffer] = await Promise.all([
            readFileAsBase64(file),
            readFileAsArrayBuffer(file)
        ]);
        const md5 = await calcFileMD5(arrayBuffer);
        const tmp = new Path().from_string(file.webkitRelativePath);
        const uniqueName = new Path().from_string(appState.currentDir).push_path(tmp).to_string_no_root();
        // console.log(uniqueName);
        const res = await API.files_upload(appState.token,appState.currentZone,false,uniqueName,md5,contentBase64)
    }
    refreshfn(appState.currentDir)
}

async function FileState(dir="") {
    const items = await API.files_list(appState.token,appState.currentZone,appState.currentDir);
    const mainBody = document.getElementById('main');
    const sidebarItems =document.getElementById("sidebar-item-container");
    
    sidebarItems.innerHTML=``;
    sidebarItems.append(backTree(),...sidebarFixedItems());
    mainBody.innerHTML = "";

    const fileContainer = document.createElement("div");
    fileContainer.id ="file-container" ;
    fileContainer.className="fileContainer";

    const cwd = document.createElement("div");
    cwd.className = "path";
    cwd.id = "currentPath";

    const refresh = async (dir)=>{
        appState.currentDir = dir;
        const items = await API.files_list(appState.token,appState.currentZone,dir);
        appState.currentFiles = items;
        cwd.textContent = `/${dir}`;
        console.log(appState.currentFiles);
        fileContainer.replaceChildren(fileTable(items,refresh));
    }
    mainBody.append(toolBar(refresh,cwd),fileContainer,noMoreContent());
    refresh(dir);
}

function getSetting(){
    const btn = document.createElement("div")
    btn.className = "sidebar-item";
    btn.id = "setting";
    btn.textContent = "设置";
    
    btn.addEventListener("click",(e)=>{
        console.log("设置");
    });
    return btn
}

function getLog(){
    const btn = document.createElement("div")
    btn.className = "sidebar-item";
    btn.id = "log";
    btn.title="查看最新的100条用户记录"
    btn.textContent = "查看日志";
    btn.addEventListener("click",async (e)=>{
        // console.log("日志");
        await logState();
    });
    return btn
}

function getLogout(){
    const btn = document.createElement("div")
    btn.className = "sidebar-item";
    btn.id = "logout";
    btn.textContent = "登出";
    btn.addEventListener("click",(e)=>{
        console.log("登出");
    });
    return btn
}

function renderZones(zones){
    zones.sort((a, b) => a.id - b.id);
    const zoneContainer = document.createElement("div");
    zoneContainer.className = "zone-container";
    zoneContainer.id = "zoneContainer";
    
    zones.forEach(zone =>{
        let fmtLords = ""; 
        if (zone.lords.length == 0){
            fmtLords = "所有人";
        }else{
            fmtLords = zone.lords.join(" , ");
        }

        const zoneDiv = document.createElement("div");
        zoneDiv.className = "zone";
        zoneDiv.dataset.zoneId = zone.id;
        zoneDiv.dataset.zoneName = zone.name;
        zoneDiv.dataset.zoneLord = JSON.stringify(zone.lords);

        const nameP = document.createElement("p");
        nameP.style.fontSize = "var(--fsize_big)";
        nameP.textContent = zone.name;
        
        const lordP = document.createElement("p");
        lordP.textContent = `主管：${fmtLords}`;
        zoneDiv.append(nameP, lordP);

        zoneDiv.addEventListener("contextmenu",async(e)=>{
            e.preventDefault();
            ZoneMenu(zone,e);})
        
        zoneDiv.addEventListener("click",async(e)=>{
            appState.currentZone = zone.name;
            appState.currentLords = zone.lords;
            await TreeState(zone.name);
        });

        zoneContainer.append(zoneDiv);

    })
    zoneContainer.append(noMoreContent());

    return zoneContainer

}

function noMoreContent(){
    const p = document.createElement("div")
    p.id = "noMoreContent";
    p.className = "loading";
    p.textContent = "没有更多内容了";
    return p
}

function testZones(){
    return [
        {"name":"公共","lords":["Alex","Bob"]},
        {"name":"测试","lords":["Bob","Charley"]},];
}



class Node {
    constructor (item){
        this.item=item;
        this.x=null;
        this.y=null;
        this.width = null;
        this.height= null;
        this.maxWidth=null;
        this.maxHeight=null;
    }    
}

class SingleRootTree {
    constructor(nodeMap){
        this.nodes=nodeMap
        this.treeWidth=null;
        this.treeHeight=null;
    }

    root() {
        const children = new Set(this.nodes.keys());
        for (const parent of this.nodes.values()){
            if (!children.has(parent)){
                return parent
            }
        }
        return null;
    }

    allNodes() {
        const all = new Set();
        for (const [child, parent] of this.nodes) {
            all.add(child);
            all.add(parent);
        }
        return all;
    }

    childrenList(){
        const childrenMap = new Map();
        for (const [child, parent] of this.nodes) {
            if (!childrenMap.has(parent)) {
                childrenMap.set(parent, [])
            };
            childrenMap.get(parent).push(child);
        }
        return childrenMap
    }
    totalWidth(){
        const tailNode = [...this.allNodes()].reduce((tailNode, node) => {
            return node.x > tailNode.x ? node : tailNode;
        });
        return tailNode.x+tailNode.maxWidth;
    }
    totalHeight(){
        const bottomNode = [...this.allNodes()].reduce((bottomNode, node) => {
            return node.y > bottomNode.y ? node : bottomNode;
        });
        return bottomNode.y+bottomNode.maxHeight;
    }
    
}


function emptyTree(){
    const treeLayout = document.createElement("div");
    treeLayout.id = "fileDirTree";
    treeLayout.className ="tree-layout";
    treeLayout.style = `--tree-width:0px;--tree-height:0px;`;

    const node = new Node("");
    node.x=50;
    node.y=50;
    node.width = 36;
    node.height = 38;
    node.maxHeight = 36;
    node.maxWidth = 38;
    treeLayout.append(renderNode(node));
    return treeLayout

}

export function renderTree(tree,intervalX=30,intervalY=0,paddingX=50,paddingY=50){
    let assignedtree = assignXY(tree,intervalX,intervalY,paddingX,paddingY);

    const width = assignedtree.totalWidth() + paddingX * 2;
    const height = assignedtree.totalHeight() + paddingY * 2;

    const treeLayout = document.createElement("div");
    treeLayout.id = "fileDirTree";
    treeLayout.className ="tree-layout";
    treeLayout.style = `--tree-width:${width}px;--tree-height:${height}px;`;
    treeLayout.append(renderLines(assignedtree));
    for (const node of assignedtree.allNodes()){
       treeLayout.append(renderNode(node));
    }
    return treeLayout
}

function renderNode(node) {
    const btn = document.createElement("button");
    btn.className = "tree-node";
    btn.dataset.name = node.item ?? "";
    btn.style = `left:${node.x}px; top:${node.y}px; width:${node.width}px; height:${node.height}px`;
    btn.textContent = `/${node.item}`;
    return btn
}

function renderLines(tree){
    const lines = document.createElementNS("http://www.w3.org/2000/svg","svg");
    lines.setAttribute("class","tree-lines")

    for (const [node, pnode] of tree.nodes){
        const x1 = pnode.x + pnode.width;
        const y1 = pnode.y + pnode.height/2;
        const x2 = node.x - (node.x - pnode.x - pnode.width)/2;
        const y2 = y1;
        const x3 = x2;
        const y3 = node.y+node.height/2;
        const x4 = node.x;
        const y4 = y3;
        
        const line1 = document.createElementNS("http://www.w3.org/2000/svg", "line");
        line1.setAttribute("x1", x1);
        line1.setAttribute("y1", y1);
        line1.setAttribute("x2", x2);
        line1.setAttribute("y2", y2);
        line1.setAttribute("class", "tree-line");

        const line2 = document.createElementNS("http://www.w3.org/2000/svg", "line");
        line2.setAttribute("x1", x2);
        line2.setAttribute("y1", y2);
        line2.setAttribute("x2", x3);
        line2.setAttribute("y2", y3);
        line2.setAttribute("class", "tree-line");

        const line3 = document.createElementNS("http://www.w3.org/2000/svg", "line");
        line3.setAttribute("x1", x3);
        line3.setAttribute("y1", y3);
        line3.setAttribute("x2", x4);
        line3.setAttribute("y2", y4);
        line3.setAttribute("class", "tree-line");
        
        lines.appendChild(line1);
        lines.appendChild(line2);
        lines.appendChild(line3);
        

        // text +=`\n<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" class = "tree-line" />
        //         \n<line x1="${x2}" y1="${y2}" x2="${x3}" y2="${y3}" class = "tree-line" />
        //         \n<line x1="${x3}" y1="${y3}" x2="${x4}" y2="${y4}" class = "tree-line"/>`;
        }
    // text += "\n</svg>";
    // return text
    return lines
}

function assignXY(tree,intervalX=40,intervalY=0,offsetX=0,offsetY=0){
    let root= tree.root();
    root.x = offsetX;
    root.y = offsetY;
    
    const childrenMap = tree.childrenList();
    
    function assignHeight(){
        for (const node of tree.allNodes()){
            node.height = 38;
        }
    }

    function assignWidth(){
        for (const node of tree.allNodes()){
            const nodeString = new String(node.item);
            node.width= 16*(nodeString.length+1)+20;
        }
    }

    function assignMaxHeight(node) {
        const children =childrenMap.get(node);
        if (!children){
            node.maxHeight = node.height+36;
            return;
        }
        node.maxHeight=0;
        for (const child of children){
            assignMaxHeight(child)
            node.maxHeight+= child.maxHeight;
        }
    }

    function assignMaxWidth(){
        for (const node of  tree.allNodes()){
            const peer = childrenMap.get(tree.nodes.get(node));
            if (!peer) { node.maxWidth = node.width;continue;}
            const max = Math.max(...peer.map(bro=>bro.width))
            node.maxWidth = max;
        }
    }

    function assignChildren(node, x, y) {        
        // node.height = assignHeight();
        // node.width = assignWidth(node);
        // node.maxWidth=124;
        node.x = x;
        node.y = y;
        // console.log(`AssignChidren${node.item},${node.x},${node.y},${node.width},${node.maxWidth},${node.height},${node.maxHeight}`)

        const children = childrenMap.get(node);
        if (!children) return;

        //assign Y
        let cursorY = node.y;
        for (const child of children) {
            assignChildren(child, node.x+node.maxWidth+intervalX, cursorY);
            cursorY += child.maxHeight + intervalY;
        }
    }
    assignHeight();
    assignWidth();
    assignMaxHeight(root);
    assignMaxWidth();
    assignChildren(root, root.x, root.y);
    return tree;

}

export function itemsToTree(relations){
    relations.sort((a, b) => a[0].localeCompare(b[0]));
    const nodePool = new Map(); // string -> Node
    const parentMap = new Map(); // Node -> Node

    function getNode(value) {
        if (!nodePool.has(value)) {
            nodePool.set(value, new Node(value));
        }
        return nodePool.get(value);
    }

    for (const [childValue, parentValue] of relations) {
        const childNode = getNode(childValue);
        const parentNode = getNode(parentValue);
        parentMap.set(childNode, parentNode);
    }
    return new SingleRootTree(parentMap);
}

function fileTable(items,refreshfn) {

    const table = document.createElement('table');
    table.className = "file-table";
    table.id = "fileTable";
    const thead = document.createElement("thead");
    thead.id = "fileTableThead";

    const tbody =document.createElement("tbody");
    tbody.id = "fileTableBody";
    
    const headtr = document.createElement('tr');

    const cbTd = document.createElement('th');
    cbTd.className = 'check-col';
    
    
    const nameTd = document.createElement('th');
    nameTd.textContent = "文件名";
    nameTd.style.width ="auto";
    nameTd.className = "th";
    

    const sizeTd = document.createElement('th');
    sizeTd.textContent = "大小";
    sizeTd.style.width ="160px";
    sizeTd.className = "th";
    

    const createdatTd=document.createElement('th');
    createdatTd.textContent ="创建时间";
    createdatTd.style.width ="160px";
    createdatTd.className = "th";
    

    const modifiedatTd=document.createElement('th');
    modifiedatTd.textContent ="最后修改于";
    modifiedatTd.style.width ="160px";
    modifiedatTd.className = "th";
    

    const creatorTd=document.createElement("th");
    creatorTd.style.width ="160px";
    creatorTd.className = "th";
    creatorTd.textContent="创建者";
    

    const modifierTd=document.createElement("th");
    modifierTd.textContent="最后修改者";
    modifierTd.style.width ="160px";
    modifierTd.className = "th";
    headtr.append(cbTd,nameTd,sizeTd,createdatTd,modifiedatTd,creatorTd,modifierTd);
    headtr.append(modifierTd);
    thead.append(headtr);

    items.sort((a, b) => {
        if (a.is_directory !== b.is_directory) return a.is_directory ? -1 : 1;
        const x = new Path().from_string(a.name).peek_filename();
        const y = new Path().from_string(b.name).peek_filename()
        return x.localeCompare(y);
    });

    items.forEach(item => {
        const tr = document.createElement('tr');
        const cbTd = document.createElement('td');
        cbTd.className = 'check-col';

        const cb = document.createElement('input');
        cb.type = 'checkbox';
        cb.className = 'file-cb';
        cb.dataset.name = item.name;
        cb.dataset.isDir = item.is_directory;
        cb.style.display="none";

        if (!item.is_directory && is_lord(item.creator)){
            cbTd.append(cb);
        }
        const nameTd = document.createElement('td');
        const iconContainer = document.createElement('img');
        iconContainer.width =18;
        iconContainer.height=18; 
        iconContainer.src = getIcon(item);
        
        const fileName = new Path().from_string(item.name).peek_filename();
        
        const textSpan = document.createElement('span');
        textSpan.textContent = ' ' + fileName; // 加个空格

        iconContainer.style.verticalAlign = 'middle';
        textSpan.style.verticalAlign = 'middle';
        nameTd.append(iconContainer,textSpan);

        nameTd.addEventListener('contextmenu',(e)=>{e.preventDefault();fileMenu(item,e,is_lord(item.creator),refreshfn)});
        nameTd.addEventListener('click', async() => fileClick(item,refreshfn));
        nameTd.addEventListener('dblclick', () => {fileDbClick(item,is_lord(item.creator))}
    );
   
        const sizeTd = document.createElement('td');
        if (item.is_directory) {sizeTd.textContent = "";}
        else {sizeTd.textContent = FMT.fmt_size(item.size);};        

        const createdatTd=document.createElement('td');
        createdatTd.textContent =FMT.fmt_time(item.created_at);
        
        const modifiedatTd=document.createElement('td');
        modifiedatTd.textContent =FMT.fmt_time(item.modified_at);
        
        const creatorTd=document.createElement("td");
        creatorTd.textContent=item.creator;
        
        const modifierTd=document.createElement("td");
        modifierTd.textContent=item.last_modifier;
        
        if (!item.is_directory){tr.draggable = true;}
        tr.append(cbTd,nameTd,sizeTd,createdatTd,modifiedatTd,creatorTd,modifierTd);
        tbody.appendChild(tr);
  });
  table.append(thead,tbody);
  return table
}


function getIcon(item) {
  if (item.is_directory) return iconMap.dir;
  const ext = (item.name || '').split('.').pop().toLowerCase();
  return iconMap[ext] || iconMap.other;
}


function fileMenu(item,event,is_lord,refreshfn){

  const existing = document.querySelector('.context-menu');
  existing?.remove();
  const menu = document.createElement('div');
  menu.className = 'context-menu';
  menu.style.left = event.clientX + 'px';
  menu.style.top = event.clientY + 'px';
  const actions = [{ label: '详细信息', action: () => detailFile(item)},]
  if (is_lord){
        actions.push({ label: '重命名', action: async() => renameFile(item,refreshfn)})
        actions.push({ label: '复制到', action: async() => copyFile(item,refreshfn)})
        actions.push({ label: '移动到', action: async() => moveFile(item,refreshfn)})
        actions.push({ label: '下载', action: async() => downloadFile(item)})
        actions.push({ label: '修改创建人', action: async() => chownFile(item,refreshfn)})
        actions.push({ label: '删除', action: async() => deleteFile(item,refreshfn)})
  }

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


function detailFile(item){
  const overlay = document.createElement('div');
  overlay.className="modal-overlay"
  const title = document.createElement('h3');
  title.className="modal-title";
  title.textContent = "详细信息"
  const modal = document.createElement('div');
  modal.className="modal-box"
  const list = document.createElement('dl');
  list.style.padding = "20px";
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
      last_modifier: '最后修改者',
      zone:"所属仓库"
    };

  for (let [key, value] of Object.entries(item)) {
    const dd = document.createElement('p');
    if (key == "parent_name" ||key == "name"){
      value=`/${value}`
    }
    dd.textContent = `${keyMap[key]} : ${value}`;
    list.appendChild(dd);
  }
  const closeBtn = document.createElement('button');
  closeBtn.textContent = '关闭';
  closeBtn.className = "modal-btn";

  closeBtn.onclick = () => overlay.remove();
  modal.append(title,list,closeBtn);
  overlay.appendChild(modal);
  document.body.appendChild(overlay);
  overlay.onclick = (e) => { if (e.target === overlay) overlay.remove(); };
}

async function renameFile(item,refreshfn){
    const suffix = new Path().from_string(item.name).get_suffix();
    const oldName = new Path().from_string(item.name).rm_suffix().peek_filename();
    const newName = await renameDialog("请输入新名称",oldName,item.is_directory,suffix);
    if (!newName){return;}

    const allName = new Path().from_string(item.name).get_parent().push_clone(newName).add_suffix(suffix).to_string_no_root()
    
    if (oldName == newName){return;}
    if (appState.currentFiles.some(item => item.name == allName)){
        window.alert(`当前文件夹下已经有一个${newName}${suffix}文件,不能重名！`)
        return;
    }
    const res = await API.files_rename(appState.token,appState.currentZone,item.is_directory,item.name,allName)
    refreshfn(appState.currentDir);
}

async function renameDialog(title,placeholder,is_dir,suffix){
    return new Promise(resolve => {
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    const box = document.createElement('div');
    box.className = 'modal-box';

    box.innerHTML = `
    <h3 class = "modal-title">${title}</h3>  
      <div style="display: flex; align-items: baseline;flex-direction:row;padding:8px 10px 8px 10px;">
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
        const check = is_path_valid(val);
        if (check.valid){close(val);}else{err.textContent = check.msg}
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

async function copyFile(item,refreshfn) {
    const target = await moveFileDialog("选择目标文件夹");
    if (!target || target == appState.currentDir){return;}
    const tmp = new Path().from_string(item.name).peek_filename();
    const newName = new Path().from_string(target).push_clone(tmp).to_string_no_root();
    const existing = await API.files_list(appState.token,appState.currentZone,target);
    if (existing.some(exs => exs.name == newName)){
        window.alert(`目标文件夹下已经有一个${newName}文件,复制会导致重名，拒绝本次操作！`)
        return;
    }
    const res = await API.files_copy(appState.token,appState.currentZone,item.is_directory,item.name,newName);
    refreshfn(appState.currentDir);
}

async function moveFile(item,refreshfn) {
    const target = await moveFileDialog("选择目标文件夹");
    if (!target || target == appState.currentDir){return;}
    const tmp = new Path().from_string(item.name).peek_filename();
    const newName = new Path().from_string(target).push_clone(tmp).to_string_no_root();
    const existing = await API.files_list(appState.token,appState.currentZone,target);
    if (existing.some(exs => exs.name == newName)){
        window.alert(`目标文件夹下已经有一个${newName}文件,移动会导致重名，拒绝本次操作！`)
        return;
    }
    const res = await API.files_rename(appState.token,appState.currentZone,item.is_directory,item.name,newName);
    refreshfn(appState.currentDir);
}

//todo
async function moveFileDialog(title) {
    const items = await API.zone_tree(appState.token,appState.currentZone);

    return new Promise(resolve => {
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    const offset=6;
    let selected=null;


    const box =  document.createElement('div');
    box.className = "modal-box";
    box.style.flex="1";
    box.style.flexDirection="column";
    box.style.maxWidth="80%";
    box.style.height="80%";
    

    const ptr = document.createElement("div");
    ptr.hidden=true;
    ptr.style.position="absolute";
    ptr.style.border = `${offset}px #25b601a2 solid`;
    ptr.style.borderRadius = "10px";

    const h = document.createElement("h3");
    h.className ="modal-title";
    h.textContent =title;
    h.width="100%";
    
    const btnContainer = document.createElement("div");
    btnContainer.className="modal-btn-container";

    const treeBox = document.createElement("div");
    treeBox.width = "100%";
    treeBox.style.overflow="scroll";
    treeBox.style.height="82%";


    const confirm = document.createElement("button");
    confirm.className="modal-btn";
    confirm.textContent = "确认";

    const cancel = document.createElement("button");
    cancel.className="modal-btn";
    cancel.textContent = "取消";

    const err = document.createElement("div");
    err.className="modal-error";
    err.id = "input-dialog-err";
    
    let tree=null;
    if (items.length ==0){
        tree=emptyTree()
    }else{
        tree=renderTree(itemsToTree(items));
    }
    tree.append(ptr);
    


    box.addEventListener("click",async (e)=>{
        const El = event.target.closest(".tree-node");
        if (!El) return;
        ptr.hidden=false;
        ptr.style.left = `${parseFloat(El.style.left)-offset}px`;
        ptr.style.top = `${parseFloat(El.style.top)-offset}px`;
        ptr.style.width = `${parseFloat(El.style.width)+2*offset}px`;
        ptr.style.height = `${parseFloat(El.style.height)+2*offset}px`;
        selected = El.dataset.name
    })
    

    const close = (value) => {
      overlay.remove();
      resolve(value);
    };

    confirm.addEventListener('click', () => {
        close(selected);        
    });

    cancel.addEventListener('click', () => close(null));
    overlay.addEventListener('click', e => {
    if (e.target === overlay) close(null);
    });
    
    overlay.addEventListener('keydown', e => {
      if (e.key === 'Enter') confirmBtn.click();
      if (e.key === 'Escape') cancelBtn.click();
    });
    
    treeBox.append(tree);
    btnContainer.append(cancel,confirm);
    box.append(h,treeBox,btnContainer,err)
    overlay.append(box);
    document.body.appendChild(overlay);
    })
}

async function downloadFile(item) {
    let check=true;
    if (item.is_directory){
        check = window.confirm("你选择下载一个文件夹，后台会将所有内容打包成zip文件发送给你，这需要点时间，也有可能耗尽内存而导致崩溃，确定吗？")
    }
    if (!check){return;}
    const loading = document.createElement("div");
    loading.className = "spinner";
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    overlay.append(loading)
    document.body.append(overlay);

      const res = await API.files_download(appState.token,appState.currentZone,item.is_directory,item.name)
      if (!res.ok) throw new Error(`HTTP ${res.status}`);

      const md5Base64 = await res.headers.get('X-Content-MD5');
      const blob = await res.blob();
      const buffer = await blob.arrayBuffer();
      // 验证 MD5
      let tmp;
      if (md5Base64) {
        const md5Compute = await calcFileMD5(buffer);
        if (md5Compute !== md5Base64) {
            tmp = window.confirm(`检测到MD5值不匹配，也就是说，传输过程中可能存在数据损失。是否放弃本次下载？`)
        }
      }
      if(tmp){overlay.remove();return;}
      
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
        a.href = url;
        a.download = new Path().from_string(item.name).peek_filename() || 'download';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    overlay.remove()
}

async function calcFileMD5(arrayBuffer) {
  const spark = new SparkMD5.ArrayBuffer();
  spark.append(arrayBuffer);
  return spark.end(); 
}

function usersCheckBox(exist_users) {
    const box = document.createElement("div");
    box.className = "check-group";
    box.id = "checkBox";
    box.style.alignItems="center";

    for (const i of appState.allUsers) {
        const label = document.createElement("label");
        const input = document.createElement("input");
        input.type = "checkbox";
        input.value = i;
        if (exist_users.includes(i)){input.checked = true;}else{input.checked = false;}
        label.append(input, i);
        box.append(label);
    }
    return box

}

async function chownDialog(title,exist_users,single) {
    return new Promise(resolve => {
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    const box = document.createElement('div');
    const checkbox = usersCheckBox(exist_users);

    box.className = 'modal-box';
    box.innerHTML = `
    <h3 class="modal-title">${title}</h3>
    ${checkbox.outerHTML}
    <div style = "padding:20px;">
        <div class="modal-btn-container">
        <button class="modal-btn" data-action="cancel">取消</button>
        <button class="modal-btn" data-action="confirm">确认</button>
        </div>
        <div class="modal-error" id = "input-dialog-err"> </div>
    </div>
    `;
    overlay.appendChild(box);
    document.body.appendChild(overlay);

    const err = box.querySelector('#input-dialog-err');
    const confirmBtn = box.querySelector('[data-action="confirm"]');
    const cancelBtn = box.querySelector('[data-action="cancel"]');

    const close = (value) => {
      overlay.remove();
      resolve(value);
    };

    confirmBtn.addEventListener('click', () => {
        const selected_users = Array.from(
            box.querySelectorAll('#checkBox input[type="checkbox"]:checked')
        ).map(input => input.value);

        if (selected_users.length==0){
            err.textContent = "你忘了做出选择";
            return;
        }
        if (single && selected_users.length!=1){
            err.textContent = "只能选一个人";
            return;
        }
        close(selected_users);
    });

    cancelBtn.addEventListener('click', () => close(null));
    overlay.addEventListener('click', e => {
      if (e.target === overlay) close(null);
    });
    overlay.addEventListener('keydown', e => {
      if (e.key === 'Enter') confirmBtn.click();
      if (e.key === 'Escape') cancelBtn.click();
    });
  });
}

async function chownFile(item,refreshfn){
  const new_creator = await chownDialog("输入新的创建人",[item.creator,],true);
  if (!new_creator||new_creator.length!=1){return;}
  const res = await API.files_chown(appState.token,appState.currentZone,item.is_directory,item.name,new_creator[0])
  refreshfn(appState.currentDir);
}
async function deleteFile(item,refreshfn){
    let msg;
    if (item.is_directory){
       msg = `你正在删除${item.name}及其所有内容,确定吗？`
    }else{
       msg = `你正在删除${item.name},确定吗？`
    }
    const check = window.confirm(msg);
    if (!check){return;}

    const res =await API.files_delete(appState.token,appState.currentZone,item.is_directory,item.name);
    refreshfn(appState.currentDir)

}
function ZoneMenu(zone,event){
  const existing = document.querySelector('.context-menu');
  existing?.remove();
  const menu = document.createElement('div');
    menu.className = 'context-menu';
    menu.style.left = event.clientX + 'px';
    menu.style.top = event.clientY + 'px';
  const actions = [
    { label: '编辑', action: async() => {
        const new_zone = await ZoneDialog("修改当前区域名称",zone.name,zone.lords,null);
        if (!new_zone){return;}
        if (new_zone.name != zone.name){
            const res = await API.zone_rename(appState.token,zone.name,new_zone.name);
        }
        if (! sameSet(zone.lords,new_zone.lords)){
            const res = await API.zone_newlords(appState.token,new_zone.name,new_zone.lords);
        }
        await zoneState();
    }},
    { label: '删除', action: async() => {
        const a = window.confirm("警告一次！你正在删除一个仓库及其文件，数据无法恢复，同时你的操作会被记录到系统日志，确定？");
        const b = window.confirm("警告两次！你正在删除一个仓库及其文件，数据无法恢复，同时你的操作会被记录到系统日志，确定？");
        const c = window.confirm("警告三次！你正在删除一个仓库及其文件，数据无法恢复，同时你的操作会被记录到系统日志，确定？");
        if(a==b==c==true){
            const res = await API.zone_delete(appState.token,zone.name);
            await zoneState();
        }
        return;
    }}];

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


function sameSet(a, b) {
  const sa = new Set(a);
  const sb = new Set(b);
  if (sa.size !== sb.size) return false;
  for (const x of sa) {
    if (!sb.has(x)) return false;
  }
  return true;
}

function is_lord(creator){
    if (appState.currentLords.length == 0){return true;}
    const user = localStorage.getItem("username");
    if(user == creator || appState.currentLords.includes(user)){
        return true;
    }
    return false;
};


async function fileClick(item,refreshfn){
    if(item.is_directory){
        // console.log(`即将进入${item.name}`);
        await refreshfn(item.name);
    }else{
        return;
    }
}

async function fileDbClick(item,is_lord) {
    if(item.is_directory){
        return;}
    if (!is_lord){return;}

    const uploadfn = async function(arrayBuffer){
        const md5 = await calcFileMD5(arrayBuffer);
        const contentBase64 = arrayBufferToBase64(arrayBuffer);
        const res = await API.files_upload(appState.token,appState.currentZone,false,item.name,md5,contentBase64)
    }

    const res = await API.files_download(appState.token,appState.currentZone,false,item.name);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const blob = await res.blob();
    Render.previewFile(blob, new Path().from_string(item.name).peek_filename(),uploadfn);
}

async function ZoneDialog(title,placeholder,lords,suffix) {
  return new Promise(resolve => {
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    const box = document.createElement('div');
    box.className = 'modal-box';
    box.innerHTML = `
    <h3 class="modal-title">${title}</h3>
    <div style = "padding:20px;">
        <input id = "input" type="text" class="modal-input" placeholder="" autofocus>
    </div>
    <h3 class="modal-title">请勾选主管
        <p style="font-size:var(--fsize_middle);">（只有主管可以在线预览/编辑/下载不属于自己的文件）</p>
        <p style="font-size:var(--fsize_middle);">（全部不勾选等同于全选）</p>
    </h3>
    
        <div id="checkBox" class= "check-group" style="align-items:center;">
        </div>
    <div style = "padding:20px;">
        <div class="modal-btn-container">
        <button class="modal-btn" data-action="cancel">取消</button>
        <button class="modal-btn" data-action="confirm">确认</button>
        </div>
        <div class="modal-error" id = "input-dialog-err"> </div>
    </div>
    `;

    overlay.appendChild(box);
    document.body.appendChild(overlay);
    const input = box.querySelector('.modal-input');
    input.value= placeholder;

    const checkBox = document.getElementById("checkBox");
    for (const i of appState.allUsers) {
        const label = document.createElement("label");
        const input = document.createElement("input");
        input.type = "checkbox";
        input.value = i;
        if (lords.includes(i)){input.checked = true;}else{input.checked = false;}
        label.append(input, i);
        checkBox.append(label);
    }

    const err = box.querySelector('#input-dialog-err');
    const confirmBtn = box.querySelector('[data-action="confirm"]');
    const cancelBtn = box.querySelector('[data-action="cancel"]');
    
    const close = (value) => {
      overlay.remove();
      resolve(value);
    };

    confirmBtn.addEventListener('click', () => {
        const name = input.value.trim();
        const lords = Array.from(
            box.querySelectorAll('#checkBox input[type="checkbox"]:checked')
        ).map(input => input.value);
        const checkResult = is_path_valid(name);
        if (!checkResult.valid) {
            err.textContent=checkResult.msg;
            return;
        }
            close({name,lords});
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


function is_path_valid(name){
  if (!name ){
    return {valid:false,msg:"空名称显然行不通，你在试图操作虚空，这很危险！"}
  }

  const hasWhitespace = /\s/.test(name);
    
  if(hasWhitespace){
    return {valid:false,msg:'你在文件名中藏了个空白字符（空格/换行/制表符...），真狡猾！你会搞坏磁盘的！'};
  }
  const forbiddenChars = /[\\/:*?"<>|]/;



  if (forbiddenChars.test(name)) {
    return {valid:false,msg:'包含这些特殊字符是个坏主意：\\ / : * ? " < > | , 地球会因此爆炸的！'};
  }

  return {valid:true,msg:'好名字！'}
}



async function check_session(){
  const res = await API.auth_verify(localStorage.getItem("token"))
  if (!res.ok){
    window.location.href = "/portal/login";
    // console.log(res)
  }
}


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

function arrayBufferToBase64(arrayBuffer) {
  const bytes = new Uint8Array(arrayBuffer);
  let binary = '';
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode(
      ...bytes.subarray(i, i + chunkSize)
    );
  }
  return btoa(binary);
}

document.body.append(FviwerInit())
check_session();
appState.allUsers= await API.accounts_list(appState.token);
zoneState();


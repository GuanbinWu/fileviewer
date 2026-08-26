export function testTree(){
    const relations = [
        ["/a2new/a4","/a2new"],
        ["/bbb/TC/TC2/湾谷海报","/bbb/TC/TC2"],
        ["/a2new/湾谷海报自建","/a2new"],
        ["/a2new","/"],
        ["/ccc/ihj","/ccc"],
        ["/bbb","/"],
        ["/nnn","/"],
        ["/UH","/"],
        ["/ccc/a2_cp/a4","/ccc/a2_cp"],
        ["/bbb/TC","/bbb"],
        ["/ccc","/"],
        ["/fast","/"],
        ["/slow","/"],
        ["/fast/a3","/fast"],
        ["/mmm","/"],
        ["/ccc/dpd","/ccc"],
        ["/bbb/TC/TC2/湾谷海报/高分辨率图象","/bbb/TC/TC2/湾谷海报"],
        ["/bbb/TC/TC2","/bbb/TC"],
        ["/ccc/a2_cp","/ccc"],
    ]
    return relations;
}




const compoundExtensions = [
  ".tar.gz",
  ".tar.bz2",
  ".tar.xz"
];

export class Path{
    constructor(segments = []){
        this.segment=segments
    }

    push_self(seg){
        this.segment.push(seg)
        return this;
    }

    pop_self(){
        this.segment.pop()
        return this;
    }

    push_clone(seg){
        return new Path([...this.segment, seg]);
    }
    push_path(path){
        let tmp=this.segment.concat(path.segment);
        return new Path(tmp);
    }
    
    pop_clone(){
        return new Path(this.segment.slice(0, -1));
    }
    
    peek_filename(){
        if (this.segment.length==0) {
            return "";
        }else{
            return this.segment[this.segment.length - 1];
        }
    }
    
    to_string_with_root(){
        let s = this.segment.join("/")
        return `/${s}`;
    }

    to_string_no_root(){
        return this.segment.join("/");
    }
    
    from_string(s){
        // console.log(s)
        if (s.startsWith("/")){
            s = s.slice(1)
        }
        if (s == ""){
            return new Path();
        }

        for (const  seg of s.split("/")) {
            this.segment.push(seg)
        }
        return this
    }
    get_parent(){
        if (this.segment.length>1){
            return new Path(this.segment.slice(0,-1))
        }
        else {
            return new Path([])
        }
    }
    get_suffix(){
        const tmp = this.segment[this.segment.length - 1];
        let dotIndex = tmp.lastIndexOf(".");
        for (const ext of compoundExtensions) {
            if (tmp.endsWith(ext)) {
            dotIndex = filename.length - ext.length;
            }
        }
        
        if (dotIndex === -1) {
            return "";
        }
        return tmp.substring(dotIndex);
    }
    add_suffix(suffix){
        if (suffix == ""){
            return this;
        }
        const last = this.segment[this.segment.length - 1];
        this.segment[this.segment.length - 1] = last + (suffix.startsWith('.') ? '' : '.') + suffix;
        return this;
    }

    rm_suffix(){
        const suffix = this.get_suffix();
        const tmp = this.segment[this.segment.length - 1];
        const idx = tmp.length -suffix.length
        this.segment[this.segment.length - 1] = tmp.substring(0,idx);
        return this;
    }
}


export class FMT{
    constructor(){}

    static fmt_time(time){
        const now = new Date();
        const diff = now - new Date(time);
        const days = Math.floor(diff / (1000 * 60 * 60 * 24));
        const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
        const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
        const seconds = Math.floor((diff % (1000 * 60)) / 1000);
        if (days > 0) {
            const d = new Date(time);
            const yy = String(d.getFullYear());
            const mm = String(d.getMonth() + 1).padStart(2, '0');
            const dd = String(d.getDate()).padStart(2, '0');
            return `${yy}/${mm}/${dd}`;
        } else if (hours > 0) {
            return `${hours}小时${minutes}分前`;
        } else if (minutes > 0) {
            return `${minutes}分${seconds}秒前`;
        } else {
            return `${seconds}秒前`;
    }
    }
    
    static  fmt_size(size) {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let index = 0;
    while (size >= 1024 && index < units.length - 1) {
        size /= 1024;
        index++;
    }
    return `${size.toFixed(2)} ${units[index]}`;
    }
}


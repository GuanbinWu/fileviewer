export const Render = {
  previewFile(blob, fileName,uploadfn) {
    const existing = document.querySelector('.previewer');
    if (existing) existing.remove();
    const type = blob.type;
    if (type === 'application/pdf') {
      this.previewPDF(blob, fileName);
    } else if (type.startsWith('text/') || type === 'application/json') {
      this.previewText(blob, fileName,uploadfn);
    } else if (type.startsWith('image/')) {
      this.previewImage(blob, fileName);
    }
  },

  previewPDF(blob, fileName) {
    const url = URL.createObjectURL(blob);
    const win = window.open('', '_blank');
    if (win) {
      win.document.title = fileName;
      win.location.href = url;
    } else {
      window.location.href = url;
    }
    setTimeout(() => URL.revokeObjectURL(url), 60000);
  },

  previewText(blob, fileName,uploadfn) {
    blob.text().then(text => {
      const container = document.createElement('div');
      container.className = 'previewer';
      const textarea = document.createElement('textarea');
      textarea.className = 'text-editor';

      textarea.value = text;
      textarea.readOnly = false;
      container.appendChild(textarea);

      const btnContainer =document.createElement('div');
      btnContainer.className='modal-btn-container';
      btnContainer.style.width="70%";
      // const saveBtn = document.createElement('button');
      // saveBtn.className = "modal-btn";
      // saveBtn.textContent = '保存';

      // const cancelBtn = document.createElement('button');
      // cancelBtn.className = "modal-btn";
      // cancelBtn.textContent = '取消';

      const saveBtn = document.createElement("button");
      saveBtn.className="modal-btn";
      saveBtn.textContent = "保存";

      const cancelBtn = document.createElement("button");
      cancelBtn.className="modal-btn";
      cancelBtn.textContent = "取消";

      btnContainer.append(cancelBtn,saveBtn);
      container.appendChild(btnContainer);
      document.body.appendChild(container);

      saveBtn.addEventListener('click', async () => {
        const newText = textarea.value;
        const newBlob = new Blob([newText], { type: 'text/plain;charset=utf-8' });
        const arrayBuffer = await newBlob.arrayBuffer()
        uploadfn(arrayBuffer);
        container.remove();
      });
      cancelBtn.addEventListener('click', (e)=>{
        container.remove()
      })
    });
  },

  previewImage(blob, fileName) {
    const container = document.createElement('div');
    container.className = 'previewer';
    const img = document.createElement('img');
    img.src = URL.createObjectURL(blob);
    img.style.maxWidth = '100%';
    img.style.maxHeight = '100%';
    container.appendChild(img);
    document.body.appendChild(container);
    container.addEventListener('click', (e) => {
      if (e.target === container) {
        container.remove();
        URL.revokeObjectURL(img.src);
      }})

    container.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      container.remove();
      URL.revokeObjectURL(img.src);
    });
  }

};
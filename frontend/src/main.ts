import init, { convert } from './nazori_wasm/nazori.js';

async function main() {
  await init(); // Initialize WebAssembly

  const targetEl = document.getElementById('target') as HTMLSelectElement;
  const methodEl = document.getElementById('method') as HTMLSelectElement;
  const formatEl = document.getElementById('format') as HTMLSelectElement;
  const formatHintEl = document.getElementById('format-hint') as HTMLElement;
  const zoomEl = document.getElementById('zoom') as HTMLInputElement;
  const epsilonEl = document.getElementById('epsilon') as HTMLInputElement;
  const fileUploadEl = document.getElementById('file-upload') as HTMLInputElement;
  const fileNameEl = document.getElementById('file-name') as HTMLElement;
  const resultAreaEl = document.getElementById('result-area') as HTMLElement;
  const resultMessageEl = document.getElementById('result-message') as HTMLElement;
  const loadingEl = document.getElementById('loading') as HTMLElement;
  const convertBtn = document.getElementById('convert-btn') as HTMLButtonElement;

  let selectedFiles: File[] = [];

  function updateMethodOptions() {
    const target = targetEl.value;
    const currentMethod = methodEl.value;
    
    methodEl.innerHTML = '';
    
    if (target === 'bldg') {
      methodEl.innerHTML = `
        <option value="flat_single">Flat (SingleId)</option>
        <option value="flat_range">Flat (RangeId)</option>
        <option value="with_attr_single">With Attr (SingleId)</option>
        <option value="with_attr_range">With Attr (RangeId)</option>
      `;
    } else {
      methodEl.innerHTML = `
        <option value="flat_single">Flat (SingleId)</option>
        <option value="with_attr_single">With Attr (SingleId)</option>
      `;
    }
    
    if (Array.from(methodEl.options).some(opt => opt.value === currentMethod)) {
      methodEl.value = currentMethod;
    } else {
      methodEl.value = 'flat_single';
    }
    
    updateFormatState();
  }

  targetEl.addEventListener('change', updateMethodOptions);

  function updateFormatState() {
    if (methodEl.value.startsWith('with_attr')) {
      formatEl.value = 'json';
      formatEl.disabled = true;
      formatHintEl.textContent = 'with_attr は JSON 出力のみです';
    } else {
      formatEl.disabled = false;
      formatHintEl.textContent = '';
    }
  }

  methodEl.addEventListener('change', updateFormatState);
  updateMethodOptions();

  const uploadAreaEl = document.querySelector('.upload-area') as HTMLElement;
  const fileListEl = document.getElementById('file-list') as HTMLUListElement;

  function handleFileSelect(files: FileList | File[]) {
    selectedFiles = Array.from(files);
    
    fileListEl.innerHTML = '';

    if (selectedFiles.length === 1) {
      fileNameEl.textContent = selectedFiles[0].name;
      fileListEl.classList.remove('hidden');
      const li = document.createElement('li');
      li.textContent = selectedFiles[0].name;
      fileListEl.appendChild(li);
    } else if (selectedFiles.length > 1) {
      fileNameEl.textContent = `${selectedFiles.length}個のファイルを選択中`;
      fileListEl.classList.remove('hidden');
      selectedFiles.forEach(file => {
        const li = document.createElement('li');
        li.textContent = file.name;
        fileListEl.appendChild(li);
      });
    } else {
      fileNameEl.textContent = 'XMLファイルを選択 または ドロップ';
      fileListEl.classList.add('hidden');
    }
    
    convertBtn.disabled = selectedFiles.length === 0;
    resultAreaEl.classList.add('hidden');
  }

  async function processFiles(files: File[]) {
    if (files.length === 0) return;
    
    loadingEl.classList.remove('hidden');
    resultAreaEl.classList.add('hidden');
    convertBtn.disabled = true;

    try {
      const target = targetEl.value;
      const method = methodEl.value;
      const format = formatEl.value;
      const zoom = parseInt(zoomEl.value, 10);
      const epsilon = parseFloat(epsilonEl.value);
      const multiMode = (document.getElementById('multi-mode') as HTMLSelectElement).value;

      // Give browser time to show the loading text
      await new Promise(resolve => setTimeout(resolve, 50));

      const startTime = performance.now();
      let totalCount = 0;

      if (multiMode === 'merge') {
        const xmlStrings = await Promise.all(files.map(f => f.text()));
        const resultString = convert(xmlStrings, zoom, epsilon, target, method, format);
        
        if (format === 'json') {
          try {
             const parsed = JSON.parse(resultString);
             totalCount = Array.isArray(parsed) ? parsed.length : 0;
          } catch(err) { /* ignore */ }
        } else {
          totalCount = resultString.split(',').filter(x => x.trim().length > 0).length;
        }

        // Create download
        const blob = new Blob([resultString], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `output_${Date.now()}.${format}`;
        a.click();
        URL.revokeObjectURL(url);
      } else {
        // separate mode
        for (let i = 0; i < files.length; i++) {
          const file = files[i];
          const xmlString = await file.text();
          const resultString = convert([xmlString], zoom, epsilon, target, method, format);

          if (format === 'json') {
            try {
               const parsed = JSON.parse(resultString);
               totalCount += Array.isArray(parsed) ? parsed.length : 0;
            } catch(err) { /* ignore */ }
          } else {
            totalCount += resultString.split(',').filter(x => x.trim().length > 0).length;
          }

          const baseName = file.name.substring(0, file.name.lastIndexOf('.')) || file.name;
          const blob = new Blob([resultString], { type: 'text/plain' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `${baseName}_converted.${format}`;
          a.click();
          URL.revokeObjectURL(url);

          // wait briefly to allow multiple downloads
          await new Promise(resolve => setTimeout(resolve, 150));
        }
      }

      const elapsed = ((performance.now() - startTime) / 1000).toFixed(2);
      resultMessageEl.textContent = `変換成功！ 合計 ${totalCount} 件のデータを抽出しました（${elapsed}秒）。ファイルのダウンロードが始まります。`;
      resultAreaEl.classList.remove('hidden');

    } catch (err: any) {
      alert("変換エラー: " + err);
    } finally {
      loadingEl.classList.add('hidden');
      convertBtn.disabled = false;
    }
  }

  convertBtn.addEventListener('click', () => {
    if (selectedFiles.length > 0) {
      processFiles(selectedFiles);
    }
  });

  fileUploadEl.addEventListener('change', (e) => {
    const files = (e.target as HTMLInputElement).files;
    if (files && files.length > 0) {
      handleFileSelect(files);
    }
  });

  uploadAreaEl.addEventListener('dragover', (e) => {
    e.preventDefault();
    uploadAreaEl.classList.add('dragover');
  });

  uploadAreaEl.addEventListener('dragleave', (e) => {
    e.preventDefault();
    uploadAreaEl.classList.remove('dragover');
  });

  uploadAreaEl.addEventListener('drop', (e) => {
    e.preventDefault();
    uploadAreaEl.classList.remove('dragover');
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      handleFileSelect(files);
    }
  });
}

main().catch(console.error);

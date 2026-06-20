<script>
  import { onMount } from 'svelte';
  
  // Lucide Icons Import
  import { 
    FolderOpen, 
    GitMerge, 
    Scissors, 
    Minimize2, 
    ScanText, 
    Sun, 
    Moon, 
    FileText, 
    ZoomIn, 
    ZoomOut, 
    Plus, 
    RotateCw, 
    Trash2, 
    CheckCircle2, 
    ArrowUp, 
    ArrowDown, 
    RefreshCw,
    Download,
    XCircle,
    Info,
    CheckCircle,
    AlertTriangle,
    Settings
  } from '@lucide/svelte';
  
  // Tauri API imports
  let tauriAvailable = false;
  let invoke = null;
  
  onMount(async () => {
    try {
      if (window.__TAURI_INTERNALS__) {
        const tauriCore = await import('@tauri-apps/api/core');
        invoke = tauriCore.invoke;
        tauriAvailable = true;
        showToast('Tauri conectado con éxito', 'success');
      } else {
        showToast('Corriendo en modo simulación (Navegador)', 'info');
      }
    } catch (e) {
      console.warn("Failed to load Tauri core APIs:", e);
    }
  });

  // App Navigation & UI State
  let activeTab = 'files';
  let isDarkMode = true;
  
  function selectTab(tab) {
    activeTab = tab;
  }

  function toggleTheme() {
    isDarkMode = !isDarkMode;
    if (isDarkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }
  
  // File management
  let loadedPdfBytes = null;
  let loadedFileName = 'Ningún archivo seleccionado';
  let loadedFileSize = '0 KB';
  let loadedFilePath = '';
  let outputSameAsSource = true;
  let outputFolder = '';
  let outputPrefix = '';
  let outputSuffix = '';
  let ocrSuffix = '_ocr';
  
  // Toasts
  let toasts = [];
  function showToast(message, type = 'info') {
    const id = Date.now() + Math.random();
    toasts = [...toasts, { id, message, type }];
    setTimeout(() => {
      toasts = toasts.filter(t => t.id !== id);
    }, 4000);
  }

  function showFileToast(message, path) {
    const id = Date.now() + Math.random();
    toasts = [...toasts, { id, message, type: 'success', path }];
    setTimeout(() => {
      toasts = toasts.filter(t => t.id !== id);
    }, 10000);
  }

  // Fast Base64 conversions for large PDF binary streams
  function uint8ToBase64(uint8) {
    const CHUNK_SIZE = 0x8000;
    let index = 0;
    const length = uint8.length;
    let result = '';
    let slice;
    while (index < length) {
      slice = uint8.subarray(index, Math.min(index + CHUNK_SIZE, length));
      result += String.fromCharCode.apply(null, slice);
      index += CHUNK_SIZE;
    }
    return window.btoa(result);
  }

  function base64ToUint8(b64) {
    const binaryString = window.atob(b64);
    const len = binaryString.length;
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes;
  }

  async function handleOpenFile(path) {
    if (tauriAvailable && invoke) {
      try {
        await invoke('open_file', { path });
        showToast('Archivo abierto', 'success');
      } catch (err) {
        showToast('Error al abrir: ' + err, 'error');
      }
    }
  }

  async function handleShowInFolder(path) {
    if (tauriAvailable && invoke) {
      try {
        await invoke('show_in_folder', { path });
        showToast('Directorio abierto', 'success');
      } catch (err) {
        showToast('Error al abrir carpeta: ' + err, 'error');
      }
    }
  }

  // Modals
  let showConfirmModal = false;
  let modalTitle = "";
  let modalMessage = "";
  let modalCallback = null;

  function triggerConfirmation(title, message, callback) {
    modalTitle = title;
    modalMessage = message;
    modalCallback = callback;
    showConfirmModal = true;
  }

  function confirmModalAction() {
    if (modalCallback) modalCallback();
    showConfirmModal = false;
  }

  // PDF.js State and Pages
  let pages = [];
  let isProcessingFile = false;
  let pdfDoc = null;

  async function renderPageCanvas(canvasElement, page) {
    if (!pdfDoc || !page.originalPageNum || page.originalPageNum > pdfDoc.numPages) {
      // Draw blank canvas for added/placeholder pages or out-of-bound pages during transition
      const canvas = canvasElement;
      const context = canvas.getContext('2d');
      canvas.width = 150;
      canvas.height = 212;
      context.fillStyle = "#ffffff";
      context.fillRect(0, 0, canvas.width, canvas.height);
      context.fillStyle = "#64748b";
      context.font = "12px sans-serif";
      context.textAlign = "center";
      context.textBaseline = "middle";
      context.fillText("Página Adicional", canvas.width / 2, canvas.height / 2);
      return;
    }
    try {
      const pdfPage = await pdfDoc.getPage(page.originalPageNum);
      const viewport = pdfPage.getViewport({ scale: 0.4 });
      const canvas = canvasElement;
      const context = canvas.getContext('2d');
      canvas.height = viewport.height;
      canvas.width = viewport.width;

      const renderContext = {
        canvasContext: context,
        viewport: viewport
      };
      await pdfPage.render(renderContext).promise;
    } catch (err) {
      console.error("Error rendering page:", err);
    }
  }

  function renderPage(node, page) {
    renderPageCanvas(node, page);
    return {
      update(newPage) {
        renderPageCanvas(node, newPage);
      }
    };
  }

  async function loadPdfBytes(bytes, name, path = '') {
    loadedFileName = name;
    loadedFilePath = path;
    loadedFileSize = (bytes.length / (1024 * 1024)).toFixed(2) + ' MB';
    loadedPdfBytes = bytes.slice();
    const pdfjsLib = await import('pdfjs-dist');
    pdfjsLib.GlobalWorkerOptions.workerSrc = new URL('pdfjs-dist/build/pdf.worker.min.mjs', import.meta.url).toString();
    pdfDoc = await pdfjsLib.getDocument({ data: bytes.slice() }).promise;
    pages = Array.from({ length: pdfDoc.numPages }, (_, i) => ({ id: i + 1, originalPageNum: i + 1, name: `Página ${i + 1}`, isOcrDone: false, rotation: 0 }));
    splitEnd = pdfDoc.numPages;
    showToast('PDF cargado correctamente', 'success');
  }

  async function choosePdfNative() {
    if (!tauriAvailable || !invoke) return;
    try {
      const file = await invoke('select_pdf_file');
      await loadPdfBytes(base64ToUint8(file.bytes_b64), file.name, file.path);
    } catch (e) { if (!String(e).includes('cancelada')) showToast(String(e), 'error'); }
  }

  async function chooseOutputFolder() {
    try { outputFolder = await invoke('choose_output_folder'); outputSameAsSource = false; }
    catch (e) { if (!String(e).includes('cancelada')) showToast(String(e), 'error'); }
  }

  async function handleFileSelect(event) {
    const file = event.target.files[0];
    if (!file) return;
    
    isProcessingFile = true;
    loadedFileName = file.name;
    loadedFileSize = (file.size / (1024 * 1024)).toFixed(2) + ' MB';
    
    showToast('Procesando PDF...', 'info');

    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const arrayBuffer = e.target.result;
        loadedPdfBytes = new Uint8Array(arrayBuffer.slice(0));

        // Load PDF.js dynamically to render previews
        const pdfjsLib = await import('pdfjs-dist');
        pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
          'pdfjs-dist/build/pdf.worker.min.mjs',
          import.meta.url
        ).toString();

        const loadingTask = pdfjsLib.getDocument({ data: loadedPdfBytes.slice() });
        const pdf = await loadingTask.promise;
        pdfDoc = pdf;
        
        let newPages = [];
        for (let i = 1; i <= pdf.numPages; i++) {
          let text = "";
          try {
            const pageObj = await pdf.getPage(i);
            const textContent = await pageObj.getTextContent();
            text = textContent.items.map(item => item.str).join(' ');
          } catch (e) {
            console.warn("Could not extract text for page " + i, e);
          }
          newPages.push({
            id: i,
            originalPageNum: i,
            name: `Página ${i}`,
            content: text || "Página sin texto descifrable (escaneada).",
            isOcrDone: false,
            isSelected: false,
            rotation: 0
          });
        }
        pages = newPages;
        isProcessingFile = false;
        showToast('PDF cargado exitosamente', 'success');
      } catch (err) {
        console.error(err);
        isProcessingFile = false;
        showToast('Error al procesar el PDF', 'error');
      }
    };
    reader.readAsArrayBuffer(file);
  }

  // Metadata & Settings
  let pdfMetadata = {
    author: 'David R.',
    keywords: 'contratos, legal, 2026, desarrollo',
    encrypt: true
  };

  // OCR state
  let ocrProgress = 0;
  let isOcrRunning = false;
  let ocrResultText = "";
  let ocrPageTarget = null;


  // Merge tool state
  let mergeFiles = []; // Array of { name, size, bytes }

  function handleMergeFileAdd(event) {
    const filesList = event.target.files;
    if (!filesList) return;

    for (let file of filesList) {
      const reader = new FileReader();
      reader.onload = (e) => {
        mergeFiles = [...mergeFiles, {
          id: Date.now() + Math.random(),
          name: file.name,
          size: (file.size / 1024).toFixed(0) + ' KB',
          bytes: new Uint8Array(e.target.result)
        }];
        showToast(`Añadido: ${file.name}`, 'info');
      };
      reader.readAsArrayBuffer(file);
    }
  }

  function moveMergeItem(index, direction) {
    const targetIdx = index + direction;
    if (targetIdx < 0 || targetIdx >= mergeFiles.length) return;
    const temp = mergeFiles[index];
    mergeFiles[index] = mergeFiles[targetIdx];
    mergeFiles[targetIdx] = temp;
    mergeFiles = [...mergeFiles];
  }

  function removeMergeItem(id) {
    mergeFiles = mergeFiles.filter(item => item.id !== id);
  }

  async function handleMerge() {
    if (mergeFiles.length < 2) {
      showToast('Por favor, selecciona al menos 2 PDFs para unir.', 'warning');
      return;
    }
    
    isProcessingFile = true;
    showToast('Uniendo PDFs en segundo plano...', 'info');
    
    try {
      await new Promise(r => setTimeout(r, 100));
      const b64Strings = mergeFiles.map(f => uint8ToBase64(f.bytes));
      
      let mergedBytes;
      if (tauriAvailable && invoke) {
        const mergedB64 = await invoke('merge_pdf_files', { filesB64: b64Strings });
        mergedBytes = base64ToUint8(mergedB64);
      } else {
        mergedBytes = mergeFiles[0].bytes;
      }
      
      await downloadPdfBlob(mergedBytes, 'merged_documento.pdf');
    } catch (err) {
      showToast('Error al unir PDFs: ' + err, 'error');
    } finally {
      isProcessingFile = false;
    }
  }

  // Split tool state
  let splitMode = "chunks";
  let splitRanges = "1-2";
  let splitStart = 1;
  let splitEnd = 1;
  let splitChunkSize = 2;
  let splitStatus = "";

  function normalizedSplitBounds() {
    const max = Math.max(1, pages.length);
    const start = Math.min(max, Math.max(1, Number(splitStart) || 1));
    const end = Math.min(max, Math.max(start, Number(splitEnd) || start));
    return { start, end };
  }

  function parseSplitRanges(value) {
    if (!value.trim()) throw new Error('Agrega al menos un rango.');
    return value.split(',').map(raw => {
      const match = raw.trim().match(/^(\d+)(?:\s*-\s*(\d+))?$/);
      if (!match) throw new Error(`Rango inválido: "${raw.trim()}". Usa 1-3, 5, 8-10.`);
      const start = Number(match[1]);
      const end = Number(match[2] || match[1]);
      if (start < 1 || end < start || end > pages.length) throw new Error(`El rango ${start}-${end} debe estar entre 1 y ${pages.length}.`);
      return [start, end];
    });
  }

  function getSplitRanges() {
    const { start, end } = normalizedSplitBounds();
    if (splitMode === 'custom') return parseSplitRanges(splitRanges);
    if (splitMode === 'even') return Array.from({ length: end - start + 1 }, (_, i) => start + i).filter(p => p % 2 === 0).map(p => [p, p]);
    if (splitMode === 'odd') return Array.from({ length: end - start + 1 }, (_, i) => start + i).filter(p => p % 2 === 1).map(p => [p, p]);
    const size = Math.max(1, Number(splitChunkSize) || 1);
    const result = [];
    for (let page = start; page <= end; page += size) result.push([page, Math.min(end, page + size - 1)]);
    return result;
  }

  function splitRangeLabel(range) {
    return range[0] === range[1] ? `Página ${range[0]}` : `Páginas ${range[0]}–${range[1]}`;
  }

  function getSplitPreview() {
    try { return getSplitRanges(); } catch { return []; }
  }

  async function handleSplit() {
    if (!loadedPdfBytes) { showToast('Por favor, carga un PDF primero.', 'warning'); return; }
    isProcessingFile = true;
    showToast('Dividiendo PDF...', 'info');
    try {
      const parsedRanges = getSplitRanges();
      if (!parsedRanges.length) throw new Error('El criterio seleccionado no incluye ninguna página.');
      if (tauriAvailable && invoke) {
        const splitResultsB64 = await invoke('split_pdf_file', { inputB64: uint8ToBase64(loadedPdfBytes), ranges: parsedRanges });
        for (let i = 0; i < splitResultsB64.length; i++) {
          await downloadPdfBlob(base64ToUint8(splitResultsB64[i]), `parte_${i + 1}_paginas_${parsedRanges[i][0]}-${parsedRanges[i][1]}.pdf`);
        }
      } else await downloadPdfBlob(loadedPdfBytes, 'parte_1.pdf');
    } catch (err) { showToast('Error al dividir: ' + err, 'error'); }
    finally { isProcessingFile = false; }
  }

  // Compression tool state
  let compressQuality = 70;
  let compressionStatus = "";

  async function handleCompress() {
    if (!loadedPdfBytes) {
      showToast('Por favor, carga un PDF primero.', 'warning');
      return;
    }
    
    isProcessingFile = true;
    compressionStatus = "Comprimiendo imágenes de PDF...";
    showToast('Procesando compresión...', 'info');

    try {
      await new Promise(r => setTimeout(r, 100));
      let compressedBytes;
      if (tauriAvailable && invoke) {
        const inputB64 = uint8ToBase64(loadedPdfBytes);
        const outputB64 = await invoke('compress_pdf_file', { 
          inputB64: inputB64, 
          quality: compressQuality 
        });
        compressedBytes = base64ToUint8(outputB64);
      } else {
        compressedBytes = loadedPdfBytes;
      }

      await downloadPdfBlob(compressedBytes, `optimized_${loadedFileName}`);
      compressionStatus = "Compresión completada con éxito.";
    } catch (err) {
      compressionStatus = `Error: ${err}`;
      showToast('Error en compresión: ' + err, 'error');
    } finally {
      isProcessingFile = false;
    }
  }

  // Native OCR Trigger
  async function renderPageForOcr(pageNumber) {
    const pdfPage = await pdfDoc.getPage(pageNumber);
    const viewport = pdfPage.getViewport({ scale: 2.5 });
    const canvas = document.createElement('canvas');
    canvas.width = Math.ceil(viewport.width);
    canvas.height = Math.ceil(viewport.height);
    await pdfPage.render({ canvasContext: canvas.getContext('2d'), viewport }).promise;
    return canvas.toDataURL('image/png').split(',')[1];
  }

  async function runOcr(pageId) {
    if (isOcrRunning || !pdfDoc) return;
    isOcrRunning = true; ocrPageTarget = pageId; ocrProgress = 15;
    try {
      if (!tauriAvailable || !invoke) throw new Error('El OCR requiere la aplicación de escritorio.');
      const target = pages.find(p => p.id === pageId);
      const imageB64 = await renderPageForOcr(target.originalPageNum);
      ocrProgress = 60;
      const rawResult = await invoke('perform_ocr', { imageB64 });
      const result = JSON.parse(rawResult);
      target.isOcrDone = true;
      target.ocrData = result;
      ocrResultText = result.text;
      pages = [...pages]; ocrProgress = 100;
      showToast('Texto reconocido correctamente', 'success');
    } catch (e) { showToast('Error OCR: ' + e, 'error'); }
    finally { isOcrRunning = false; }
  }

  async function runBatchOcr() {
    if (!loadedPdfBytes || !pdfDoc || isOcrRunning) return;
    isOcrRunning = true; ocrResultText = '';
    try {
      for (let i = 0; i < pages.length; i++) {
        ocrPageTarget = pages[i].id;
        ocrProgress = Math.round((i / pages.length) * 100);
        const raw = await invoke('perform_ocr', { imageB64: await renderPageForOcr(pages[i].originalPageNum) });
        pages[i].ocrData = JSON.parse(raw); pages[i].isOcrDone = true;
        ocrResultText += `${pages[i].name}\n${pages[i].ocrData.text}\n\n`;
        pages = [...pages];
      }
      const { PDFDocument, StandardFonts, rgb } = await import('pdf-lib');
      const doc = await PDFDocument.load(loadedPdfBytes.slice());
      const font = await doc.embedFont(StandardFonts.Helvetica);
      const pdfPages = doc.getPages();
      for (const item of pages) {
        const page = pdfPages[item.originalPageNum - 1]; const size = page.getSize();
        const sx = size.width / item.ocrData.width; const sy = size.height / item.ocrData.height;
        for (const word of item.ocrData.words) if (word.text) page.drawText(word.text, { x: word.x*sx, y: size.height-(word.y+word.height)*sy, size: Math.max(4,word.height*sy*.82), font, color: rgb(0,0,0), opacity: 0 });
      }
      const base = loadedFileName.replace(/\.pdf$/i,'');
      await downloadPdfBlob(await doc.save({ useObjectStreams: true }), `${base}${ocrSuffix || '_ocr'}.pdf`, true);
      showToast('OCR terminado y PDF creado', 'success');
    } catch (e) { showToast('Error OCR: ' + e, 'error'); }
    finally { isOcrRunning = false; ocrProgress = 100; }
  }

  // Helper to trigger browser downloads or native save dialogs
  async function downloadPdfBlob(bytes, filename, preserveOcrName = false) {
    if (tauriAvailable && invoke) {
      try {
        const bytesB64 = uint8ToBase64(bytes);
        const dot = filename.toLowerCase().endsWith('.pdf') ? filename.slice(0, -4) : filename;
        const finalName = preserveOcrName ? filename : `${outputPrefix}${dot}${outputSuffix}.pdf`;
        const sourceFolder = loadedFilePath ? loadedFilePath.replace(/[\/][^\/]+$/, '') : '';
        const targetFolder = outputSameAsSource ? sourceFolder : outputFolder;
        const savedPath = targetFolder
          ? await invoke('save_pdf_to_folder', { bytesB64, filename: finalName, folder: targetFolder })
          : await invoke('save_pdf_dialog', { bytesB64, defaultName: finalName });
        showFileToast(`Guardado: ${savedPath.split('\\').pop()}`, savedPath);
      } catch (err) {
        if (err !== "Operación cancelada") {
          showToast(`Error al guardar: ${err}`, 'error');
        } else {
          showToast('Guardado cancelado', 'warning');
        }
      }
    } else {
      const blob = new Blob([new Uint8Array(bytes)], { type: 'application/pdf' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  }

  // Drag and drop page simulation
  let draggedIdx = null;

  function dragStart(e, index) {
    draggedIdx = index;
  }

  function dragOver(e) {
    e.preventDefault();
  }

  function drop(e, targetIdx) {
    if (draggedIdx === null || draggedIdx === targetIdx) return;
    const items = [...pages];
    const [draggedItem] = items.splice(draggedIdx, 1);
    items.splice(targetIdx, 0, draggedItem);
    pages = items;
    draggedIdx = null;
  }

  function rotatePage(pageId) {
    const page = pages.find(p => p.id === pageId);
    if (page) {
      page.rotation = (page.rotation + 90) % 360;
      pages = [...pages];
      showToast(`Rotando página ${pageId} a ${page.rotation}°`, 'info');
    }
  }

  function deletePage(pageId) {
    triggerConfirmation(
      'Eliminar página',
      `¿Estás seguro de que quieres eliminar la Página ${pageId} de la estructura?`,
      () => {
        pages = pages.filter(p => p.id !== pageId);
        showToast(`Página ${pageId} eliminada`, 'info');
      }
    );
  }

  function addPage() {
    const nextId = pages.length > 0 ? Math.max(...pages.map(p => p.id)) + 1 : 1;
    pages = [...pages, {
      id: nextId,
      name: `Página ${nextId}`,
      content: `Página adicional agregada ${nextId}.`,
      isOcrDone: false,
      isSelected: false,
      rotation: 0
    }];
    showToast('Página añadida al final', 'success');
  }

  // Declared variables for rendering
  let max_id = 0;
  let showMetadataSidebar = false;
</script>

<div class="flex h-screen w-full select-none overflow-hidden {isDarkMode ? 'bg-[#0b0c10] text-[#e2e8f0]' : 'bg-[#f4f5f9] text-[#1e293b]'} transition-colors duration-300">
  
  <!-- Toast System -->
  <div class="fixed top-4 right-4 z-[100] space-y-2 pointer-events-none">
    {#each toasts as toast (toast.id)}
      <div class="flex flex-col gap-2 px-4 py-3 rounded-xl shadow-2xl border pointer-events-auto transition-all duration-300 animate-slide-in max-w-sm
        {toast.type === 'success' ? 'bg-emerald-950/90 border-emerald-500/40 text-emerald-300' : ''}
        {toast.type === 'error' ? 'bg-red-950/90 border-red-500/40 text-red-300' : ''}
        {toast.type === 'warning' ? 'bg-amber-950/90 border-amber-500/40 text-amber-300' : ''}
        {toast.type === 'info' ? 'bg-indigo-950/90 border-indigo-500/40 text-indigo-300' : ''}"
      >
        <div class="flex items-center gap-3">
          {#if toast.type === 'success'}
            <CheckCircle class="w-4 h-4 shrink-0 text-emerald-400" />
          {:else if toast.type === 'error'}
            <XCircle class="w-4 h-4 shrink-0 text-red-400" />
          {:else if toast.type === 'warning'}
            <AlertTriangle class="w-4 h-4 shrink-0 text-amber-400" />
          {:else}
            <Info class="w-4 h-4 shrink-0 text-indigo-400" />
          {/if}
          <span class="text-xs font-bold">{toast.message}</span>
        </div>
        {#if toast.path}
          <div class="flex gap-2 pl-7 mt-1">
            <button 
              on:click={() => handleOpenFile(toast.path)}
              class="px-2 py-1 text-[10px] font-bold bg-white/10 hover:bg-white/20 active:scale-95 rounded text-white transition-all cursor-pointer"
            >
              Abrir Archivo
            </button>
            <button 
              on:click={() => handleShowInFolder(toast.path)}
              class="px-2 py-1 text-[10px] font-bold bg-white/10 hover:bg-white/20 active:scale-95 rounded text-white transition-all cursor-pointer"
            >
              Abrir Carpeta
            </button>
          </div>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Global Glass Processing Overlay -->
  {#if isProcessingFile}
    <div class="fixed inset-0 bg-slate-950/60 backdrop-blur-md z-[80] flex flex-col items-center justify-center gap-4">
      <RefreshCw class="w-12 h-12 animate-spin text-indigo-400" />
      <h3 class="text-lg font-bold text-white tracking-wide">Procesando archivo...</h3>
      <p class="text-xs text-slate-400 font-medium">Por favor, espera un momento</p>
    </div>
  {/if}

  <!-- Confirmation Modal -->
  {#if showConfirmModal}
    <div class="fixed inset-0 bg-slate-950/80 backdrop-blur-sm z-[90] flex items-center justify-center p-4">
      <div class="glass-effect max-w-sm w-full rounded-2xl p-6 space-y-6 animate-zoom-in">
        <div class="flex gap-4">
          <div class="w-10 h-10 rounded-full bg-red-500/20 text-red-500 flex items-center justify-center shrink-0">
            <AlertTriangle class="w-5 h-5" />
          </div>
          <div>
            <h4 class="text-sm font-bold text-white">{modalTitle}</h4>
            <p class="text-xs text-slate-400 mt-1">{modalMessage}</p>
          </div>
        </div>
        <div class="flex gap-3 justify-end">
          <button 
            on:click={() => showConfirmModal = false}
            class="px-4 py-2 text-xs font-bold bg-white/5 border border-white/10 rounded-lg hover:bg-white/10 text-white cursor-pointer"
          >
            Cancelar
          </button>
          <button 
            on:click={confirmModalAction}
            class="px-4 py-2 text-xs font-bold bg-red-600 hover:bg-red-500 rounded-lg text-white cursor-pointer"
          >
            Confirmar
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Left Glass SideNavBar -->
  <aside class="w-[250px] h-full glass-nav flex flex-col justify-between py-6 px-4 z-40 relative font-sans">
    <div>
      <div class="mb-10 px-4 flex items-center justify-between">
        <div>
          <h1 class="text-xl font-bold tracking-tight bg-gradient-to-r from-indigo-400 to-emerald-400 bg-clip-text text-transparent">Precision PDF</h1>
          <p class="text-[10px] uppercase tracking-wider text-slate-400">Desktop Suite</p>
        </div>
        <button 
          on:click={toggleTheme} 
          class="p-2 rounded-full hover:bg-white/5 active:scale-95 transition-all text-slate-400 hover:text-indigo-400 cursor-pointer"
          title="Cambiar Tema"
        >
          {#if isDarkMode}
            <Sun class="w-5 h-5" />
          {:else}
            <Moon class="w-5 h-5" />
          {/if}
        </button>
      </div>

      <nav class="space-y-2">
        <button 
          on:click={() => selectTab('files')}
          class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 active:scale-[0.98] cursor-pointer {activeTab === 'files' ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/20 shadow-[0_0_15px_rgba(79,70,229,0.15)]' : 'text-slate-400 hover:bg-white/5'}"
        >
          <FolderOpen class="w-5 h-5" />
          <span class="text-sm font-semibold">Mis Archivos</span>
        </button>

        <button 
          on:click={() => selectTab('merge')}
          class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 active:scale-[0.98] cursor-pointer {activeTab === 'merge' ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/20 shadow-[0_0_15px_rgba(79,70,229,0.15)]' : 'text-slate-400 hover:bg-white/5'}"
        >
          <GitMerge class="w-5 h-5" />
          <span class="text-sm font-semibold">Unir PDF</span>
        </button>

        <button 
          on:click={() => selectTab('split')}
          class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 active:scale-[0.98] cursor-pointer {activeTab === 'split' ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/20 shadow-[0_0_15px_rgba(79,70,229,0.15)]' : 'text-slate-400 hover:bg-white/5'}"
        >
          <Scissors class="w-5 h-5" />
          <span class="text-sm font-semibold">Dividir</span>
        </button>

        <button 
          on:click={() => selectTab('compress')}
          class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 active:scale-[0.98] cursor-pointer {activeTab === 'compress' ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/20 shadow-[0_0_15px_rgba(79,70,229,0.15)]' : 'text-slate-400 hover:bg-white/5'}"
        >
          <Minimize2 class="w-5 h-5" />
          <span class="text-sm font-semibold">Comprimir</span>
        </button>

        <button 
          on:click={() => selectTab('ocr')}
          class="w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 active:scale-[0.98] cursor-pointer {activeTab === 'ocr' ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/20 shadow-[0_0_15px_rgba(79,70,229,0.15)]' : 'text-slate-400 hover:bg-white/5'}"
        >
          <ScanText class="w-5 h-5" />
          <span class="text-sm font-semibold">OCR por lotes</span>
        </button>
        <button on:click={() => selectTab('settings')} class="w-full flex items-center gap-3 px-4 py-3 rounded-xl {activeTab === 'settings' ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/20' : 'text-slate-400 hover:bg-white/5'}"><Settings class="w-5 h-5" /><span class="text-sm font-semibold">Configuración</span></button>
      </nav>
    </div>

    <!-- User & Version Profile Card -->
    <div class="px-4 py-3 rounded-xl border border-white/5 bg-white/[0.02] flex items-center gap-3">
      <div class="w-9 h-9 rounded-full bg-gradient-to-tr from-indigo-500 to-emerald-500 flex items-center justify-center font-bold text-sm text-white">
        DR
      </div>
      <div class="overflow-hidden">
        <p class="text-xs font-bold truncate">David Ramos</p>
        <p class="text-[9px] text-slate-500">v2.5.0 Premium (Win)</p>
      </div>
    </div>
  </aside>

  <!-- Main Workstation -->
  <main class="flex-1 flex flex-col relative overflow-hidden font-sans">
    
    <!-- Glass Header -->
    <header class="h-[64px] w-full glass-header flex justify-between items-center px-8 z-30">
      <div class="flex items-center gap-3">
        <FileText class="w-5 h-5 text-indigo-400" />
        <h2 class="text-sm font-bold tracking-tight">{loadedFileName}</h2>
        <span class="px-2 py-0.5 rounded-md bg-white/5 border border-white/10 text-slate-400 text-[10px] font-semibold">{loadedFileSize}</span>
      </div>
      
      <div class="flex items-center gap-4">
        {#if loadedPdfBytes}
          <button 
            on:click={() => showMetadataSidebar = !showMetadataSidebar}
            class="px-3 py-2 text-xs font-bold {showMetadataSidebar ? 'bg-indigo-600 text-white' : 'bg-white/5 hover:bg-white/10 text-slate-300'} border border-white/10 rounded-lg cursor-pointer transition-all active:scale-95 flex items-center gap-2"
            title="Metadatos del PDF"
          >
            <Info class="w-4 h-4" />
            Metadatos
          </button>
        {/if}
        <label class="px-4 py-2 text-xs font-bold bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg cursor-pointer transition-all active:scale-95">
          Cargar PDF
          <input type="file" accept="application/pdf" class="hidden" on:change={handleFileSelect} />
        </label>
      </div>
    </header>

    <!-- Content Workspace -->
    <div class="flex-1 overflow-auto bg-[#08090c] p-8 flex items-start justify-center">
      
      <!-- TAB 1: FILES & DRAG AND DROP PAGES -->
      {#if activeTab === 'files'}
        <div class="max-w-6xl w-full space-y-6">
          <div class="flex justify-between items-center">
            <div>
              <h3 class="text-lg font-bold text-slate-200">Organizar Páginas</h3>
              <p class="text-xs text-slate-400">Arrastra y suelta para reordenar las páginas de tu PDF de forma visual.</p>
            </div>
            <button 
              on:click={addPage}
              class="flex items-center gap-2 px-4 py-2 text-xs font-semibold bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg shadow-[0_0_15px_rgba(16,185,129,0.2)] transition-all cursor-pointer"
            >
              <Plus class="w-4 h-4" />
              Insertar Página
            </button>
          </div>

          {#if isProcessingFile}
            <div class="w-full py-20 flex flex-col items-center justify-center gap-3 text-slate-400">
              <RefreshCw class="w-8 h-8 animate-spin text-indigo-400" />
              <p class="text-sm">Renderizando páginas con PDF.js...</p>
            </div>
          {:else if pages.length === 0}
            <div class="border-2 border-dashed border-white/10 rounded-2xl p-16 text-center space-y-4">
              <FolderOpen class="w-12 h-12 text-slate-600 mx-auto" />
              <div>
                <p class="text-sm font-bold text-slate-400">No hay ningún PDF cargado</p>
                <p class="text-xs text-slate-500 mt-1">Haz clic en el botón superior para seleccionar un PDF de tu disco local.</p>
              </div>
            </div>
          {:else}
            <!-- Page Grid -->
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
              {#each pages as page, idx (page.id)}
                <div 
                  draggable="true"
                  on:dragstart={(e) => dragStart(e, idx)}
                  on:dragover={dragOver}
                  on:drop={(e) => drop(e, idx)}
                  class="group relative space-y-2 bg-white/[0.02] border border-white/5 hover:border-indigo-500/40 rounded-xl p-4 transition-all duration-300 shadow-lg hover:shadow-indigo-500/5 cursor-move"
                >
                  <!-- Page Canvas with rotation -->
                  <div 
                    class="aspect-[1/1.41] bg-white rounded-lg shadow-[0_8px_30px_rgb(0,0,0,0.4)] relative border border-slate-700/50 overflow-hidden transition-transform duration-300"
                    style="transform: rotate({page.rotation}deg)"
                  >
                    <canvas use:renderPage={page} class="w-full h-full object-contain"></canvas>

                    <!-- Scanline effect for OCR processing -->
                    {#if isOcrRunning && ocrPageTarget === page.id}
                      <div class="absolute inset-0 bg-indigo-500/10 pointer-events-none">
                        <div class="w-full h-0.5 bg-gradient-to-r from-transparent via-indigo-400 to-transparent absolute animate-pulse" style="top: {ocrProgress}%"></div>
                      </div>
                    {/if}

                    <!-- Hover Control Overlay -->
                    <div class="absolute inset-0 bg-slate-950/70 opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex items-center justify-center gap-2">
                      <button 
                        on:click={() => rotatePage(page.id)}
                        class="w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition-all hover:scale-110 cursor-pointer" 
                        title="Rotar"
                      >
                        <RotateCw class="w-4 h-4" />
                      </button>
                      <button 
                        on:click={() => runOcr(page.id)}
                        class="w-8 h-8 rounded-full bg-indigo-600/80 hover:bg-indigo-600 text-white flex items-center justify-center transition-all hover:scale-110 cursor-pointer" 
                        title="OCR Nivel de Página"
                      >
                        <ScanText class="w-4 h-4" />
                      </button>
                      <button 
                        on:click={() => deletePage(page.id)}
                        class="w-8 h-8 rounded-full bg-red-600/80 hover:bg-red-600 text-white flex items-center justify-center transition-all hover:scale-110 cursor-pointer" 
                        title="Eliminar"
                      >
                        <Trash2 class="w-4 h-4" />
                      </button>
                    </div>
                  </div>

                  <div class="flex justify-between items-center px-1">
                    <span class="text-xs font-semibold text-slate-400">{page.name}</span>
                    {#if page.isOcrDone}
                      <span class="text-[9px] text-emerald-400 font-bold bg-emerald-400/10 px-1.5 py-0.5 rounded flex items-center gap-1">
                        <CheckCircle2 class="w-3.5 h-3.5" />
                        OCR
                      </span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

      <!-- TAB 2: MERGE FILES -->
      {:else if activeTab === 'merge'}
        <div class="max-w-2xl w-full glass-effect rounded-2xl p-8 space-y-6">
          <div class="flex justify-between items-center border-b border-white/5 pb-4">
            <div class="flex items-center gap-3">
              <GitMerge class="w-6 h-6 text-indigo-400" />
              <div>
                <h3 class="text-lg font-bold">Unir Múltiples PDFs</h3>
                <p class="text-xs text-slate-400">Combina varios archivos PDF en un único documento de alto rendimiento.</p>
              </div>
            </div>
            
            <label class="px-4 py-2 text-xs font-bold bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg cursor-pointer transition-all active:scale-95">
              Agregar Archivos
              <input type="file" accept="application/pdf" multiple class="hidden" on:change={handleMergeFileAdd} />
            </label>
          </div>

          <div class="space-y-3">
            {#if mergeFiles.length === 0}
              <div class="py-10 text-center text-slate-500 text-xs">
                Usa el botón superior para agregar los PDFs que deseas unir.
              </div>
            {:else}
              {#each mergeFiles as item, index}
                <div class="flex items-center justify-between p-4 bg-white/[0.02] border border-white/5 rounded-xl">
                  <div class="flex items-center gap-3">
                    <FileText class="w-8 h-8 text-red-400" />
                    <div>
                      <p class="text-sm font-semibold text-slate-200">{item.name}</p>
                      <p class="text-[10px] text-slate-500">{item.size}</p>
                    </div>
                  </div>
                  <div class="flex gap-2">
                    <button 
                      on:click={() => moveMergeItem(index, -1)}
                      class="p-1.5 hover:bg-white/5 rounded text-slate-400 hover:text-slate-200 cursor-pointer"
                      title="Mover arriba"
                    >
                      <ArrowUp class="w-4 h-4" />
                    </button>
                    <button 
                      on:click={() => moveMergeItem(index, 1)}
                      class="p-1.5 hover:bg-white/5 rounded text-slate-400 hover:text-slate-200 cursor-pointer"
                      title="Mover abajo"
                    >
                      <ArrowDown class="w-4 h-4" />
                    </button>
                    <button 
                      on:click={() => removeMergeItem(item.id)}
                      class="p-1.5 hover:bg-red-500/20 rounded text-red-400 cursor-pointer"
                      title="Quitar"
                    >
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </div>
                </div>
              {/each}
            {/if}
          </div>

          <button 
            on:click={handleMerge}
            class="w-full py-3 text-sm font-bold bg-indigo-600 hover:bg-indigo-500 rounded-xl shadow-lg active:scale-95 transition-all text-white cursor-pointer"
          >
            Unir Archivos ahora (Paralelizado)
          </button>
        </div>

      <!-- TAB 3: SPLIT PDF -->
      {:else if activeTab === 'split'}
        <div class="max-w-2xl w-full glass-effect rounded-2xl p-8 space-y-6">
          <div class="flex items-center gap-3 border-b border-white/5 pb-4">
            <Scissors class="w-6 h-6 text-indigo-400" />
            <div><h3 class="text-lg font-bold">Dividir archivo PDF</h3><p class="text-xs text-slate-400">Elige c?mo quieres crear los archivos nuevos.</p></div>
          </div>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-2">
            {#each [['chunks','Cada X páginas'],['even','Páginas pares'],['odd','Páginas impares'],['custom','Rangos manuales']] as option}
              <button on:click={() => splitMode = option[0]} class="p-3 rounded-xl text-xs font-bold border transition-all {splitMode === option[0] ? 'bg-indigo-600 border-indigo-400 text-white' : 'bg-white/[0.03] border-white/10 text-slate-400'}">{option[1]}</button>
            {/each}
          </div>
          {#if splitMode !== 'custom'}
            <div class="p-4 bg-white/[0.03] border border-white/10 rounded-xl space-y-4">
              <div class="flex justify-between text-xs font-bold"><span>Empezar en la página {splitStart}</span><span>Terminar en la página {splitEnd}</span></div>
              <input aria-label="Página inicial" type="range" min="1" max={Math.max(1,pages.length)} bind:value={splitStart} class="w-full accent-indigo-500" />
              <input aria-label="Página final" type="range" min="1" max={Math.max(1,pages.length)} bind:value={splitEnd} class="w-full accent-emerald-500" />
              {#if splitMode === 'chunks'}
                <label class="block text-xs font-bold text-slate-300">Crear un archivo por cada
                  <input type="number" min="1" max={Math.max(1,pages.length)} bind:value={splitChunkSize} class="mx-2 w-20 bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-center" /> páginas
                </label>
              {/if}
            </div>
          {:else}
            <div><label for="split-ranges-input" class="block text-xs font-bold text-slate-400 uppercase mb-2">Rangos separados por comas</label><input id="split-ranges-input" bind:value={splitRanges} class="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-sm text-white font-mono" placeholder="Ejemplo: 1-3, 5, 8-10" /></div>
          {/if}
          <div class="p-4 bg-indigo-500/10 border border-indigo-500/20 rounded-xl">
            <p class="text-xs font-bold text-indigo-300 mb-2">Vista previa</p>
            <p class="text-xs text-slate-300">Se crearán {getSplitPreview().length} archivo{getSplitPreview().length === 1 ? '' : 's'}:</p>
            <div class="flex flex-wrap gap-2 mt-3">{#each getSplitPreview().slice(0, 12) as range}<span class="px-2 py-1 bg-white/5 rounded text-[11px]">{splitRangeLabel(range)}</span>{/each}{#if getSplitPreview().length > 12}<span class="text-[11px] text-slate-500">y {getSplitPreview().length - 12} más…</span>{/if}</div>
          </div>
          <button on:click={handleSplit} class="w-full py-3 text-sm font-bold bg-indigo-600 hover:bg-indigo-500 rounded-xl text-white">Crear {getSplitPreview().length} archivo{getSplitPreview().length === 1 ? '' : 's'}</button>
        </div>

      <!-- TAB 4: COMPRESS -->
      {:else if activeTab === 'compress'}
        <div class="max-w-2xl w-full glass-effect rounded-2xl p-8 space-y-6">
          <div class="flex items-center gap-3 border-b border-white/5 pb-4">
            <Minimize2 class="w-6 h-6 text-indigo-400" />
            <div>
              <h3 class="text-lg font-bold">Optimizar y Comprimir</h3>
              <p class="text-xs text-slate-400">Recomprime imágenes integradas usando compresión en paralelo.</p>
            </div>
          </div>

          <div class="space-y-6">
            <div>
              <div class="flex justify-between items-center mb-2">
                <span class="text-xs font-bold text-slate-400 uppercase">Calidad de Imágenes</span>
                <span class="text-sm font-bold text-indigo-400">{compressQuality}%</span>
              </div>
              <input 
                type="range" 
                min="10" 
                max="100" 
                bind:value={compressQuality}
                class="w-full accent-indigo-500 cursor-pointer"
              />
            </div>

            <div class="grid grid-cols-3 gap-4 text-center">
              <div class="p-3 bg-white/[0.02] border border-white/5 rounded-xl">
                <p class="text-xs text-slate-500">Estimado anterior</p>
                <p class="text-sm font-bold">{loadedFileSize === '0 KB' ? '4.2 MB' : loadedFileSize}</p>
              </div>
              <div class="p-3 bg-white/[0.02] border border-white/5 rounded-xl">
                <p class="text-xs text-indigo-400">Estimado nuevo</p>
                <p class="text-sm font-bold text-indigo-400">
                  {(parseFloat(loadedFileSize === '0 KB' ? '4.2' : loadedFileSize) * (compressQuality/100)).toFixed(1)} MB
                </p>
              </div>
              <div class="p-3 bg-white/[0.02] border border-white/5 rounded-xl">
                <p class="text-xs text-emerald-400">Ahorro</p>
                <p class="text-sm font-bold text-emerald-400">~{Math.round(100 - compressQuality)}%</p>
              </div>
            </div>

            {#if compressionStatus}
              <div class="p-3 bg-indigo-500/10 border border-indigo-500/20 rounded-lg text-xs text-indigo-400">
                {compressionStatus}
              </div>
            {/if}

            <button 
              on:click={handleCompress}
              class="w-full py-3 text-sm font-bold bg-indigo-600 hover:bg-indigo-500 rounded-xl shadow-lg active:scale-95 transition-all text-white cursor-pointer"
            >
              Comprimir Documento
            </button>
          </div>
        </div>

      <!-- TAB 5: BATCH OCR -->
      {:else if activeTab === 'ocr'}
        <div class="max-w-4xl w-full grid grid-cols-1 md:grid-cols-2 gap-8">
          <div class="glass-effect rounded-2xl p-8 space-y-6">
            <div class="flex items-center gap-3 border-b border-white/5 pb-4">
              <ScanText class="w-6 h-6 text-indigo-400" />
              <div>
                <h3 class="text-lg font-bold">Procesamiento OCR Nativo</h3>
                <p class="text-xs text-slate-400">Ejecuta OCR sobre las páginas utilizando la API nativa de Windows.</p>
              </div>
            </div>

            <div class="space-y-3"><label class="block text-xs font-bold text-slate-400">Sufijo del PDF OCR</label><input bind:value={ocrSuffix} class="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3" /><button on:click={runBatchOcr} disabled={isOcrRunning || !loadedPdfBytes} class="w-full py-3 bg-indigo-600 rounded-xl font-bold disabled:opacity-40">{isOcrRunning ? `Procesando ${ocrProgress}%` : 'Crear PDF con OCR'}</button></div>
            <div class="space-y-4">
              <div class="p-4 bg-white/[0.02] border border-white/5 rounded-xl">
                <h4 class="text-xs font-bold text-slate-400 uppercase mb-3">Páginas de Documento</h4>
                <div class="space-y-2">
                  {#if pages.length === 0}
                    <div class="py-4 text-center text-slate-500 text-xs">
                      Carga un archivo PDF para listar sus páginas.
                    </div>
                  {:else}
                    {#each pages as page}
                      <div class="flex justify-between items-center p-2 hover:bg-white/5 rounded-lg">
                        <span class="text-xs font-semibold">{page.name}</span>
                        {#if page.isOcrDone}
                          <span class="text-[10px] text-emerald-400 font-bold bg-emerald-400/10 px-2 py-0.5 rounded">Reconocido</span>
                        {:else}
                          <button 
                            on:click={() => runOcr(page.id)}
                            class="text-[10px] text-indigo-400 hover:text-indigo-300 font-bold border border-indigo-500/30 hover:border-indigo-500 px-2 py-0.5 rounded bg-indigo-500/5 transition-all cursor-pointer"
                          >
                            Escanear
                          </button>
                        {/if}
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
            </div>
          </div>

          <div class="glass-effect rounded-2xl p-8 flex flex-col">
            <h3 class="text-sm font-bold text-slate-400 uppercase tracking-wider mb-4">Texto Extraído</h3>
            <textarea 
              aria-label="Texto Extraído del OCR"
              class="flex-1 w-full bg-white/[0.02] border border-white/5 focus:border-indigo-500/50 rounded-xl p-4 text-xs font-mono outline-none resize-none text-slate-300"
              placeholder="El texto digitalizado aparecerá aquí tras el escaneo..."
              bind:value={ocrResultText}
            ></textarea>
          </div>
        </div>
      {:else if activeTab === 'settings'}
        <div class="max-w-2xl w-full glass-effect rounded-2xl p-8 space-y-6">
          <div class="flex items-center gap-3 border-b border-white/5 pb-4"><Settings class="w-6 h-6 text-indigo-400" /><div><h3 class="text-lg font-bold">Configuración global</h3><p class="text-xs text-slate-400">Define dónde y con qué nombre se guardan los PDF.</p></div></div>
          <label class="flex gap-3 items-center p-4 bg-white/[0.03] rounded-xl"><input type="checkbox" bind:checked={outputSameAsSource} class="accent-indigo-500" /><span class="text-sm">Guardar en la misma carpeta del PDF original</span></label>
          {#if !outputSameAsSource}<div class="flex gap-2"><input readonly value={outputFolder || 'Ninguna carpeta seleccionada'} class="flex-1 bg-white/5 border border-white/10 rounded-xl px-4 text-sm" /><button on:click={chooseOutputFolder} class="px-4 py-3 bg-indigo-600 rounded-xl text-sm font-bold">Elegir carpeta</button></div>{/if}
          <div class="grid grid-cols-2 gap-4"><label class="text-xs font-bold text-slate-400">Prefijo global<input bind:value={outputPrefix} placeholder="Ej. procesado_" class="mt-2 w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white" /></label><label class="text-xs font-bold text-slate-400">Sufijo global<input bind:value={outputSuffix} placeholder="Ej. _final" class="mt-2 w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white" /></label></div>
          <p class="text-xs text-slate-500">El OCR usa su sufijo espec?fico <strong>{ocrSuffix}</strong>. Las dem?s herramientas usan el prefijo y sufijo globales.</p>
        </div>
      {/if}

    </div>

    <!-- Side Panel (Properties / Metadata) -->
    {#if showMetadataSidebar && loadedPdfBytes}
      <aside class="absolute right-0 top-[64px] bottom-8 w-[300px] glass-nav border-l border-white/5 flex flex-col justify-between p-6 z-25 animate-slide-in">
        <div>
          <h3 class="text-xs font-bold uppercase tracking-widest text-indigo-400 mb-6">Metadatos del PDF</h3>
          
          <div class="space-y-6">
            <div>
              <label for="pdf-author-input" class="block text-[10px] font-bold uppercase text-slate-500 mb-2">Autor del Documento</label>
              <input 
                id="pdf-author-input"
                type="text" 
                bind:value={pdfMetadata.author}
                class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-xs focus:border-indigo-500 outline-none text-slate-300"
              />
            </div>

            <div>
              <label for="pdf-keywords-input" class="block text-[10px] font-bold uppercase text-slate-500 mb-2">Palabras Clave</label>
              <textarea 
                id="pdf-keywords-input"
                bind:value={pdfMetadata.keywords}
                class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-xs focus:border-indigo-500 outline-none h-20 resize-none text-slate-300"
              ></textarea>
            </div>

            <div class="flex items-center gap-3 p-3 bg-white/[0.02] border border-white/5 rounded-xl">
              <input 
                type="checkbox" 
                bind:checked={pdfMetadata.encrypt} 
                id="aes" 
                class="accent-indigo-500 rounded cursor-pointer"
              />
              <label for="aes" class="text-xs font-semibold cursor-pointer">Cifrado AES-256 de Seguridad</label>
            </div>
          </div>
        </div>

        <button class="w-full py-3 bg-indigo-600 hover:bg-indigo-500 rounded-xl text-xs font-bold text-white shadow-lg active:scale-95 transition-all cursor-pointer">
          Aplicar Metadatos
        </button>
      </aside>
    {/if}

    <!-- Bottom Status Bar -->
    <footer class="fixed bottom-0 left-0 w-full h-8 z-50 glass-header border-t border-white/5 flex items-center justify-between px-6">
      <div class="flex items-center gap-4 text-[10px] text-slate-500">
        <span class="flex items-center gap-1 text-emerald-400">
          <CheckCircle2 class="w-3.5 h-3.5" />
          Motor OCR Listo (Nativo)
        </span>
        <span class="w-[1px] h-3 bg-white/10"></span>
        <span>Páginas Cargadas: {pages.length}</span>
      </div>
      <div class="flex items-center gap-4 text-[10px] text-slate-500">
        <span class="flex items-center gap-1">
          <RefreshCw class="w-3 h-3 animate-spin" />
          Sincronizado
        </span>
      </div>
    </footer>

  </main>
</div>

<style>
  /* Extra transitions and keyframes for toast/modals */
  @keyframes slide-in {
    from { transform: translateY(-20px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
  @keyframes zoom-in {
    from { transform: scale(0.95); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
  }
  .animate-slide-in {
    animation: slide-in 0.3s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }
  .animate-zoom-in {
    animation: zoom-in 0.2s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }
</style>

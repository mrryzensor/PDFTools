pub trait OcrEngine {
    fn recognize_text(&self, image_path: &str) -> Result<String, String>;
}

#[cfg(target_os = "windows")]
pub struct WindowsOcr;

#[cfg(target_os = "windows")]
impl OcrEngine for WindowsOcr {
    fn recognize_text(&self, image_path: &str) -> Result<String, String> {
        use windows::Storage::StorageFile;
        use windows::Graphics::Imaging::BitmapDecoder;
        use windows::Media::Ocr::OcrEngine as WinOcrEngine;

        let path = std::path::Path::new(image_path);
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|p| p.join(path))
                .map_err(|e| e.to_string())?
        };

        let path_str = abs_path.to_string_lossy().to_string();
        // Remove UNC prefix if present
        let clean_path = path_str.trim_start_matches(r"\\?\").to_string();

        let file = StorageFile::GetFileFromPathAsync(&windows::core::HSTRING::from(&clean_path))
            .map_err(|e| e.to_string())?
            .get()
            .map_err(|e| e.to_string())?;

        let stream = file.OpenAsync(windows::Storage::FileAccessMode::Read)
            .map_err(|e| e.to_string())?
            .get()
            .map_err(|e| e.to_string())?;

        let decoder = BitmapDecoder::CreateAsync(&stream)
            .map_err(|e| e.to_string())?
            .get()
            .map_err(|e| e.to_string())?;

        let software_bitmap = decoder.GetSoftwareBitmapAsync()
            .map_err(|e| e.to_string())?
            .get()
            .map_err(|e| e.to_string())?;

        let engine = WinOcrEngine::TryCreateFromUserProfileLanguages()
            .map_err(|e| e.to_string())?;

        let ocr_result = engine.RecognizeAsync(&software_bitmap)
            .map_err(|e| e.to_string())?
            .get()
            .map_err(|e| e.to_string())?;

        let text = ocr_result.Text().map_err(|e| e.to_string())?.to_string();
        let lines = ocr_result.Lines().map_err(|e| e.to_string())?;
        let mut words = Vec::new();
        for i in 0..lines.Size().map_err(|e| e.to_string())? {
            let line = lines.GetAt(i).map_err(|e| e.to_string())?;
            let line_words = line.Words().map_err(|e| e.to_string())?;
            for j in 0..line_words.Size().map_err(|e| e.to_string())? {
                let word = line_words.GetAt(j).map_err(|e| e.to_string())?;
                let rect = word.BoundingRect().map_err(|e| e.to_string())?;
                words.push(serde_json::json!({"text": word.Text().map_err(|e| e.to_string())?.to_string(), "x": rect.X, "y": rect.Y, "width": rect.Width, "height": rect.Height}));
            }
        }
        let width = software_bitmap.PixelWidth().map_err(|e| e.to_string())?;
        let height = software_bitmap.PixelHeight().map_err(|e| e.to_string())?;
        serde_json::to_string(&serde_json::json!({"text": text, "width": width, "height": height, "words": words})).map_err(|e| e.to_string())
    }
}

#[cfg(target_os = "macos")]
pub struct MacOcr;

#[cfg(target_os = "macos")]
impl OcrEngine for MacOcr {
    fn recognize_text(&self, image_path: &str) -> Result<String, String> {
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let macos_dir = executable.parent().ok_or("No se pudo resolver el bundle de macOS")?;
        let candidates = [
            macos_dir.join("precision-pdf-ocr"),
            macos_dir.join("../Resources/bin/precision-pdf-ocr"),
            std::path::PathBuf::from("src-tauri/bin/precision-pdf-ocr"),
        ];
        let helper = candidates.iter().find(|path| path.exists()).ok_or("No se encontr? el motor OCR nativo de macOS")?;
        let output = std::process::Command::new(helper).arg(image_path).output().map_err(|e| format!("No se pudo iniciar Vision OCR: {e}"))?;
        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).trim().to_string()); }
        String::from_utf8(output.stdout).map_err(|e| e.to_string())
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub struct PlaceholderOcr;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
impl OcrEngine for PlaceholderOcr {
    fn recognize_text(&self, _image_path: &str) -> Result<String, String> { Err("OCR nativo no disponible en esta plataforma.".to_string()) }
}

pub fn get_ocr_engine() -> Box<dyn OcrEngine> {
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsOcr)
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(MacOcr)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Box::new(PlaceholderOcr)
    }
}

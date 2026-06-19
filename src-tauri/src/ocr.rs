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

        let text = ocr_result.Text().map_err(|e| e.to_string())?;
        Ok(text.to_string())
    }
}

#[cfg(not(target_os = "windows"))]
pub struct PlaceholderOcr;

#[cfg(not(target_os = "windows"))]
impl OcrEngine for PlaceholderOcr {
    fn recognize_text(&self, _image_path: &str) -> Result<String, String> {
        Ok("OCR nativo no disponible en esta plataforma de compilación.".to_string())
    }
}

pub fn get_ocr_engine() -> Box<dyn OcrEngine> {
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsOcr)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::new(PlaceholderOcr)
    }
}

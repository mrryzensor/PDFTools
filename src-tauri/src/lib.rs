mod ocr;
mod pdf_ops;

use base64::{Engine as _, engine::general_purpose::STANDARD};

fn decode_b64(b64: &str) -> Result<Vec<u8>, String> {
    {
    let bytes = STANDARD.decode(b64).map_err(|e| format!("Error decodificando Base64: {}", e))?;
    if bytes.len() < 5 || &bytes[..5] != b"%PDF-" { return Err("El archivo recibido no contiene una cabecera PDF válida. Vuelve a cargarlo.".into()); }
    Ok(bytes)
}
}

fn encode_b64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

#[tauri::command]
fn merge_pdf_files(files_b64: Vec<String>) -> Result<String, String> {
    let mut files = Vec::new();
    for f_b64 in files_b64 {
        files.push(decode_b64(&f_b64)?);
    }
    let output_bytes = pdf_ops::merge_pdfs_mem(files)?;
    Ok(encode_b64(&output_bytes))
}

#[tauri::command]
fn compress_pdf_file(input_b64: String, quality: u8) -> Result<String, String> {
    let input_bytes = decode_b64(&input_b64)?;
    let output_bytes = pdf_ops::compress_pdf_mem(&input_bytes, quality)?;
    Ok(encode_b64(&output_bytes))
}

#[tauri::command]
fn split_pdf_file(input_b64: String, ranges: Vec<(u32, u32)>) -> Result<Vec<String>, String> {
    let input_bytes = decode_b64(&input_b64)?;
    let split_results = pdf_ops::split_pdf_mem(&input_bytes, ranges)?;
    
    let mut results_b64 = Vec::new();
    for res in split_results {
        results_b64.push(encode_b64(&res));
    }
    Ok(results_b64)
}

#[tauri::command]
fn perform_ocr(image_b64: String) -> Result<String, String> {
    let image_bytes = STANDARD.decode(&image_b64).map_err(|e| format!("Imagen OCR inválida: {}", e))?;
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("tauri_ocr_temp.png");
    std::fs::write(&temp_path, image_bytes).map_err(|e| e.to_string())?;
    
    let engine = ocr::get_ocr_engine();
    let result = engine.recognize_text(&temp_path.to_string_lossy());
    
    let _ = std::fs::remove_file(temp_path);
    result
}


#[derive(serde::Serialize)]
struct SelectedPdf { name: String, path: String, bytes_b64: String }

#[tauri::command]
fn select_pdf_file() -> Result<SelectedPdf, String> {
    let path = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_file().ok_or("Operación cancelada")?;
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    Ok(SelectedPdf { name: path.file_name().unwrap_or_default().to_string_lossy().to_string(), path: path.to_string_lossy().to_string(), bytes_b64: encode_b64(&bytes) })
}

#[tauri::command]
fn choose_output_folder() -> Result<String, String> {
    rfd::FileDialog::new().pick_folder().map(|p| p.to_string_lossy().to_string()).ok_or("Operación cancelada".into())
}

#[tauri::command]
fn save_pdf_to_folder(bytes_b64: String, filename: String, folder: String) -> Result<String, String> {
    let bytes = decode_b64(&bytes_b64)?;
    let dir = std::path::PathBuf::from(folder);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(filename);
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn save_pdf_dialog(bytes_b64: String, default_name: String) -> Result<String, String> {
    let bytes = decode_b64(&bytes_b64)?;
    let file_path = rfd::FileDialog::new()
        .set_file_name(&default_name)
        .add_filter("PDF Document", &["pdf"])
        .save_file();

    if let Some(path) = file_path {
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        Ok(path.to_string_lossy().to_string())
    } else {
        Err("Operación cancelada".to_string())
    }
}

#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    let clean_path = path.replace("/", "\\");
    std::process::Command::new("cmd")
        .args(&["/c", "start", "", &clean_path])
        .spawn()
        .map_err(|e| format!("No se pudo abrir el archivo: {}", e))?;
    Ok(())
}

#[tauri::command]
fn show_in_folder(path: String) -> Result<(), String> {
    let clean_path = path.replace("/", "\\");
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", clean_path))
        .spawn()
        .map_err(|e| format!("No se pudo abrir el directorio: {}", e))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            merge_pdf_files,
            compress_pdf_file,
            split_pdf_file,
            perform_ocr,
            select_pdf_file,
            choose_output_folder,
            save_pdf_to_folder,
            save_pdf_dialog,
            open_file,
            show_in_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use lopdf::{Document, Object, ObjectId, Dictionary};
use rayon::prelude::*;

pub fn merge_pdfs_mem(files: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
    if files.is_empty() {
        return Err("No se proporcionaron archivos para unir.".to_string());
    }

    // Load documents in parallel from memory
    let docs: Vec<Result<Document, String>> = files
        .par_iter()
        .map(|bytes| Document::load_mem(bytes).map_err(|e| format!("Error al decodificar PDF: {}", e)))
        .collect();

    let mut documents = Vec::new();
    for doc_res in docs {
        documents.push(doc_res?);
    }

    // Renumber sequentially so they have unique object IDs
    let mut max_id = 1;
    for doc in &mut documents {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;
    }

    // Precalculate pages_id and catalog_id
    let pages_id = (max_id, 0);
    let catalog_id = (max_id + 1, 0);

    let mut merged_doc = Document::with_version("1.5");
    let mut kids = Vec::new();

    // Import all objects and update page Parent references
    for mut doc in documents {
        let doc_pages = doc.get_pages();
        for (_, page_ref) in doc_pages {
            kids.push(Object::Reference(page_ref));
            if let Some(Object::Dictionary(ref mut page_dict)) = doc.objects.get_mut(&page_ref) {
                page_dict.set("Parent", Object::Reference(pages_id));
            }
        }

        for (id, object) in doc.objects {
            let mut keep = true;
            if let Ok(dict) = object.as_dict() {
                if let Ok(type_name) = dict.get(b"Type") {
                    if let Ok(name) = type_name.as_name() {
                        if name == b"Catalog" || name == b"Pages" {
                            keep = false;
                        }
                    }
                }
            }
            if keep {
                merged_doc.objects.insert(id, object);
            }
        }
    }

    let mut catalog = Dictionary::new();
    catalog.set("Type", Object::Name("Catalog".as_bytes().to_vec()));
    catalog.set("Pages", Object::Reference(pages_id));

    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", Object::Name("Pages".as_bytes().to_vec()));
    pages_dict.set("Count", Object::Integer(kids.len() as i64));
    pages_dict.set("Kids", Object::Array(kids));

    merged_doc.objects.insert(pages_id, Object::Dictionary(pages_dict));
    merged_doc.objects.insert(catalog_id, Object::Dictionary(catalog));
    merged_doc.trailer.set("Root", Object::Reference(catalog_id));
    merged_doc.max_id = catalog_id.0;

    let mut output = Vec::new();
    merged_doc.save_to(&mut output).map_err(|e| e.to_string())?;
    Ok(output)
}

pub fn compress_pdf_mem(input_pdf: &[u8], quality: u8) -> Result<Vec<u8>, String> {
    let mut doc = Document::load_mem(input_pdf).map_err(|e| e.to_string())?;
    let image_ids: Vec<ObjectId> = doc.objects.iter().filter_map(|(&id, obj)| {
        let dict = obj.as_stream().ok().map(|s| &s.dict)?;
        (dict.get(b"Subtype").ok()?.as_name().ok()? == b"Image").then_some(id)
    }).collect();
    let mut changed = 0usize;
    for id in image_ids {
        let Some(Object::Stream(stream)) = doc.objects.get_mut(&id) else { continue };
        let width = stream.dict.get(b"Width").ok().and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
        let height = stream.dict.get(b"Height").ok().and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
        let bits = stream.dict.get(b"BitsPerComponent").ok().and_then(|v| v.as_i64().ok()).unwrap_or(8);
        if width == 0 || height == 0 || bits != 8 { continue; }
        let colors = stream.dict.get(b"ColorSpace").ok().and_then(|v| v.as_name().ok()).unwrap_or(b"");
        let raw = match stream.decompressed_content() { Ok(v) => v, Err(_) => continue };
        let image = if colors == b"DeviceRGB" && raw.len() >= (width*height*3) as usize {
            image::RgbImage::from_raw(width, height, raw[..(width*height*3) as usize].to_vec()).map(image::DynamicImage::ImageRgb8)
        } else if colors == b"DeviceGray" && raw.len() >= (width*height) as usize {
            image::GrayImage::from_raw(width, height, raw[..(width*height) as usize].to_vec()).map(image::DynamicImage::ImageLuma8)
        } else { None };
        let Some(mut img) = image else { continue };
        let scale = if quality < 45 { 0.55 } else if quality < 70 { 0.75 } else { 1.0 };
        if scale < 1.0 && width > 1000 {
            img = img.resize((width as f32*scale) as u32, (height as f32*scale) as u32, image::imageops::FilterType::Triangle);
        }
        let mut jpeg = Vec::new();
        if image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, quality.clamp(10, 90)).encode_image(&img).is_err() { continue; }
        if jpeg.len() >= stream.content.len() { continue; }
        stream.content = jpeg;
        stream.dict.set("Width", Object::Integer(img.width() as i64));
        stream.dict.set("Height", Object::Integer(img.height() as i64));
        stream.dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
        stream.dict.set("BitsPerComponent", Object::Integer(8));
        stream.dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
        stream.dict.remove(b"DecodeParms");
        stream.dict.set("Length", Object::Integer(stream.content.len() as i64));
        changed += 1;
    }
    doc.prune_objects();
    doc.compress();
    let mut output = Vec::new();
    doc.save_to(&mut output).map_err(|e| e.to_string())?;
    if changed == 0 { return Err("No se encontraron imágenes PDF compatibles para recomprimir. El archivo puede estar ya optimizado o usar un formato no compatible.".into()); }
    Ok(output)
}

pub fn split_pdf_mem(input_pdf: &[u8], ranges: Vec<(u32, u32)>) -> Result<Vec<Vec<u8>>, String> {
    let source = Document::load_mem(input_pdf).map_err(|e| e.to_string())?;
    let page_count = source.get_pages().len() as u32;
    if ranges.is_empty() { return Err("Debes indicar al menos un rango de páginas.".into()); }
    let mut results = Vec::new();
    for (start, end) in ranges {
        if start == 0 || end < start || end > page_count { return Err(format!("Rango inválido {start}-{end}. El PDF tiene {page_count} páginas.")); }
        let mut doc = source.clone();
        let remove: Vec<u32> = (1..=page_count).filter(|p| *p < start || *p > end).collect();
        doc.delete_pages(&remove);
        doc.prune_objects();
        doc.renumber_objects();
        doc.compress();
        let mut output = Vec::new();
        doc.save_to(&mut output).map_err(|e| e.to_string())?;
        results.push(output);
    }
    Ok(results)
}


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
    
    let image_ids: Vec<ObjectId> = doc.objects.iter()
        .filter(|(_, obj)| {
            if let Ok(dict) = obj.as_dict() {
                if let Ok(subtype) = dict.get(b"Subtype") {
                    if let Ok(name) = subtype.as_name() {
                        return name == b"Image";
                    }
                }
            }
            false
        })
        .map(|(&id, _)| id)
        .collect();

    for id in image_ids {
        if let Some(Object::Stream(ref mut stream)) = doc.objects.get_mut(&id) {
            if let Ok(data) = stream.decompressed_content() {
                if let Ok(img) = image::load_from_memory(&data) {
                    let mut compressed_data = Vec::new();
                    let mut cursor = std::io::Cursor::new(&mut compressed_data);
                    
                    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
                    if encoder.encode_image(&img).is_ok() {
                        stream.set_content(compressed_data);
                        stream.dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
                        stream.dict.set("Length", Object::Integer(stream.content.len() as i64));
                    }
                }
            }
        }
    }

    let mut output = Vec::new();
    doc.save_to(&mut output).map_err(|e| e.to_string())?;
    Ok(output)
}

pub fn split_pdf_mem(input_pdf: &[u8], ranges: Vec<(u32, u32)>) -> Result<Vec<Vec<u8>>, String> {
    let doc = Document::load_mem(input_pdf).map_err(|e| e.to_string())?;
    let mut created_docs = Vec::new();

    for range in ranges.iter() {
        let mut new_doc = Document::with_version("1.5");
        let mut kids = Vec::new();
        let mut max_id = 1;

        let catalog = doc.catalog().map_err(|e| e.to_string())?;
        let pages_ref = catalog.get(b"Pages").map_err(|e| e.to_string())?;
        let pages_dict = doc.get_object(pages_ref.as_reference().map_err(|_| "Invalid pages reference".to_string())?)
            .map_err(|e| e.to_string())?
            .as_dict()
            .map_err(|_| "Pages is not a dict".to_string())?;
        let doc_kids = pages_dict.get(b"Kids").map_err(|e| e.to_string())?.as_array().map_err(|_| "Kids is not an array".to_string())?;

        let start = (range.0.saturating_sub(1)) as usize;
        let end = (range.1) as usize;

        if start >= doc_kids.len() || start >= end {
            continue;
        }

        let target_kids = &doc_kids[start..std::cmp::min(end, doc_kids.len())];
        for kid in target_kids {
            kids.push(kid.clone());
        }

        for (id, object) in doc.objects.clone() {
            new_doc.objects.insert(id, object);
            if id.0 >= max_id {
                max_id = id.0 + 1;
            }
        }

        let catalog_id = (max_id, 0);
        let pages_id = (max_id + 1, 0);

        let mut new_catalog = Dictionary::new();
        new_catalog.set("Type", Object::Name("Catalog".as_bytes().to_vec()));
        new_catalog.set("Pages", Object::Reference(pages_id));

        let mut new_pages_dict = Dictionary::new();
        new_pages_dict.set("Type", Object::Name("Pages".as_bytes().to_vec()));
        new_pages_dict.set("Count", Object::Integer(kids.len() as i64));
        new_pages_dict.set("Kids", Object::Array(kids.clone()));

        new_doc.objects.insert(catalog_id, Object::Dictionary(new_catalog));
        new_doc.objects.insert(pages_id, Object::Dictionary(new_pages_dict));
        new_doc.trailer.set("Root", Object::Reference(catalog_id));
        new_doc.max_id = pages_id.0;

        // Update kids Parent field to point to split pages node
        for kid in &kids {
            if let Ok(kid_ref) = kid.as_reference() {
                if let Some(Object::Dictionary(ref mut kid_dict)) = new_doc.objects.get_mut(&kid_ref) {
                    kid_dict.set("Parent", Object::Reference(pages_id));
                }
            }
        }

        let mut output = Vec::new();
        new_doc.save_to(&mut output).map_err(|e| e.to_string())?;
        created_docs.push(output);
    }

    Ok(created_docs)
}

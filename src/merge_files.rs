use std::{collections::BTreeMap, path::PathBuf, thread, time::SystemTime};

use lopdf::{Bookmark, Document, Object};

pub fn start(file_paths: Vec<PathBuf>, save_path: PathBuf) {
    thread::spawn(move || {
        let sys_time = SystemTime::now();
        let open_documents: Vec<Document> = file_paths
            .iter()
            .map(|file_path| Document::load(file_path).expect("Invalid File Path"))
            .collect();
        merge_documents(open_documents, file_paths, save_path);
        println!("{:?}", sys_time.elapsed());
    });
}

fn merge_documents(documents: Vec<Document>, doc_names: Vec<PathBuf>, save_path: PathBuf) {
    let mut page_vec = vec![];
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version("1.7");
    let mut max_id = 1;
    for (mut doc, doc_name) in documents.into_iter().zip(doc_names) {
        let mut first = true;
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;
        page_vec.extend(doc.get_pages().into_iter().map(|(_, object_id)| {
            if first {
                let bookmark = Bookmark::new(
                    doc_name.file_name().unwrap().to_str().unwrap().to_string(),
                    [0.0, 0.0, 1.0],
                    0,
                    object_id,
                );
                document.add_bookmark(bookmark, None);
                first = false;
            }

            object_id
        }));
        documents_objects.extend(doc.objects);
    }

    let mut catalog_id = None;
    let mut catalog_dict = None;
    let mut pages_id = None;
    let mut pages_dict = None;
    for (object_id, object) in documents_objects {
        // maybe check to not add duplicates
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                if catalog_id.is_none() {
                    catalog_id = Some(object_id);
                    catalog_dict = Some(object.as_dict().unwrap().clone());
                }
            }
            "Pages" => {
                if pages_id.is_none() {
                    pages_id = Some(object_id);
                    pages_dict = Some(object.as_dict().unwrap().clone());
                } else if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some(old_dictionary) = pages_dict {
                        dictionary.extend(&old_dictionary);
                    }
                    pages_dict = Some(dictionary);
                }
            }
            // Modified later
            "Page" => {}
            "Outlines" | "Outline" => {}
            _ => {}
        }
        document.objects.insert(object_id, object.clone());
    }

    // Modify page objects
    for page_id in &page_vec {
        let mut page_dict = document
            .get_object(*page_id)
            .unwrap()
            .as_dict()
            .unwrap()
            .clone();
        page_dict.set("Parent", pages_id.unwrap());
        document
            .objects
            .insert(*page_id, Object::Dictionary(page_dict));
    }

    // Modify pages object
    let pages_id = pages_id.unwrap();
    let mut pages_dict = pages_dict.unwrap();

    // Set new pages count
    pages_dict.set("Count", page_vec.len() as u32);
    // Set new "Kids" list (collected from documents pages) for "Pages"
    pages_dict.set(
        "Kids",
        page_vec
            .into_iter()
            .map(Object::Reference)
            .collect::<Vec<_>>(),
    );
    document
        .objects
        .insert(pages_id, Object::Dictionary(pages_dict));

    let catalog_id = catalog_id.unwrap();
    let mut catalog_dict = catalog_dict.unwrap();
    // Set new "Kids" list (collected from documents pages) for "Pages"
    catalog_dict.set("Pages", pages_id);
    // This will remove all outlines
    catalog_dict.remove(b"Outlines"); // TODO fix outline merging
    document
        .objects
        .insert(catalog_id, Object::Dictionary(catalog_dict));

    // Update the max internal ID as wasn't updated before due to direct objects insertion
    document.max_id = document.objects.len() as u32;

    document.trailer.set("Root", catalog_id);

    // Reorder all new Document objects
    document.renumber_objects();

    //Set any Bookmarks to the First child if they are not set to a page
    document.adjust_zero_pages();

    // Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
    // Create bookmarks based on file names
    if let Some(n) = document.build_outline() {
        if let Object::Dictionary(ref mut dict) = document.get_object_mut(catalog_id).unwrap() {
            dict.set("Outlines", Object::Reference(n));
        }
    }

    document.prune_objects();
    document.compress();
    document.save(&save_path).unwrap();
    println!("Completed")
}

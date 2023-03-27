use std::{
    collections::BTreeMap,
    path::PathBuf,
    thread::{self, JoinHandle},
};

use anyhow::Error;
use lopdf::{Bookmark, Document, Object};
use rayon::prelude::*;

pub fn start(file_paths: Vec<PathBuf>, save_path: PathBuf) -> JoinHandle<()> {
    // TODO: have error messages show up in the UI
    // Create new thead to merge pdf
    thread::spawn(move || {
        // Open all specified documents using rayon thread pools
        // TODO Make it so removing duplicate pgaes is optional
        let open_documents: Vec<Document> = file_paths
            .par_iter()
            .map(|file_path| {
                process_documents(
                    Document::load(file_path).expect("Invalid Input File Path"),
                    0.85,
                )
                .expect("Error Occured While Processing Documents")
            })
            .collect();
        // Run merge_documents code
        merge_documents(open_documents, file_paths, save_path)
            .expect("Error Occured While Merging Documents");
        // save_individual(open_documents, PathBuf::from(""), file_paths);
    })
}

fn process_documents(mut document: Document, threshold: f64) -> Result<Document, Error> {
    // get the size of the pages in each document
    let media_box = get_media_box(&document)?;

    let mut rem_pages = vec![];
    let mut prev_page_text = String::new();

    for (page_num, page_id) in document.get_pages() {
        fix_size(&mut document, page_id, &media_box)?;

        let curr_page_text = document.extract_text(&[page_num])?;
        if strsim::normalized_levenshtein(&prev_page_text, &curr_page_text) >= threshold {
            rem_pages.push(page_num - 1);
        }
        prev_page_text = curr_page_text;
    }
    document.delete_pages(&rem_pages);

    // Reorder all new Document objects
    document.renumber_objects();
    //Set any Bookmarks to the First child if they are not set to a page
    document.adjust_zero_pages();

    document.prune_objects();
    Ok(document)
}

fn get_media_box(document: &Document) -> Result<Object, Error> {
    for (_object_id, object) in document.objects.iter() {
        if let "Pages" = object.type_name().unwrap_or("") {
            let object_dict = object.as_dict()?;
            if object_dict.has("MediaBox".as_bytes()) {
                return Ok(object_dict.get("MediaBox".as_bytes())?.clone());
            }
        }
    }
    // return a default size media_box
    Ok(vec![0.into(), 0.into(), 1024.into(), 768.into()].into())
}

fn fix_size(document: &mut Document, page_id: (u32, u16), media_box: &Object) -> Result<(), Error> {
    // get the dictionary of the current page
    let page_dict = &mut document.get_object_mut(page_id)?.as_dict_mut()?;
    // set the media box of the page if it isn't currently set
    if !page_dict.has("MediaBox".as_bytes()) {
        page_dict.set("MediaBox", media_box.clone());
    }
    Ok(())
}

fn merge_documents(
    documents: Vec<Document>,
    doc_names: Vec<PathBuf>,
    save_path: PathBuf,
) -> Result<(), Error> {
    let mut page_vec = vec![];
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version("1.7");
    let mut max_id = 1;
    for (mut doc, doc_name) in documents.into_iter().zip(doc_names) {
        let mut first = true;
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;
        page_vec.extend(doc.get_pages().into_values().map(|object_id| {
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
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                if catalog_id.is_none() {
                    catalog_id = Some(object_id);
                    catalog_dict = Some(object.as_dict()?.clone());
                }
            }
            "Pages" => {
                if pages_id.is_none() {
                    pages_id = Some(object_id);
                    pages_dict = Some(object.as_dict()?.clone());
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
        let mut page_dict = document.get_object(*page_id)?.as_dict()?.clone();
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
        if let Object::Dictionary(ref mut dict) = document.get_object_mut(catalog_id)? {
            dict.set("Outlines", Object::Reference(n));
        }
    }

    save_document(document, &save_path);
    Ok(())
}

fn save_document(mut document: Document, save_path: &PathBuf) {
    document.prune_objects();
    document.compress();
    document
        .save(save_path)
        .expect("An Error Occured While Saving the Merged Document");
}

fn save_individual(documents: Vec<Document>, save_folder: PathBuf, file_paths: Vec<PathBuf>) {
    for (document, file_path) in documents.into_iter().zip(file_paths) {
        let file_path = save_folder.join(file_path.file_name().unwrap());
        save_document(document, &file_path)
    }
}

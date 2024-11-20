use super::toc::render::{Render, ValidSize};
use crate::config::combine::CombinePDFParam;
use crate::config::utils::Language;
use lopdf::{dictionary, Document, Object, ObjectId};
use std::collections::BTreeMap;

pub fn combine(param: &mut CombinePDFParam) -> anyhow::Result<()> {
    combine_pdf(param)?;
    Ok(())
}

fn combine_pdf(param: &mut CombinePDFParam) -> anyhow::Result<()> {
    param.update_pages()?;
    // create toc
    let mut render = Render::new()?;
    render.set_content(match param.language {
        Language::CN => "目录",
        Language::EN => "Table of Content",
    });
    render.set_size(match param.language {
        Language::CN => &ValidSize::A4,
        Language::EN => &ValidSize::LETTER,
    });
    render.print(&param.files, &param.toc)?;

    let mut docs = vec![];
    if let Some(cover) = &param.cover {
        docs.push(Document::load(&cover)?);
    }
    docs.push(Document::load(&param.toc)?);
    for output in &param.files {
        docs.push(Document::load(&output.filepath)?);
    }

    param.update_pages()?;

    // start combine
    let mut max_id = 1;

    let mut document_pages = BTreeMap::new();
    let mut document_objects = BTreeMap::new();
    let mut merged_doc = Document::with_version("1.7");

    for (_, mut doc) in docs.into_iter().enumerate() {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;
        document_pages.extend(
            doc.get_pages()
                .into_iter()
                .map(|(_, object_id)| (object_id, doc.get_object(object_id).unwrap().to_owned()))
                .collect::<BTreeMap<ObjectId, Object>>(),
        );
        document_objects.extend(doc.objects);
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;
    for (object_id, object) in document_objects.iter() {
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object {
                        id
                    } else {
                        *object_id
                    },
                    object.clone(),
                ));
            }
            "Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref object)) = pages_object {
                        if let Ok(old_dictionary) = object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }
                    pages_object = Some((
                        if let Some((id, _)) = pages_object {
                            id
                        } else {
                            *object_id
                        },
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            "Page" => {}
            "Outlines" => {}
            "Outline" => {}
            _ => {
                merged_doc.objects.insert(*object_id, object.clone());
            }
        }
    }

    if pages_object.is_none() {
        println!("Pages root not found.");
        return Ok(());
    }

    for (object_id, object) in document_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.as_ref().unwrap().0);

            merged_doc
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    if catalog_object.is_none() {
        println!("Catalog root not found.");
        return Ok(());
    }

    let catalog_object = catalog_object.unwrap();
    let pages_object = pages_object.unwrap();

    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", document_pages.len() as u32);

        dictionary.set(
            "Kids",
            document_pages
                .into_iter()
                .map(|(object_id, _)| Object::Reference(object_id))
                .collect::<Vec<_>>(),
        );

        merged_doc
            .objects
            .insert(pages_object.0, Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove("Outlines".as_bytes());

        merged_doc
            .objects
            .insert(catalog_object.0, Object::Dictionary(dictionary));
    }
    merged_doc.trailer.set("Root", catalog_object.0);
    merged_doc.max_id = merged_doc.objects.len() as u32;
    merged_doc.renumber_objects();
    merged_doc.adjust_zero_pages();

    merged_doc.compress();
    let mut title_object_id = vec![];
    for (id, obj) in merged_doc.objects.iter() {
        let dict = obj.as_dict();
        if let Ok(dict) = dict {
            let t = dict.get(b"Title");
            if t.is_ok() {
                if dict.get(b"Parent").is_ok() {
                    title_object_id.push(id);
                }
            }
        }
    }
    rebuild_toc_links(&mut merged_doc, &param)?;
    merged_doc.save(&param.destination)?;
    Ok(())
}

/// rebuild links in toc according combine parameters
fn rebuild_toc_links(doc: &mut Document, param: &CombinePDFParam) -> anyhow::Result<()> {
    let outputs = &param.files;
    let obj_ids = doc
        .objects
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<Vec<ObjectId>>();
    for id in obj_ids {
        let obj = doc.get_object_mut(id)?;
        if let Ok(obj) = obj.as_dict_mut() {
            if obj.type_is(b"Annot") {
                if let Ok(dest) = obj.get(b"Dest")?.as_name_str() {
                    let id = dest.to_string().parse::<usize>()?;
                    let page = outputs[id].page_actual as i64;
                    obj.set(
                        b"Dest",
                        Object::Array(vec![
                            Object::Integer(page),
                            Object::Name(b"XYZ".into()),
                            Object::Null,
                            Object::Null,
                            Object::Null,
                            Object::Dictionary(dictionary! {
                                "XYZ" => vec![Object::Null, Object::Null, Object::Null]
                            }),
                        ]),
                    );
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{combiner::pdf::outline::add_outline, config::combine::PDFFile, top::read_top};

    use super::*;
    #[test]
    fn combine_pdf_test() -> anyhow::Result<()> {
        let bin = Path::new(r"D:\projects\py\outlines\dist\outline.exe");
        let mut param: CombinePDFParam = param();
        // param.update_pages()?;
        combine(&mut param)?;
        add_outline(&param.workspace, &param.to_outline_param(), bin)?;
        Ok(())
    }

    fn param() -> CombinePDFParam {
        let source_dir = Path::new(
            r"D:\Users\yuqi01.chen\.temp\app\mobiuskit\fusion\workspace\AS1F2uo6LW\converted",
        );
        let destination =
            Path::new(r"D:\Studies\ak101\203\stats\dryrun\product\output\combined\final.pdf");
        let files = read_top(Path::new(
            r"D:\Studies\ak101\203\stats\dryrun\utility\top-ak112-303-CSR.xlsx",
        ))
        .unwrap()
        .into_iter()
        .map(|top| PDFFile {
            title: top.title,
            filepath: source_dir.join(top.filename.replace(".rtf", ".pdf")),
            ..Default::default()
        })
        .collect::<Vec<PDFFile>>();

        let files = files
            .iter()
            .filter(|file| file.filepath.exists())
            .map(|f| f.clone())
            .collect();

        let workspace = Path::new(r"D:\projects\rusty\toc\.data\combine_workspace");
        CombinePDFParam {
            workspace: workspace.into(),
            language: Language::CN,
            cover: Some(
                Path::new(r"D:/Studies/ak112/303/stats/CSR/product/output/combined/cover.pdf")
                    .into(),
            ),
            // cover: None,
            toc: workspace.join("toc.pdf"),
            toc_start_pages: 0,
            files,
            destination: destination.into(),
        }
    }
}

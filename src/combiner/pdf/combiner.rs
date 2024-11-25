use super::{
    location::{Location, LocationManager},
    toc::render::{Render, ValidSize},
};
use crate::config::combine::CombinePDFParam;
use crate::config::utils::Language;
use anyhow::anyhow;
use lopdf::{dictionary, Document, Object, ObjectId};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
};

const COVER_TITLE_EN: &str = "Cover";
const COVER_TITLE_CN: &str = "封面";
const TOC_TITLE_EN: &str = "Table of Content";
const TOC_TITLE_CN: &str = "目录";

#[derive(Debug, Serialize)]
pub struct CombineConfig {
    destination: PathBuf,
    language: Language,
    total: usize,
    files: Vec<Location>,
}

pub struct PDFCombiner {
    combine_bin: PathBuf,
    param: CombinePDFParam,
    location: LocationManager,
    total_pages: usize,
}

impl PDFCombiner {
    pub fn new(param: &CombinePDFParam, combine_bin: &Path) -> anyhow::Result<Self> {
        let mut total_pages = 0;
        let mut location: LocationManager = LocationManager::new();
        for file in &param.files {
            let doc = Document::load(&file.filepath)?;
            let pages = doc.get_pages().len();
            location.push(Some(file.id), &file.title, pages, &file.filepath);
            total_pages += pages;
        }
        Ok(PDFCombiner {
            param: param.clone(),
            location,
            combine_bin: combine_bin.into(),
            total_pages,
        })
    }

    pub fn combine(&mut self) -> anyhow::Result<()> {
        self.create_toc()?;
        self.combine_pdf()?;
        self.rebuild_toc_links()?;
        Ok(())
    }

    fn create_toc(&mut self) -> anyhow::Result<()> {
        let mut render = Render::new()?;
        render.set_content(match self.param.language {
            Language::CN => TOC_TITLE_CN,
            Language::EN => TOC_TITLE_EN,
        });
        render.set_size(match self.param.language {
            Language::CN => &ValidSize::A4,
            Language::EN => &ValidSize::LETTER,
        });
        render.set_toc_headers(&self.param.toc_headers);
        render.print(&self.location.data(), &self.param.toc)?;
        Ok(())
    }

    fn combine_pdf(&mut self) -> anyhow::Result<()> {
        // toc
        if self.param.toc.exists() {
            let toc = self.param.toc.as_path();
            let title: String = match self.param.language {
                Language::CN => TOC_TITLE_CN.into(),
                Language::EN => TOC_TITLE_EN.into(),
            };
            let page = Document::load(toc)?.get_pages().len();
            self.location.insert_head(None, &title, page, toc);
        }
        // cover
        if let Some(cover) = &self.param.cover {
            if cover.exists() {
                let title: String = match self.param.language {
                    Language::CN => COVER_TITLE_CN.into(),
                    Language::EN => COVER_TITLE_EN.into(),
                };
                let page = Document::load(&cover)?.get_pages().len();
                self.location.insert_head(None, &title, page, &cover);
            }
        }

        let config = self.param.workspace.join("config.json");
        fs::write(
            &config,
            serde_json::to_vec(&CombineConfig {
                destination: self.param.destination.clone(),
                language: self.param.language.clone(),
                files: self.location.data(),
                total: self.total_pages,
            })?,
        )?;
        let mut cmd = Command::new("cmd");
        cmd.creation_flags(0x08000000);
        // call binary to combine pdf and add outline
        let result = cmd.arg("/C").arg(&self.combine_bin).arg(&config).output()?;
        if !result.status.success() {
            let err_message = result.stderr;
            return Err(anyhow!(String::from_utf8(err_message)?));
        }
        Ok(())
    }
    /// rebuild links in toc according combine parameters
    fn rebuild_toc_links(&self) -> anyhow::Result<()> {
        let mut doc = Document::load(&self.param.destination)?;
        let outputs = self
            .location
            .data()
            .into_iter()
            .filter(|l| l.id.is_some())
            .map(|l| (l.id.unwrap(), l.page))
            .collect::<HashMap<usize, usize>>();
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
                        if let Some(page) = outputs.get(&id) {
                            obj.set(
                                b"Dest",
                                Object::Array(vec![
                                    Object::Integer(*page as i64),
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
        }
        doc.save(&self.param.destination)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{config::combine::PDFFile, top::read_top};

    use super::*;
    #[test]
    fn combine_pdf_test() -> anyhow::Result<()> {
        let bin = Path::new(r"D:\projects\py\combiner\dist\combiner.exe");
        let param: CombinePDFParam = param();
        let mut combiner = PDFCombiner::new(&param, bin)?;
        combiner.combine()?;
        Ok(())
    }

    fn param() -> CombinePDFParam {
        let source_dir = Path::new(
            r"D:\Users\yuqi01.chen\.temp\app\mobiuskit\fusion\workspace\MU0LrjDeuu\converted",
        );
        let destination =
            Path::new(r"D:\Studies\ak101\203\stats\dryrun\product\output\combined\final.pdf");
        let files = read_top(Path::new(
            r"D:\Studies\ak102\202\stats\idmc\utility\top-ak112-101-20240620.xlsx",
        ))
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(id, top)| PDFFile {
            id,
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
            language: Language::EN,
            cover: None,
            toc: workspace.join("toc.pdf"),
            toc_start_pages: 0,
            files,
            destination: destination.into(),
            toc_headers: ("".into(), "".into(), "".into(), "".into()),
        }
    }
}

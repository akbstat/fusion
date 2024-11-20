use std::{
    fs,
    path::{Path, PathBuf},
};

use lopdf::Document;
use serde::Serialize;

use super::utils::Language;

#[derive(Debug, Serialize)]
pub struct OutlineParam {
    pub(crate) target: PathBuf,
    pub(crate) locations: Vec<Location>,
}

#[derive(Debug, Serialize)]
pub struct Location {
    pub(crate) title: String,
    pub(crate) page: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PDFFile {
    pub id: usize,
    pub title: String,
    pub filepath: PathBuf,
    pub page_display: usize,
    pub page_actual: usize,
}

#[derive(Debug, Clone)]
pub struct CombinePDFParam {
    pub(crate) workspace: PathBuf,
    pub(crate) language: Language,
    pub(crate) cover: Option<PathBuf>,
    pub(crate) toc: PathBuf,
    pub(crate) toc_start_pages: usize,
    pub(crate) files: Vec<PDFFile>,
    pub(crate) destination: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RTFCombineParam {
    pub(crate) destination: PathBuf,
    pub(crate) files: Vec<PathBuf>,
}

impl CombinePDFParam {
    pub fn new(
        workspace: &Path,
        language: &Language,
        cover: &Option<PathBuf>,
        toc: &Path,
        files: &[PDFFile],
        destination: &Path,
    ) -> anyhow::Result<CombinePDFParam> {
        if !workspace.exists() {
            fs::create_dir_all(&workspace)?;
        }
        Ok(CombinePDFParam {
            workspace: workspace.into(),
            language: language.clone(),
            cover: cover.clone(),
            toc: toc.into(),
            toc_start_pages: 0,
            files: files.to_vec(),
            destination: destination.into(),
        })
    }
    pub fn update_pages(&mut self) -> anyhow::Result<()> {
        if !self.workspace.exists() {
            fs::create_dir_all(&self.workspace)?;
        }

        let cover_pages = match &self.cover {
            Some(p) => Document::load(&p)?.get_pages().len(),
            None => 0,
        };
        let toc_pages = if self.toc.exists() {
            Document::load(&self.toc)?.get_pages().len()
        } else {
            0
        };
        self.toc_start_pages = cover_pages;
        let mut page_display = 1;
        let mut page_actual = cover_pages + toc_pages;
        for (index, file) in self.files.iter_mut().enumerate() {
            let page = Document::load(&file.filepath)?.get_pages().len();
            file.id = index;
            file.page_display = page_display;
            file.page_actual = page_actual;
            page_display += page;
            page_actual += page;
        }
        Ok(())
    }
    pub fn to_outline_param(&self) -> OutlineParam {
        let mut locations = Vec::with_capacity(self.files.len());
        if let Some(_) = self.cover {
            locations.push(Location {
                title: cover_title(&self.language),
                page: 0,
            });
        }
        locations.push(Location {
            title: toc_title(&self.language),
            page: self.toc_start_pages,
        });
        self.files.iter().for_each(|f| {
            locations.push(Location {
                title: f.title.clone(),
                page: f.page_actual,
            })
        });
        OutlineParam {
            target: self.destination.clone(),
            locations,
        }
    }
}

fn toc_title(lang: &Language) -> String {
    match lang {
        Language::CN => "目录".into(),
        Language::EN => "Table of Content".into(),
    }
}

fn cover_title(lang: &Language) -> String {
    match lang {
        Language::CN => "封面".into(),
        Language::EN => "Cover".into(),
    }
}

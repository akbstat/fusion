use super::template;
use crate::combiner::pdf::location::Location;
use anyhow::Ok;
use headless_chrome::{types::PrintToPdfOptions, Browser};
use serde::Serialize;
use std::{fs, path::Path};
use tera::{Context, Tera};

const TOC_TEMPLATE: &str = "toc";

#[derive(Debug, Serialize)]
struct RenderData {
    pub items: Vec<Location>,
    pub content: String,
    pub size: ValidSize,
    pub toc_headers: (String, String, String, String),
}

#[derive(Debug, Default, Clone, Serialize)]
pub enum ValidSize {
    #[default]
    A4,
    LETTER,
}

#[derive(Debug, Default)]
pub struct Render {
    template: Tera,
    content: String,
    company: String,
    study: String,
    purpose: String,
    toc_headers: (String, String, String, String),
    pub size: ValidSize,
}

impl Render {
    pub fn new() -> anyhow::Result<Render> {
        let mut tmpl = Tera::default();
        tmpl.add_raw_template(TOC_TEMPLATE, template::TEMPLATE)?;
        tmpl.autoescape_on(vec![]);
        Ok(Render {
            template: tmpl,
            ..Default::default()
        })
    }
    pub fn size(&self) -> ValidSize {
        self.size.clone()
    }

    pub fn set_content(&mut self, content: &str) -> &mut Self {
        self.content = content.to_owned();
        self
    }

    pub fn set_study(&mut self, study: &str) -> &mut Self {
        self.study = study.to_owned();
        self
    }

    pub fn set_company(&mut self, company: &str) -> &mut Self {
        self.company = company.to_owned();
        self
    }

    pub fn set_purpose(&mut self, purpose: &str) -> &mut Self {
        self.purpose = purpose.to_owned();
        self
    }

    pub fn set_size(&mut self, size: &ValidSize) -> &mut Self {
        self.size = size.clone();
        self
    }

    pub fn set_toc_headers(&mut self, toc_headers: &(String, String, String, String)) -> &mut Self {
        self.toc_headers = toc_headers.clone();
        self
    }

    pub fn print(&self, items: &[Location], dest: &Path) -> anyhow::Result<()> {
        let data = RenderData {
            content: self.content.clone(),
            items: items.to_vec(),
            size: self.size.clone(),
            toc_headers: self.toc_headers.clone(),
        };
        let bytes = self
            .template
            .render(TOC_TEMPLATE, &Context::from_serialize(&data)?)?
            .as_bytes()
            .to_vec();
        let html_dest = dest.parent().unwrap().join(format!(
            "{}.html",
            dest.file_stem().unwrap().to_str().unwrap()
        ));
        fs::write(&html_dest, bytes)?;
        html_to_pdf(&html_dest, &dest)?;
        Ok(())
    }
}

pub fn html_to_pdf(source: &Path, destination: &Path) -> anyhow::Result<()> {
    let url = format!("file:///{}", source.to_string_lossy().to_string());
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    let pdf_options: Option<PrintToPdfOptions> = Some(PrintToPdfOptions {
        landscape: None,
        display_header_footer: None,
        print_background: None,
        scale: None,
        paper_width: None,
        paper_height: None,
        margin_top: None,
        margin_bottom: None,
        margin_left: None,
        margin_right: None,
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: Some(true),
        transfer_mode: None,
    });
    let pdf = tab
        .navigate_to(&url)?
        .wait_until_navigated()?
        .print_to_pdf(pdf_options)?;
    fs::write(destination, pdf)?;
    Ok(())
}

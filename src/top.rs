use calamine::{open_workbook, DataType, Reader, Xlsx};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use serde::Serialize;
use std::{char, path::Path};

const TARGET_ROWS_START: usize = 1;
const OUTPUT_NAME_COLUMN: usize = 4;
const OUTPUT_NUMBER_COLUMN: usize = 5;
const TITLE_COLUMN: usize = 6;
const TYPE_COLUMN: usize = 2;

#[derive(Debug, Serialize)]
pub struct Top {
    pub filename: String,
    pub title: String,
}

#[derive(Debug)]
enum ProjectLanguage {
    EN,
    CN,
}

fn top_language(title: &str) -> ProjectLanguage {
    if title.chars().any(|c| c > '\u{7F}') {
        ProjectLanguage::CN
    } else {
        ProjectLanguage::EN
    }
}

pub fn read_top(file: &Path) -> anyhow::Result<Vec<Top>> {
    let mut result = vec![];
    let mut workbook: Xlsx<_> = open_workbook(file)?;
    if let Some(range) = workbook.worksheet_range_at(0) {
        let range = range?;
        for (n, row) in range.rows().into_iter().enumerate() {
            // skipping untarget rows
            if n < TARGET_ROWS_START {
                continue;
            }
            let name = if let Some(data) = row.get(OUTPUT_NAME_COLUMN) {
                data.as_string().map(|s| s.to_lowercase())
            } else {
                None
            };
            let number = if let Some(data) = row.get(OUTPUT_NUMBER_COLUMN) {
                data.as_string()
            } else {
                None
            };
            let title = if let Some(data) = row.get(TITLE_COLUMN) {
                data.as_string().map(|s| handle_unicode_declaration(&s))
            } else {
                None
            };
            let output_type = if let Some(data) = row.get(TYPE_COLUMN) {
                data.as_string()
            } else {
                None
            };
            if name.is_some() && number.is_some() && title.is_some() && output_type.is_some() {
                let title = title.unwrap();
                result.push(Top {
                    filename: format!("{}.rtf", &name.unwrap()),
                    title: format!(
                        "{} {}: {}",
                        title_prefix(&output_type.unwrap(), &top_language(&title)),
                        number.unwrap(),
                        title,
                    )
                    .trim()
                    .to_string(),
                });
            }
        }
    }
    Ok(result)
}

fn title_prefix(symbol: &str, language: &ProjectLanguage) -> String {
    match symbol {
        "T" => match language {
            ProjectLanguage::EN => "Table".into(),
            ProjectLanguage::CN => "表".into(),
        },
        "F" => match language {
            ProjectLanguage::EN => "Figure".into(),
            ProjectLanguage::CN => "图".into(),
        },
        "L" => match language {
            ProjectLanguage::EN => "Listing".into(),
            ProjectLanguage::CN => "列表".into(),
        },
        _ => "".into(),
    }
}

/// handle unicode decalration in title, such as "PT Rate ≥ 5~{unicode 0025}", "~{unicode 0025}" stands for "%",
///
/// unicode declaration using hex code
fn handle_unicode_declaration(source: &str) -> String {
    static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"~\{unicode\s(\d{4})}").unwrap());
    let replaced = PATTERN.replace_all(source, |cap: &Captures| {
        if let Some(code) = cap.get(1) {
            unicode_convert(code.as_str())
        } else {
            "".to_string()
        }
    });
    replaced.into()
}

/// convert unicode in rtf to a char, for example: "0025;" => '%'
///
/// if invalid unicode then return empty string
fn unicode_convert(source: &str) -> String {
    if let Ok(n) = u32::from_str_radix(source, 16) {
        if let Some(c) = char::from_u32(n) {
            return c.into();
        }
    }
    "".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_top_test() -> anyhow::Result<()> {
        let filepath = Path::new(r"D:\Studies\ak112\303\stats\CSR\utility\top-ak112-303-CSR.xlsx");
        let top = read_top(filepath)?;
        assert!(!top.is_empty());
        top.iter().for_each(|e| {
            println!("{:?}", e);
        });
        Ok(())
    }
    #[test]
    fn handle_unicode_declaration_test() {
        let source = "表 3.1.2.2.3: 整体治疗阶段的TEAE按SOC、PT总结（任意一组别PT发生率 ≥ 1~{unicode 0025}）（安全性分析集）";
        let dest = handle_unicode_declaration(source);
        assert_eq!("表 3.1.2.2.3: 整体治疗阶段的TEAE按SOC、PT总结（任意一组别PT发生率 ≥ 1%）（安全性分析集）", dest)
    }

    #[test]
    fn unicode_convert_test() {
        let source = "0025";
        assert_eq!(unicode_convert(source), "%")
    }
}

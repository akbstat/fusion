use crate::config::combine::RTFCombineParam;
use std::{
    fs::{read, remove_file, OpenOptions},
    io::Write,
    ops::Sub,
};

const PAGE_PAR: &'static [u8] = br"{\page\par}";
const WINDOW_CTRL: &'static [u8] = br"\widowctrl";

/// combine muliple rtfs into one rtf
pub fn combine(param: &RTFCombineParam) -> anyhow::Result<()> {
    // create destination file
    if param.destination.exists() {
        remove_file(&param.destination)?;
    }
    let mut destination = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&param.destination)?;

    // record if header is writen
    let mut header_writen = false;
    for (index, p) in param.files.iter().enumerate() {
        let data = read(p)?;
        if let Some((start, end)) = extract_file_content(&data) {
            if !header_writen {
                destination.write(data.get(0..start).unwrap())?;
                header_writen = true;
            }
            destination.write(data.get(start..end).unwrap())?;
            if index.lt(&param.files.len().sub(&1)) {
                destination.write(&PAGE_PAR)?;
            }
        }
    }
    destination.write(br"}")?;
    Ok(())
}

/// extract content of rtf, start from symbol "\widowctrl", end with the next to last charater
/// ```rust
/// #[test]
/// fn extract_test() {
///     let data = br"\widowctrl\test}}";
///     let result = extract_file_content(&data);
///     assert_eq(result, Some((0, data.len() - 3)));
/// }
/// ```
fn extract_file_content(data: &[u8]) -> Option<(usize, usize)> {
    let mut last_curly_brace = data.len() - 1;
    // seek the last curly brace
    while last_curly_brace.gt(&0) {
        if let Some(char) = data.get(last_curly_brace) {
            if char.eq(&b'}') {
                break;
            }
            last_curly_brace -= 1;
        }
    }
    match pattern_position(&WINDOW_CTRL, &data, 0) {
        Some((start, _)) => Some((start, last_curly_brace)),
        None => None,
    }
}

/// find out the position of specify pattern in occurs for the first time
///
/// @pattern: the pattern you want to find out
///
/// @source: target that you want to find out group
///
/// @pointer: the start position in the source
///
/// ```rust
/// #[test]
/// fn test_pattern_position() {
///     let content = br"{\fonttbl{\f1\froman\fprq2\fcharset0 SimSun;}".to_vec();
///     let pattern = br"\fonttbl".to_vec();
///     let result = pattern_position(&pattern, &content, 0).unwrap();
///     let result = String::from_utf8(content.get(result.0..result.1).unwrap().into()).unwrap();
///     assert_eq!(pattern, result.as_bytes());
///     let pattern = br"\test".to_vec();
///     let result = pattern_position(&pattern, &content, 0);
///     assert_eq!(result, None);
/// }
/// ```
pub fn pattern_position(pattern: &[u8], source: &[u8], pointer: usize) -> Option<(usize, usize)> {
    let mut pointer = pointer;
    let pattern_size = pattern.len();
    if pointer > source.len() {
        return None;
    }
    while pointer < source.len() {
        if pointer < pattern_size + 1 {
            pointer += 1;
            continue;
        }
        let start = pointer - pattern_size - 1;
        let end = pointer - 1;
        match source.get(start..end) {
            Some(target) => {
                if pattern.eq(target) {
                    return Some((start, end));
                }
            }
            None => {
                continue;
            }
        };
        pointer += 1;
    }
    None
}

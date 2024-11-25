use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub id: Option<usize>,
    pub title: String,
    pub page: usize,
    pub path: PathBuf,
}

pub struct LocationManager {
    data: Vec<Location>,
    total_pages: usize,
}

impl LocationManager {
    pub fn new() -> Self {
        LocationManager {
            data: vec![],
            total_pages: 0,
        }
    }
    pub fn push(
        &mut self,
        id: Option<usize>,
        title: &str,
        total_pages: usize,
        path: &Path,
    ) -> &mut Self {
        let page = self.total_pages;
        self.data.push(Location {
            id,
            title: title.into(),
            page,
            path: path.into(),
        });
        self.total_pages += total_pages;
        self
    }
    pub fn insert_head(
        &mut self,
        id: Option<usize>,
        title: &str,
        total_pages: usize,
        path: &Path,
    ) -> &mut Self {
        self.data.iter_mut().for_each(|location| {
            location.page += total_pages;
        });
        self.data.insert(
            0,
            Location {
                id,
                title: title.into(),
                page: 0,
                path: path.into(),
            },
        );
        self.total_pages += total_pages;
        self
    }
    pub fn data(&self) -> Vec<Location> {
        self.data.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn location_test() {
        let mut manager = LocationManager::new();
        manager.push(Some(1), "output 1", 2, Path::new(""));
        manager.push(Some(2), "output 2", 3, Path::new(""));
        manager.push(Some(3), "output 3", 1, Path::new(""));
        manager.insert_head(None, "toc", 1, Path::new(""));
        assert_eq!(manager.data().len(), 4);
        assert_eq!(manager.data().first().unwrap().title, "toc");
        assert_eq!(manager.data().last().unwrap().page, 5);
        assert_eq!(manager.total_pages, 7);
    }
}

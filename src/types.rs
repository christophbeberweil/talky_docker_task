use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum TalkyError {
    IoError(std::io::Error),
    TextError(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Directory {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct File {
    pub name: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RenderData {
    pub current_path: String,
    pub directories: Vec<Directory>,
    pub files: Vec<File>,
    pub breadcrumbs: Vec<Breadcrumb>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Breadcrumb {
    pub path: String,
    pub display: String,
}

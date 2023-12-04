use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum TalkyError {
    IoError(std::io::Error),
    TextError(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Directory {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub name: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RenderData {
    pub directories: Vec<Directory>,
    pub files: Vec<File>,
}

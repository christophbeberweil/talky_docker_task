use std::{fs, str::FromStr};

use crate::{
    config::Config,
    types::{Directory, File, RenderData, TalkyError},
};

use axum::{
    body::Bytes,
    extract::State,
    http::{uri::PathAndQuery, Uri},
    response::IntoResponse,
};

/// the single view we use to render the html from a request path
pub async fn render_folder_contents(uri: Uri, State(config): State<Config>) -> impl IntoResponse {
    let request_path = uri
        .path_and_query()
        .unwrap_or(
            &PathAndQuery::from_str("/").expect("PathAndQuery from static str should always work"),
        )
        .path()
        .to_owned();

    let request_path = format!("{}{}", config.base_dir, request_path);

    // necessary, because blanks in the path at this point are url encoded to %20.
    let request_path = request_path.replace("%20", " ");

    // are we looking at a file or a directory?
    // directory -> list the elements
    // file -> todo: Initiate Download somehow

    match fs::read(request_path.clone()) {
        Ok(file_data) => {
            // serve the data
            serve_file(file_data).into_response()
        }
        Err(_) => {
            // we ignore the error, because then it is not a file and we assume a directory
            match list_elements_in_directory(request_path) {
                Ok(render_data) => {
                    // todo: do not re-initialize the upon::Engine in every request, but re-use it instead

                    let default_template = fs::read_to_string("src/index.html")
                        .expect("Should have been able to read the default index.html");

                    let mut engine = upon::Engine::new();
                    engine
                        .add_template("default_template", default_template)
                        .expect("The base template should render");

                    // if there is an _index_talky.html in the directory, try to render it instead

                    let rendered_template = engine
                        .template("default_template")
                        .render(render_data)
                        .to_string();

                    match rendered_template {
                        Ok(rendered_text) => axum::response::Html(rendered_text).into_response(),
                        Err(e) => axum::response::Html(format!("{e:?}")).into_response(),
                    }
                }
                Err(e) => axum::response::Html(format!("{e:?}")).into_response(),
            }
        }
    }
}

// this view function serves an individual file for download. The file coul be read via std::fs::read
fn serve_file(file_data: Vec<u8>) -> impl IntoResponse {
    Bytes::from(file_data).into_response()
}

/// given a path to a directory as string, this function will calculate the RenderData for the directory.
fn list_elements_in_directory(dirpath: String) -> Result<RenderData, TalkyError> {
    println!("{dirpath}");
    match fs::read_dir(dirpath) {
        Ok(directory_content) => {
            let mut files: Vec<File> = vec![];
            let mut directories: Vec<Directory> = vec![];

            for entry in directory_content.into_iter() {
                match entry {
                    Ok(dir_entry) => {
                        let file_type =
                            dir_entry.file_type().expect("file type should be readable");

                        if file_type.is_dir() {
                            let directory_name =
                                dir_entry.file_name().to_str().unwrap_or("").to_owned();
                            directories.push(Directory {
                                name: directory_name.clone(),
                            })
                        }
                        if file_type.is_file() {
                            let file_name = dir_entry.file_name().to_str().unwrap_or("").to_owned();
                            // hide hidden files. But maybe there is a better way?
                            if !file_name.starts_with('.') {
                                files.push(File {
                                    name: file_name.clone(),
                                })
                            }
                        }

                        // there is also is_symlink, but we will ignore symlinks, as they can and maybe should not be downloaded
                    }
                    Err(_) => {
                        // ignore file errors for now
                    }
                }
            }

            Ok(RenderData { directories, files })
        }
        Err(e) => Err(TalkyError::IoError(e)),
    }
}

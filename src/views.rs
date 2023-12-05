use std::{fs, str::FromStr};

use crate::{
    config::Config,
    types::{Directory, File, RenderData, TalkyError},
    util::get_custom_template,
};

use axum::{
    body::Bytes,
    extract::State,
    http::{uri::PathAndQuery, Uri},
    response::IntoResponse,
};

/// the single view we use to render the html from a request path
pub async fn render_folder_contents(uri: Uri, State(config): State<Config>) -> impl IntoResponse {
    tracing::event!(tracing::Level::INFO, "render_folder_contents {}", &uri);
    let request_path = uri
        .path_and_query()
        .unwrap_or(
            &PathAndQuery::from_str("/").expect("PathAndQuery from static str should always work"),
        )
        .path()
        .to_owned();

    // necessary, because blanks in the path at this point are url encoded to %20.
    let mut request_path = request_path.replace("%20", " ");

    // remove first / from request path for proper path joining via easy_paths
    if request_path.starts_with('/') {
        request_path = request_path[1..request_path.len()].to_owned();
    }

    let Some(fullpath) =
        easy_paths::get_path_joined(&[&config.base_dir.as_str(), &request_path.as_str()])
    else {
        return axum::response::Html("could not join paths".to_string()).into_response();
    };

    tracing::event!(
        tracing::Level::DEBUG,
        "render_folder_contents fullpath: {} + {} = {} ",
        &config.base_dir,
        &request_path,
        &fullpath
    );
    // are we looking at a file or a directory?
    // directory -> list the elements
    // file -> todo: Initiate Download somehow

    if easy_paths::is_dir(&fullpath) {
        // we ignore the error, because then it is not a file and we assume a directory

        match get_render_data_from_dir(&config.base_dir, &request_path) {
            Ok(render_data) => {
                // todo: do not re-initialize the upon::Engine in every request, but re-use it instead

                let custom_template = get_custom_template(config.base_dir, request_path.clone());

                let template = custom_template.unwrap_or(config.default_template);

                let engine = upon::Engine::new();
                //engine  .add_template("template", template) ;
                match engine.compile(template) {
                    Ok(rendered_template) => {
                        match rendered_template.render(&engine, render_data).to_string() {
                            Ok(rendered_text) => {
                                axum::response::Html(rendered_text).into_response()
                            }
                            Err(e) => {
                                tracing::event!(
                                    tracing::Level::ERROR,
                                    "Error when rendering template at {}: {:?}",
                                    &request_path,
                                    e
                                );
                                axum::response::Html(format!("{e:?}")).into_response()
                            }
                        }
                    }

                    Err(e) => {
                        tracing::event!(
                            tracing::Level::ERROR,
                            "Error when compiling template at {}: {:?}",
                            &request_path,
                            e
                        );
                        axum::response::Html(format!("{e:?}")).into_response()
                    }
                }

                // if there is an _index_talky.html in the directory, try to render it instead
            }
            Err(e) => {
                tracing::event!(
                    tracing::Level::ERROR,
                    "List elements in directory did not work {}: {:?}",
                    &request_path,
                    e
                );
                axum::response::Html(format!("{e:?}")).into_response()
            }
        }
    } else if easy_paths::is_file(&fullpath) {
        match fs::read(&fullpath) {
            Ok(file_data) => {
                // serve the data
                serve_file(file_data).into_response()
            }
            Err(e) => {
                tracing::event!(
                    tracing::Level::ERROR,
                    "could not read from file: {}",
                    &fullpath,
                );
                axum::response::Html(format!("{e:?}")).into_response()
            }
        }
    } else {
        let message = format!("'{fullpath}' is not a file not a directory 🤷‍♂️");
        tracing::event!(tracing::Level::ERROR, "{}", &message,);
        axum::response::Html(message).into_response()
    }
}

// this view function serves an individual file for download. The file coul be read via std::fs::read
fn serve_file(file_data: Vec<u8>) -> impl IntoResponse {
    Bytes::from(file_data).into_response()
}

/// given a path to a directory as string, this function will calculate the RenderData for the directory.
fn get_render_data_from_dir(
    base_dir: &String,
    request_path: &String,
) -> Result<RenderData, TalkyError> {
    let Some(dirpath) = easy_paths::get_path_joined(&[base_dir, request_path]) else {
        return Err(TalkyError::TextError(
            "Could not join paths with easy_paths".to_owned(),
        ));
    };

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

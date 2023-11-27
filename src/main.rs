use std::net::SocketAddr;
use std::{fs, str::FromStr};

use crate::config::Config;
use axum::body::{Bytes, Full, StreamBody};
use axum::extract::State;
use axum::http::header::CONTENT_DISPOSITION;
use axum::http::{self, HeaderMap, HeaderValue, Response, StatusCode};
use axum::{
    http::{uri::PathAndQuery, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
mod config;

#[derive(Debug)]
enum TalkyError {
    IoError(std::io::Error),
}

#[derive(Serialize, Deserialize, Debug)]
struct Directory {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct File {
    name: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct RenderData {
    directories: Vec<Directory>,
    files: Vec<File>,
}

/// the single view we use to render the html from a request path
async fn render_folder_contents(uri: Uri, State(config): State<Config>) -> impl IntoResponse {
    let request_path = uri
        .path_and_query()
        .unwrap_or(&PathAndQuery::from_str("/").unwrap())
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
            //axum::response::Html(format!("{file_data:?}")).into_response()

            let filename = request_path
                .clone()
                .split('/')
                .last()
                .unwrap_or("myfile.txt")
                .to_owned();
            serve_file(file_data, filename).into_response()
        }
        Err(e) => {
            // return the error
            println!("{e:?}");
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

// this will eventually serve a single file to download
fn serve_file(file_data: Vec<u8>, filename: String) -> impl IntoResponse {
    let mut header_map = HeaderMap::new();

    header_map.insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"Cargo.toml\""),
    );

    Bytes::from(file_data).into_response()

    /*
    let mut builder = Response::builder()
        .header(
            "CONTENT_DISPOSITION",
            format!("attachment; filename=\"{}\"", filename).as_str(),
        )
        .status(StatusCode::OK);

    builder.body("file_data".to_owned()).unwrap()

        */

    /*
       let (mut parts, body) = response.into_parts();
       parts.status = StatusCode::OK;
       parts.headers = header_map;

       let response = Response::from_parts(parts, body);

       let mut a = HeaderMap::new();

       a.append(
           CONTENT_DISPOSITION,
           HeaderValue::from_static("attachment; filename=\"Cargo.toml\""),
       );

       response
    */
    //axum::response::Html(format!("{file_data:?}"))
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

#[tokio::main]
async fn main() -> Result<(), TalkyError> {
    let config = Config::init();

    let router = Router::new()
        .route("/", get(render_folder_contents))
        .fallback(get(render_folder_contents))
        .with_state(config);
    let port: u16 = 3000;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 listening on {}", &addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
    Ok(())
}

use http_types::mime::{Mime, BYTE_STREAM};
use pavex::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use pavex::request::path::PathParams;
use pavex::response::body::raw::Full;
use pavex::response::Response;
use std::fs;
use std::path::Path;

#[PathParams]
pub struct SubPath<'a> {
    pub path: &'a str,
}

pub fn serve_files(subpath: &PathParams<SubPath>) -> Response {
    let prefix = "assets";

    // TODO: Here's where we would modify it for the incoming path. Check how Leptos does it
    let basepath = Path::new(&format!("./{}", prefix)).to_path_buf();

    let mut path = match basepath.join(subpath.0.path).canonicalize() {
        Ok(p) => p,
        Err(_) => return Response::not_found(),
    };

    if path.is_dir() {
        path.push("index.html");
    }

    match path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Response::not_found(),
        Err(_) => return Response::internal_server_error(),
    }

    let mime = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(Mime::from_extension)
        .unwrap_or(BYTE_STREAM)
        .to_string();

    let hv = pavex::http::HeaderValue::from_str(&mime).expect("valid mime type");

    match fs::read(path) {
        Ok(file) => Response::ok()
            .append_header(CONTENT_TYPE, hv)
            .append_header(CONTENT_LENGTH, file.len().into())
            .set_raw_body(Full::new(file.into())),
        Err(_) => Response::internal_server_error(),
    }
}

pub fn index() -> Response {
    serve_files(&PathParams(SubPath { path: "" }))
}

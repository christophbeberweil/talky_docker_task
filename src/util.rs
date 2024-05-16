use std::{fs, thread::panicking};

/// traverses the directories in path from base_dir to find the
/// upper _index_talky.html file and return its contents
pub fn get_custom_template(base_dir: String, path: String) -> Option<String> {
    tracing::event!(
        tracing::Level::INFO,
        "Get custom template {} @ {}",
        &base_dir,
        &path
    );

    let folders = get_path_list(&path);

    let mut template: Option<String> = None;
    for folder in folders {
        //let path = format!("{base_dir}{folder}/_index_talky.html");
        let combined_path =
            easy_paths::get_path_joined(&[&base_dir, &folder, &"_index_talky.html".to_owned()])?;

        match fs::read_to_string(&combined_path) {
            Ok(cusom_template) => {
                tracing::event!(
                    tracing::Level::DEBUG,
                    "Found custom template at {}",
                    &combined_path
                );
                template = Some(cusom_template);
            }
            Err(_e) => {
                // ignore the error
            }
        }
    }

    template
}

/// rewrites a path to an array of paths to the directories in the hierarchy
/// the path may start with a prefix of / or not:
/// "/a/b/c" -> ["/a", "/a/b", "/a/b/c"]
/// "a/b/c" -> ["a", "a/b", "a/b/c"]
pub fn get_path_list(path: &String) -> Vec<String> {
    let mut path = path.to_owned();
    let mut prefix: Option<String> = Some("/".to_owned());
    if path.starts_with('/') {
        path = path[1..path.len()].to_owned();
        prefix = Some("/".to_owned())
    }

    let folders = match path.len() {
        0 => match prefix {
            Some(_) => vec![],
            None => vec!["".to_owned()],
        },
        _ => path.split('/').map(|x| x.to_owned()).collect(),
    };

    let mut result_vec: Vec<String> = vec![];

    for folder in folders {
        let last_element = result_vec.last();

        match last_element {
            Some(element) => {
                // we have the last element of the result vec, join it with folder and add it
                result_vec.push(format!("{}/{}", element, folder));
            }
            None => {
                // result_vec is still empty, add the first folder
                result_vec.push(folder.to_string());
            }
        }
    }

    // if we have a prefix, we need to add it to every element and also add one element with just the prefix in front
    if prefix.is_some() {
        result_vec.insert(0, "".to_owned());
        let pf = prefix.unwrap();

        for e in result_vec.iter_mut() {
            e.insert_str(0, &pf);
        }
    }

    result_vec
}

pub fn format_prefix_path(path: &str) -> String {
    let mut path = path.to_owned();
    if !path.starts_with('/') {
        path = format!("/{path}");
    };

    if !path.ends_with('/') {
        path = format!("{path}/");
    };
    path
}

#[cfg(test)]
mod test {
    use crate::util::get_path_list;

    #[test]
    fn test_get_path_list_a() {
        assert_eq!(
            get_path_list(&"a/b/c".to_owned()),
            vec!["/", "/a", "/a/b", "/a/b/c",]
        );
    }

    #[test]
    fn test_get_path_list_b() {
        assert_eq!(get_path_list(&"".to_owned()), vec!["/"]);
    }

    #[test]
    fn test_get_path_list_c() {
        assert_eq!(get_path_list(&"/".to_owned()), vec!["/"]);
    }

    #[test]
    fn test_get_path_list_d() {
        assert_eq!(get_path_list(&"/a".to_owned()), vec!["/", "/a"]);
    }

    #[test]
    fn test_get_path_prefix_list() {
        assert!(get_path_list(&"/a/b/c".to_owned()) == vec!["/", "/a", "/a/b", "/a/b/c"]);
    }
}

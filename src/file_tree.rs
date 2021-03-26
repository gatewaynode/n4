use serde;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Deserialize, Serialize)]
pub struct DirTree {
    pub absolute_path: String,
    pub relative_path: String,
    pub dir_meta: FileMeta,
    pub files: HashMap<String, FileMeta>,
    pub directories: HashMap<String, DirTree>,
}

impl Default for DirTree {
    fn default() -> Self {
        DirTree {
            absolute_path: String::new(),
            relative_path: String::new(),
            dir_meta: FileMeta::default(),
            files: HashMap::new(),
            directories: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileMeta {
    pub created: f64,
    pub modified: f64,
    pub size: u64,
}

impl Default for FileMeta {
    fn default() -> Self {
        FileMeta {
            created: 0.0,
            modified: 0.0,
            size: 0,
        }
    }
}

pub fn files_in_tree(dir_tree: DirTree) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();

    if dir_tree.files.len() > 0 {
        for filename in dir_tree.files.keys() {
            files.push(format!("{}/{}", dir_tree.relative_path, filename))
        }
    }
    if dir_tree.directories.len() > 0 {
        for _dir_tree in dir_tree.directories {
            files.append(&mut files_in_tree(_dir_tree.1));
        }
    }

    files
}

pub fn get_file_metadata(_this_file_path: PathBuf) -> FileMeta {
    let this_file_metadata = match fs::metadata(_this_file_path) {
        Ok(_this_meta) => _this_meta,
        Err(_err) => panic!("{}", _err), // TODO Remove panic
    };

    FileMeta {
        created: match this_file_metadata.created() {
            Ok(_created) => match _created.duration_since(UNIX_EPOCH) {
                Ok(_created_after) => _created_after.as_secs_f64(),
                Err(_err) => panic!("{}", _err), // TODO Remove panic
            },
            Err(_err) => panic!("{}", _err),
        },
        modified: match this_file_metadata.modified() {
            Ok(_modified) => match _modified.duration_since(UNIX_EPOCH) {
                Ok(_modified_after) => _modified_after.as_secs_f64(),
                Err(_err) => panic!("{}", _err), // TODO Remove panic
            },
            Err(_err) => panic!("{}", _err), // TODO Remove panic
        },
        size: this_file_metadata.len(),
    }
}

pub fn dir_to_tree(path: &str, relative: &str) -> DirTree {
    let mut dir_tree: DirTree = DirTree::default();
    let mut current_path = Path::new(path).components();
    // INFO both paths may not be necessary here, may deprecate absolute_path later
    dir_tree.absolute_path = path.to_string();
    let _temp_string = format!(
        "{}/{}",
        relative,
        current_path
            .nth_back(0)
            .unwrap()
            .as_os_str()
            .to_string_lossy()
    );
    if _temp_string.starts_with("/") {
        dir_tree.relative_path = _temp_string.strip_prefix("/").unwrap().to_string();
    } else {
        dir_tree.relative_path = _temp_string;
    }
    dir_tree.dir_meta = get_file_metadata(PathBuf::from(path));

    let paths = match fs::read_dir(path) {
        Ok(_paths) => _paths,
        Err(_err) => panic!("{}", _err),
    };

    for path in paths {
        let this_path = match path {
            Ok(_potential_path) => _potential_path.path(),
            Err(_err) => panic!("{}", _err), // TODO Remove panic
        };

        let this_file_stem: String = String::from(match this_path.file_stem() {
            Some(_file_stem) => _file_stem.to_string_lossy(),
            None => Cow::Borrowed("ERROR_File_stem_not_parsed"),
        });

        if this_path.is_dir() {
            dir_tree.directories.insert(
                this_file_stem,
                dir_to_tree(&this_path.to_string_lossy(), &dir_tree.relative_path),
            );
        } else {
            dir_tree
                .files
                .insert(this_file_stem, get_file_metadata(this_path));
        }
    }
    dir_tree
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

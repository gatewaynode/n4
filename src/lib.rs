//! N4
//!
//! A flat file based information management system suitable for building web sites or tree based documentation.
//!
/// Picking back up after quite a bit of time away from this.
extern crate dotenv;

use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use chrono;
use chrono::prelude::*;
use dirs;
use markdown;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use v_htmlescape::escape;

// Currently a development dependency
use file_tree::*;

// TODO Temporarily hard coded, move to config file
#[derive(Serialize, Deserialize, Debug)]
pub struct SiteConfig {
    prod_host: String, // Production protocol and FQDN such as https://gatewaynode.com
    xml_priority: String, // Just 0.64 normally for sitemap.xml
    base_dir: String,  // Relative root directory name of the content
    local_content_dir: String, // Absolute path to content directory, concatenated with base dir on end
}

impl Default for SiteConfig {
    fn default() -> Self {
        SiteConfig {
            prod_host: String::from("https://gatewaynode.com"),
            xml_priority: String::from("0.64"),
            base_dir: String::from("website/"),
            local_content_dir: String::from("/home/anon/Documents/gatewaynode_notes/"),
        }
    }
}

pub fn load_config() -> SiteConfig {
    let config_dir = dirs::config_dir();
    dbg!(config_dir);
    let site_config = SiteConfig::default();
    site_config
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PageContent {
    pub markdown: MDContent,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MDContent {
    // pub created: NaiveDateTime,
    pub created: chrono::DateTime<chrono::Utc>,
    pub title: String,
    pub path: String,
    pub body: String,
    pub list: Vec<PageContent>,
    pub meta: ContentMeta,
}

impl Default for MDContent {
    fn default() -> Self {
        MDContent {
            // created: NaiveDateTime::from_timestamp(0, 0),
            created: unix_time_to_iso(0.0),
            title: String::from("None"),
            path: String::from("/"),
            body: String::from("None"),
            list: Vec::new(),
            meta: ContentMeta::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentMeta {
    content_icon: String,
    description: String,
    timestamp_override: Option<chrono::DateTime<chrono::Utc>>,
    weight: u32,
    author: String,
    license: String,
    content_list: Vec<String>,
    content_type: String,
    content_class: String,
    // template_override: String,
    // javascript_include: Vec<String>,
    // javascript_inline: String,
    // css_include: Vec<String>
    // css_inline: String,
}

impl Default for ContentMeta {
    fn default() -> Self {
        ContentMeta {
            content_icon: String::from("/static/images/content_default_icon.svg"),
            description: String::from("Default description value"),
            timestamp_override: None,
            weight: 100,
            author: String::from("Gatewaynode"), //TODO move this to configurable
            license: String::from("cc-by-sa"),
            content_list: Vec::new(),
            content_type: String::from("page"),
            content_class: String::from("basic-page"),
        }
    }
}

// impl ContentMeta {
//     pub fn as_string(Self) -> String {
//     }
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct DirContent {
    modified: chrono::DateTime<chrono::Utc>, //NaiveDateTime,
    title: String,
    relative_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SiteMapEntry {
    pub location: String,
    pub lastmod: DateTime<Utc>,
    pub priority: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MenuItem {
    menu_meta: MenuItemMeta,
    number_of_files: u32,
    relative_path: String,
    children: HashMap<String, MenuItem>,
}

impl Default for MenuItem {
    fn default() -> Self {
        MenuItem {
            menu_meta: MenuItemMeta::default(),
            number_of_files: 0,
            relative_path: "Default".to_string(),
            children: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MenuItemMeta {
    menu_icon: String,   // Really a path to an svg
    description: String, // Used in title attribute for hover detail
    weight: u32,
}

impl Default for MenuItemMeta {
    fn default() -> Self {
        MenuItemMeta {
            menu_icon: String::from("/static/images/menu_default_icon.svg"),
            description: String::from("Menu entry to another section of the site."),
            weight: 100,
        }
    }
}

// TODOget back to this once we have the extensions in the file meta from file_tree
// fn count_md_files(regular_file_list: HashMap<String, file_tree::FileMeta>) -> u32 {
//    0
// }

pub fn read_menu_meta_file(file_path: PathBuf) -> MenuItemMeta {
    let mut content = String::new();

    // File read
    let mut _file = match fs::File::open(&file_path) {
        Err(why) => panic!("Couldn't open file: {}", why),
        Ok(mut _file) => _file.read_to_string(&mut content),
    };
    // Deserialize the JSON
    let return_struct: MenuItemMeta = match serde_json::from_str(&content) {
        Err(why) => panic!("Bad menu meta JSON: {} \n {:#?}", why, content),
        Ok(value) => value,
    };
    return_struct
}

pub fn add_menu_metadata(meta_path_raw: &String) -> MenuItemMeta {
    let meta_path: PathBuf = PathBuf::from(&format!(
        "{}{}",
        meta_path_raw, //.strip_suffix("/").unwrap_or(),
        ".menu_meta"
    ));

    if meta_path.exists() {
        // return MenuItemMeta::default();
        read_menu_meta_file(meta_path)
    } else {
        return MenuItemMeta::default();
    }
}

// TODO pass in prefix to strip from site config
pub fn tree_to_menus(dir_tree: DirTree) -> HashMap<String, MenuItem> {
    let mut menus: HashMap<String, MenuItem> = HashMap::new();
    for (key, value) in dir_tree.directories {
        if value.directories.len() > 0 {
            menus.insert(
                key,
                MenuItem {
                    menu_meta: add_menu_metadata(&value.absolute_path),
                    number_of_files: value.files.len() as u32,
                    relative_path: value
                        .relative_path
                        .strip_prefix("website")
                        .unwrap()
                        .to_string(),
                    children: tree_to_menus(value),
                },
            );
        } else {
            menus.insert(
                key,
                MenuItem {
                    menu_meta: add_menu_metadata(&value.absolute_path),
                    number_of_files: value.files.len() as u32,
                    relative_path: value
                        .relative_path
                        .strip_prefix("website")
                        .unwrap()
                        .to_string(),
                    children: HashMap::new(),
                },
            );
        }
    }
    menus
}

// Oh the things we do to get the correct ISO timestamps
pub fn unix_time_to_iso(timestamp: f64) -> chrono::DateTime<chrono::Utc> {
    let converted_timestamp: i64 = timestamp as i64;
    let naive_datetime = NaiveDateTime::from_timestamp(converted_timestamp, 0);
    let datetime_again: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
    datetime_again
}

fn tree_to_sitemap(dir_tree: DirTree) -> Vec<SiteMapEntry> {
    let config = SiteConfig::default();
    let mut files: Vec<SiteMapEntry> = Vec::new();

    if dir_tree.files.len() > 0 {
        for filename in dir_tree.files.keys() {
            // Strip leading dir in relative path
            let mut stripped_relative_path = String::new();
            if dir_tree.relative_path.ends_with("/") {
                let temp_base_dir = &config.base_dir.strip_suffix("/").unwrap();
                stripped_relative_path = String::from(
                    escape(
                        dir_tree
                            .relative_path
                            .strip_prefix(temp_base_dir)
                            .unwrap_or(""),
                    )
                    .to_string(),
                );
            } else {
                stripped_relative_path = String::from(
                    escape(
                        dir_tree
                            .relative_path
                            .strip_prefix(&config.base_dir)
                            .unwrap_or(""),
                    )
                    .to_string(),
                );
            }
            if &stripped_relative_path.len() > &0 {
                files.push(SiteMapEntry {
                    location: format!(
                        "{}/{}/{}",
                        config.prod_host, stripped_relative_path, filename
                    ),
                    lastmod: unix_time_to_iso(dir_tree.files[filename].modified),
                    priority: config.xml_priority.clone(),
                });
            } else {
                files.push(SiteMapEntry {
                    location: format!("{}/{}", config.prod_host, filename),
                    lastmod: unix_time_to_iso(dir_tree.files[filename].modified),
                    priority: config.xml_priority.clone(),
                });
            }
        }
    }
    if dir_tree.directories.len() > 0 {
        for _dir_tree in dir_tree.directories {
            files.append(&mut tree_to_sitemap(_dir_tree.1));
        }
    }

    files
}

pub fn generate_sitemap() -> Vec<SiteMapEntry> {
    let dir_tree = file_tree::dir_to_tree("/home/anon/Documents/gatewaynode_notes/website", "");

    let sitemap = tree_to_sitemap(dir_tree);

    return sitemap;
}

pub fn generate_content_state() -> file_tree::DirTree {
    let dir_tree = file_tree::dir_to_tree("/home/anon/Documents/gatewaynode_notes/website", "");
    dir_tree
}

pub fn read_md_dirs(dir: &str, rel_path: &str) -> Vec<DirContent> {
    let paths = fs::read_dir(dir).unwrap();
    let mut contents: Vec<DirContent> = Vec::new();

    for item in paths {
        let this_path = &item.unwrap().path();
        if !this_path.is_dir() && this_path.extension().unwrap() == "md" {
            let new_content = DirContent {
                modified: read_file_modified_time(&this_path),
                title: String::from(this_path.file_stem().unwrap().to_string_lossy()),
                relative_path: String::from(rel_path),
            };
            contents.push(new_content);
        } else if this_path.is_dir() {
            let dir_name: String = String::from(this_path.file_stem().unwrap().to_string_lossy());
            let new_rel_path: String = format!("{}/{}", rel_path, dir_name);
            // Recursive call here
            contents.append(&mut read_md_dirs(
                &this_path.to_string_lossy(),
                &new_rel_path,
            ));
        }
    }
    contents.sort_unstable_by_key(|x| x.modified); // Not sure if this needs to be sorted this way now
                                                   // println!("{:?}", contents);
    contents
}

// pub fn read_top_ten()

fn localpath_to_webpath(this_path: &std::path::PathBuf) -> String {
    let config = SiteConfig::default();
    let mut rel_path = this_path.to_string_lossy().to_string();
    let offset = rel_path.find(&config.base_dir).unwrap() + config.base_dir.len(); // I think panic here is ok as it will break the site generally
    rel_path.replace_range(..offset, "/");
    rel_path.strip_suffix(".md").unwrap().to_string()
}

pub fn read_full_dir_sorted(dir: &str) -> Vec<PageContent> {
    let paths = fs::read_dir(dir).unwrap();
    let mut pages: HashMap<String, PageContent> = HashMap::new();

    for item in paths {
        let this_path = &item.unwrap().path(); // Just easier this way
        if !&this_path.is_dir() {
            let file_stem: String = String::from(this_path.file_stem().unwrap().to_string_lossy());
            if !pages.contains_key(&file_stem) {
                pages.insert(file_stem.clone(), PageContent::default());
            }
            if this_path.extension().unwrap() == "md" {
                let mut new_content = MDContent {
                    created: read_file_creation_time(&this_path),
                    title: file_stem.clone(),
                    path: localpath_to_webpath(this_path),
                    body: read_markdown_from_path(&this_path),
                    list: Vec::new(),
                    meta: ContentMeta::default(),
                };
                // Check for metadata and load that if it exists
                if check_path_alternatives(this_path, ".content_meta") {
                    let content_meta_path =
                        String::from(this_path.to_string_lossy()).replace(".md", ".content_meta");
                    new_content.meta = read_content_meta_file(PathBuf::from(content_meta_path));
                }
                pages.get_mut(&file_stem).unwrap().markdown = new_content;
            }
        }
    }
    // Convert to Vec for sorting
    let mut contents: Vec<PageContent> = Vec::new();
    for (_key, value) in pages.drain() {
        contents.push(value);
    }

    contents.sort_unstable_by_key(|x| x.markdown.created);
    contents
}

// Mainly for reading the content_meta content_list values prefixes local dir and document base dir
pub fn read_content_list(list_o_content: &Vec<String>) -> Vec<PageContent> {
    let config = SiteConfig::default();
    let mut page_list: Vec<PageContent> = Vec::new();
    for item in list_o_content {
        let temp_string = format!("{}{}{}", config.local_content_dir, config.base_dir, item);
        let this_path = PathBuf::from(temp_string);
        if !&this_path.is_dir() && this_path.exists() {
            page_list.push(read_single_page(this_path.as_path()));
        } else {
            println!("Content list failure.");
            dbg!(list_o_content);
        }
    }

    page_list.sort_unstable_by_key(|x| x.markdown.meta.weight);
    page_list
}

pub fn read_single_page(this_path: &std::path::Path) -> PageContent {
    let mut page_content: PageContent = PageContent::default();
    let file_stem: String = String::from(this_path.file_stem().unwrap().to_string_lossy());

    // Ensure markdown extension and convert markdown to HTML string
    if this_path.extension().unwrap() == "md" {
        page_content.markdown = MDContent {
            created: read_file_creation_time(&this_path),
            title: file_stem,
            path: localpath_to_webpath(&PathBuf::from(this_path)), //this_path.to_string_lossy().to_string().clone(),
            body: read_markdown_from_path(&this_path),
            list: Vec::new(),
            meta: ContentMeta::default(),
        };
    }
    // Check for metadata and load that if it exists
    if check_path_alternatives(this_path, ".content_meta") {
        let content_meta_path =
            String::from(this_path.to_string_lossy()).replace(".md", ".content_meta");
        page_content.markdown.meta = read_content_meta_file(PathBuf::from(content_meta_path));
    }
    // If the meta file contains a content_list load that into the MDContent list
    // This is essentially recursive

    if page_content.markdown.meta.content_list.len() > 0 {
        page_content.markdown.list = read_content_list(&page_content.markdown.meta.content_list);
    }
    page_content
}

pub fn read_content_meta_file(file_path: PathBuf) -> ContentMeta {
    let mut content_meta = String::new();

    // File read
    let mut _file = match fs::File::open(&file_path) {
        Err(why) => panic!("Couldn't open file: {}", why),
        Ok(mut _file) => _file.read_to_string(&mut content_meta),
    };
    // Deserialize the JSON
    let return_struct: ContentMeta = match serde_json::from_str(&content_meta) {
        Err(why) => {
            // TODO This should trigger an integrity check and correct the JSON file with default values if
            // possible while preserving existing values.
            let mut error_meta = ContentMeta::default();
            error_meta.description = format!("JSON Parse Error: {}", why);
            error_meta
        }
        Ok(value) => value,
    };
    return_struct
}

// This function looks for a base markdown file by extension and returns TRUE if it exists, confirming there is a piece
// of content that inherits CSS or JSON
// TODO Add an input validation layer here, check for illegal escape attempts and return False if found
pub fn check_path_alternatives(this_path: &Path, extension: &str) -> bool {
    let path_check_str: String = this_path.to_string_lossy().replace(".md", extension);
    let new_path: &Path = Path::new(&path_check_str);
    new_path.exists()
}

// TODO Deprecate in favor of file_tree data
pub fn read_file_creation_time(path: &std::path::Path) -> chrono::DateTime<chrono::Utc> {
    //NaiveDateTime {
    let metadata = fs::metadata(path).expect("Not found");

    let _ = match metadata.created() {
        Err(why) => panic!("Couldn't get file metadata: {}", why),
        Ok(_time) => {
            let _temp_time = _time.duration_since(UNIX_EPOCH).unwrap().as_secs() as f64;
            return unix_time_to_iso(_temp_time); //NaiveDateTime::from_timestamp(_temp_time, 0);
        }
    };
}

// TODO Deprecate in favor of file_tree data
pub fn read_file_modified_time(path: &std::path::Path) -> chrono::DateTime<chrono::Utc> {
    //NaiveDateTime {
    let metadata = fs::metadata(path).expect("Not found");

    let _ = match metadata.modified() {
        Err(why) => panic!("Couldn't get file metadata: {}", why),
        Ok(_time) => {
            let _temp_time = _time.duration_since(UNIX_EPOCH).unwrap().as_secs() as f64;
            return unix_time_to_iso(_temp_time); //NaiveDateTime::from_timestamp(_temp_time, 0);
        }
    };
    // unix_time_to_iso()
}

// TODO The following functions are place holders for the same but with strong validation

pub fn read_markdown_from_path(path: &std::path::Path) -> String {
    let mut content = String::new();
    let mut _file = match fs::File::open(&path) {
        Err(why) => panic!("Couldn't open file: {}", why),
        Ok(mut _file) => match _file.read_to_string(&mut content) {
            Err(why) => panic!("Couldn't read file: {}", why),
            Ok(_) => return markdown::to_html(&content),
        },
    };
}

pub fn read_html_from_path(path: &std::path::Path) -> String {
    let mut content = String::new();
    let mut _file = match fs::File::open(&path) {
        Err(why) => panic!("Couldn't open file: {}", why),
        Ok(mut _file) => match _file.read_to_string(&mut content) {
            Err(why) => panic!("Couldn't read file: {}", why),
            Ok(_) => return content,
        },
    };
}
pub fn read_json_from_path(path: &std::path::Path) -> String {
    let mut content = String::new();
    let mut _file = match fs::File::open(&path) {
        Err(why) => panic!("Couldn't open file: {}", why),
        Ok(mut _file) => match _file.read_to_string(&mut content) {
            Err(why) => panic!("Couldn't read file: {}", why),
            Ok(_) => return content,
        },
    };
}

pub fn read_css_from_path(path: &std::path::Path) -> String {
    let mut content = String::new();
    let mut _file = match fs::File::open(&path) {
        Err(why) => panic!("Couldn't open file: {}", why),
        Ok(mut _file) => match _file.read_to_string(&mut content) {
            Err(why) => panic!("Couldn't read file: {}", why),
            Ok(_) => return content,
        },
    };
}

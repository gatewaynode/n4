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

// Currently a development dependency, probably going to move to a module
use file_tree::*;

/// Struct to hold the site configuration
///
/// prod_host
///     sitemap-data: Production protocol and FQDN such as https://slashdot.org/
/// xml_priority: String
///     sitemap-data: Just 0.64 normally for sitemap.xml
/// base_dir
///     content-data: Relative root directory name of the content
/// local_content_dir
///     content-data: Absolute path to content directory, concatenated with base dir on end
#[derive(Serialize, Deserialize, Debug)]
pub struct SiteConfig {
    pub prod_host: String,
    pub xml_priority: String,
    pub base_dir: String,
    pub local_content_dir: String,
}

impl SiteConfig {
    pub fn local_path(self) -> String {
        format!("{}{}", self.local_content_dir, self.base_dir)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PageContent {
    pub markdown: MDContent,
    pub html: Option<HTMLContent>,
    pub json: Option<JSONContent>,
    pub list: Vec<PageContent>,
    pub meta: ContentMeta,
    pub section_meta: MenuItemMeta,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MDContent {
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    // pub path: String,
    pub body: String,
    // pub list: Vec<PageContent>, // TODO move to meta file
    // pub meta: ContentMeta,
}

impl Default for MDContent {
    fn default() -> Self {
        MDContent {
            created: unix_time_to_iso(0.0),
            modified: unix_time_to_iso(0.0),
            // path: String::from("/"),
            body: String::from("Default value"),
            // list: Vec::new(), // TODO move to meta file
            // meta: ContentMeta::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HTMLContent {
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    // pub path: String,
    pub body: String,
}

impl Default for HTMLContent {
    fn default() -> Self {
        HTMLContent {
            created: unix_time_to_iso(0.0),
            modified: unix_time_to_iso(0.0),
            // path: String::from("/"),
            body: String::from("None"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JSONContent {
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    // pub path: String,
    pub body: String,
}

impl Default for JSONContent {
    fn default() -> Self {
        JSONContent {
            created: unix_time_to_iso(0.0),
            modified: unix_time_to_iso(0.0),
            // path: String::from("/"),
            body: String::from("None"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentMeta {
    pub title: String,
    pub path: String,
    pub content_icon: String,
    pub description: String,
    pub weight: u32,
    pub author: String,
    pub license: String,
    pub content_list: Vec<String>,
    pub content_type: String,
    content_class: String,
    pub template_override: String,
    javascript_include: Vec<String>,
    javascript_inline: String,
    css_include: Vec<String>,
    css_inline: String,
    created_time_default: String,
    modified_time_default: String,
}

impl Default for ContentMeta {
    fn default() -> Self {
        ContentMeta {
            title: String::from("Default ContentMeta struct title"),
            path: String::from("/"),
            content_icon: String::from("/static/images/content_default_icon.svg"),
            description: String::from("Default description value"),
            weight: 100,
            author: String::from("Default"), //TODO move this to configurable
            license: String::from("cc-by-sa"),
            content_list: Vec::new(),
            content_type: String::from("page"),
            content_class: String::from("basic-page"),
            template_override: String::from(""),
            javascript_include: Vec::new(),
            javascript_inline: String::from(""),
            css_include: Vec::new(),
            css_inline: String::from(""),
            created_time_default: String::from("markdown"),
            modified_time_default: String::from("markdown"),
        }
    }
}

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
    pub menu_icon: String,   // Really a path to an svg
    pub description: String, // Used in title attribute for hover detail
    pub weight: u32,
    pub section_template: String, // This is intended to be a new default for all content in the directory
    pub template_override: String, // This override is for just the index page of the directory
    pub content_type: String,
    section_class: String,                   // This is an inherited body class
    content_class: String,                   // Not inherited, just for the directory index page
    section_javascript_include: Vec<String>, // Inherited
    javascript_include: Vec<String>,
    javascript_inline: String,
    section_css_include: Vec<String>, // Inherited
    css_include: Vec<String>,
    css_inline: String,
}

impl Default for MenuItemMeta {
    fn default() -> Self {
        MenuItemMeta {
            menu_icon: String::from("/static/images/menu_default_icon.svg"),
            description: String::from("Menu default description."),
            weight: 100,
            section_template: String::from("article"),
            template_override: String::from(""),
            content_type: String::from("directory"),
            section_class: String::from("section"),
            content_class: String::from("directory-page"),
            section_javascript_include: Vec::new(),
            javascript_include: Vec::new(),
            javascript_inline: String::from(""),
            section_css_include: Vec::new(),
            css_include: Vec::new(),
            css_inline: String::from(""),
        }
    }
}

pub fn load_config() -> SiteConfig {
    let mut site_config = String::new();
    let mut config_file_path: PathBuf = match dirs::config_dir() {
        Some(val) => PathBuf::from(val),
        _ => panic!("No system config value found!"),
    };
    config_file_path.push("n4");
    config_file_path.push("default.json");
    // File read
    let mut _file = match fs::File::open(&config_file_path) {
        Err(why) => panic!("Couldn't open config file: {}", why),
        Ok(mut _file) => _file.read_to_string(&mut site_config),
    };
    // Deserialize the JSON
    let deserialized_site_config: SiteConfig = match serde_json::from_str(&site_config) {
        Err(why) => panic!("Config couldn't be deserialized: {}", why),
        Ok(value) => value,
    };
    deserialized_site_config
}

/// Creates the standard user config directory and an empty config JSON file
/// Meant to be called from the CLI
pub fn setup_config() {
    let mut config_dir: PathBuf = match dirs::config_dir() {
        Some(val) => PathBuf::from(val),
        _ => panic!("No system config value found!"),
    };

    config_dir.push("n4");
    if config_dir.exists() {
        println!(
            "Config dir exists!  Edit the file at {}/default.json",
            &config_dir.to_string_lossy()
        )
    } else {
        match std::fs::create_dir(&config_dir) {
            Err(why) => panic!("Directory couldn't be created: {}", why),
            _ => println!("Created!"),
        };
    }

    config_dir.push("default.json");
    if config_dir.exists() {
        panic!("Default config already exists.  Exiting.");
    } else {
        let default_config = SiteConfig {
            prod_host: String::from("https://localhost:8000"),
            xml_priority: String::from("0.64"),
            base_dir: String::from("/"),
            local_content_dir: String::from("/"),
        };
        let mut file = match fs::File::create(config_dir) {
            Err(why) => panic!("File creation fail: {}", why),
            Ok(value) => value,
        };
        let serialized_config = match serde_json::to_string_pretty(&default_config) {
            Err(why) => panic!("Serialize to json fail: {}", why),
            Ok(value) => value,
        };
        match file.write_all(&serialized_config.as_bytes()) {
            Err(why) => panic!("Could not write to config file: {}", why),
            Ok(_) => println!("Default config file created."),
        };
    }
}

/// Generate a simple robots.txt file
pub fn generate_robot_food() -> String {
    let config = load_config();
    format!(
        "User-agents: *
Allow: *

Sitemap: {}/sitemap.xml",
        config.prod_host
    )
}

// This really just breaks out the file read and JSON deserialize into it's own function
pub fn read_menu_meta_file(file_path: PathBuf) -> MenuItemMeta {
    let mut content = String::new();

    // File read
    let mut _file = match fs::File::open(&file_path) {
        Err(why) => panic!("Couldn't open file: {}", why),
        Ok(mut _file) => _file.read_to_string(&mut content),
    };
    // Deserialize the JSON
    let return_struct: MenuItemMeta = match serde_json::from_str(&content) {
        Err(why) => {
            println!("Bad menu meta JSON: {} \n {:#?}", why, content); // TODO Change to logging
            return MenuItemMeta::default();
        }
        Ok(value) => value,
    };
    return_struct
}

// Formats a path to a directory for the .menu_meta extension and checks if it exists
pub fn add_menu_metadata(meta_path_raw: &String) -> MenuItemMeta {
    let meta_path: PathBuf = PathBuf::from(&format!("{}{}", meta_path_raw, ".menu_meta"));

    if meta_path.exists() {
        read_menu_meta_file(meta_path)
    } else {
        return MenuItemMeta::default();
    }
}

pub fn tree_to_menus(dir_tree: DirTree) -> HashMap<String, MenuItem> {
    let mut menus: HashMap<String, MenuItem> = HashMap::new();
    let config = load_config();
    let prefix_to_strip = match config.base_dir.strip_suffix("/") {
        Some(val) => val,
        _ => panic!("Base dir is missing the trailing directory delimiter."),
    };
    for (key, value) in dir_tree.directories {
        if value.directories.len() > 0 {
            menus.insert(
                key,
                MenuItem {
                    menu_meta: add_menu_metadata(&value.absolute_path),
                    number_of_files: value.files.len() as u32,
                    relative_path: value
                        .relative_path
                        .strip_prefix(prefix_to_strip)
                        .unwrap()
                        .to_string(),
                    children: tree_to_menus(value), // Recursion
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
                        .strip_prefix(prefix_to_strip)
                        .unwrap()
                        .to_string(),
                    children: HashMap::new(), // Blank default
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
    let config = load_config();
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
    let config = load_config();
    let dir_tree = file_tree::dir_to_tree(&config.local_path(), "");

    let sitemap = tree_to_sitemap(dir_tree);

    return sitemap;
}

pub fn generate_content_state() -> file_tree::DirTree {
    let config = load_config();
    let dir_tree = file_tree::dir_to_tree(&config.local_path(), "");
    dir_tree
}

// TODO Rename this function to something clearer
pub fn read_full_dir_sorted(web_path_dir: String) -> Vec<ContentMeta> {
    let local_path = webpath_to_localpath(web_path_dir);
    let paths = match fs::read_dir(&local_path) {
        Err(why) => panic!("Dir exists but can't be read: {}", why),
        Ok(val) => val,
    };
    let mut page_metas: Vec<ContentMeta> = Vec::new();
    let mut entries_read: Vec<String> = Vec::new(); // We just need one metafile read per content file track it here
    for dir_entry in paths {
        let check_path = match &dir_entry {
            Err(why) => panic!("Well this was an unexpected entry in a dir: {}", why),
            Ok(val) => val.path(),
        };
        let this_path = &check_path.to_string_lossy().to_string();
        if !&check_path.is_dir() && !this_path.ends_with("meta") {
            if !entries_read.iter().any(|x| {
                // If we already read it, it's in the entries Vec so skip
                x == &check_path
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            }) {
                entries_read.push(
                    check_path
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                );
                page_metas.push(read_content_meta(&this_path));
            }
        }
    }
    page_metas.sort_unstable_by_key(|x| x.weight);
    page_metas
}

// Mainly for reading the content_meta content_list values prefixes local dir and document base dir
pub fn read_content_list(list_o_content: &Vec<String>) -> Vec<PageContent> {
    let mut page_list: Vec<PageContent> = Vec::new();
    for item in list_o_content {
        if does_content_exist(item.clone()) {
            page_list.push(read_single_page(item.clone()));
        } else {
            println!("Content list failure.  This doesn't exist: {}", item);
        }
    }

    page_list.sort_unstable_by_key(|x| x.meta.weight);
    page_list
}

/// This is a compositional function to pull the parts together into a page.  Each component load also breaks down
/// further into file system operations, parsing and such.
///
/// Parameters:
///     this_path(String), a web path most likely delivered by the web server routing
/// Returns:
///     PageContent, struct containing all the pieces of a content page
pub fn read_single_page(this_path: String) -> PageContent {
    let config = load_config();

    let full_path_string = format!("{}{}", config.local_path(), &this_path);
    let mut page_content: PageContent = PageContent::default();

    // SET SECTION META
    page_content.section_meta = read_section_meta(&full_path_string);
    // SET CONTENT META
    page_content.meta = read_content_meta(&full_path_string);
    // SET MARKDOWN CONTENT
    page_content.markdown = read_markdown_content(&full_path_string);
    // SET HTML CONTENT
    page_content.html = read_html_content(&full_path_string);
    // SET JSON CONTENT
    page_content.json = read_json_content(&full_path_string);

    // If the meta file contains a content_list of web paths, load the content from that list
    // into the PageContent.list Vec.
    // NOTE: This is recursive in an unsafe way, a circular reference will break things here
    if page_content.meta.content_list.len() > 0 {
        page_content.list = read_content_list(&page_content.meta.content_list);
    }

    page_content
}

/// Take a String turn it into a pathbuf and read the content meta if it has it.
///
/// NOTE: Unlike the other simple readers, this one will create a default, customize it a bit and save it
/// if the metafile doesn't exist so our page can still render somewhat correctly and we can modify the
/// values manually.
///
/// Parameters:
///     full_path_string(&String), the absolute path in the filesystem for the metafile
/// Returns:
///     ContentMeta, The metafile struct for content
pub fn read_content_meta(full_path_string: &String) -> ContentMeta {
    let mut this_path = PathBuf::from(full_path_string);
    this_path.set_extension("content_meta");
    if this_path.exists() {
        let this_content_meta = read_content_meta_file(this_path);
        return this_content_meta;
    } else {
        let mut new_meta = ContentMeta::default();
        new_meta.title = string_from_stem(&this_path);
        new_meta.path = localpath_to_webpath(&this_path);
        save_content_meta_file(&this_path, &new_meta);
        return new_meta;
    }
}

fn read_markdown_content(this_path_string: &String) -> MDContent {
    let mut markdown_path = PathBuf::from(this_path_string);
    markdown_path.set_extension("md");
    if markdown_path.exists() {
        let markdown_content = MDContent {
            created: read_file_creation_time(&markdown_path),
            modified: read_file_modified_time(&markdown_path),
            body: read_markdown_from_path(&markdown_path), //TODO Lint/Validate/Filter here?
        };
        return markdown_content;
    } else {
        let mut markdown_content = MDContent::default();
        markdown_content.body = format!(
            "Markdown file does not exist: {}",
            markdown_path.to_string_lossy()
        );
        return markdown_content;
    }
}

fn read_html_content(this_path_string: &String) -> Option<HTMLContent> {
    let mut html_path = PathBuf::from(this_path_string);
    html_path.set_extension("html");
    if html_path.exists() {
        let html_content = HTMLContent {
            created: read_file_creation_time(&html_path),
            modified: read_file_modified_time(&html_path),
            body: read_html_from_path(&html_path), //TODO Lint/Validate/Filter here?
        };
        return Some(html_content);
    } else {
        // let mut html_content = HTMLContent::default();
        // html_content.body = format!("HTML file does not exist: {}", html_path.to_string_lossy());
        // return html_content;
        return None;
    }
}

fn read_json_content(this_path_string: &String) -> Option<JSONContent> {
    let mut json_path = PathBuf::from(this_path_string);
    json_path.set_extension("json");
    if json_path.exists() {
        let json_content = JSONContent {
            created: read_file_creation_time(&json_path),
            modified: read_file_modified_time(&json_path),
            body: read_json_from_path(&json_path), //TODO Lint/Validate/Filter here?
        };
        return Some(json_content);
    } else {
        // let mut json_content = JSONContent::default();
        // json_content.body = format!("JSON file does not exist: {}", json_path.to_string_lossy());
        // return json_content;
        return None;
    }
}

/// Just wraps the .filestem() method to always return a string even if it's an error.
fn string_from_stem(this_path: &PathBuf) -> String {
    let this_string = match this_path.file_stem() {
        Some(val) => val.to_string_lossy().to_string(),
        _ => String::from("Default file stem value ERROR."),
    };
    this_string
}

/// Standard set of filesystem and serialization operations to save a content metafile
fn save_content_meta_file(this_path: &PathBuf, metadata: &ContentMeta) {
    let mut file = match fs::File::create(this_path) {
        Err(why) => panic!("Content meta default file creation fail: {}", why),
        Ok(value) => value,
    };
    let serialized_meta = match serde_json::to_string_pretty(&metadata) {
        Err(why) => panic!("Serialize to json fail: {}", why),
        Ok(value) => value,
    };
    match file.write_all(&serialized_meta.as_bytes()) {
        Err(why) => panic!(
            "A default metafile couldn't be created for this path: {}, {}",
            this_path.to_string_lossy(),
            why
        ),
        Ok(val) => val,
    };
}

/// File system read and deserialization of a ContentMeta file
pub fn read_content_meta_file(file_path: PathBuf) -> ContentMeta {
    let mut content_meta = String::new();

    // File read
    let mut _file = match fs::File::open(&file_path) {
        Err(why) => panic!(
            "Couldn't open content meta file: {} -> {}",
            &file_path.to_string_lossy(),
            why
        ),
        Ok(mut _file) => _file.read_to_string(&mut content_meta),
    };
    // Deserialize the JSON
    let return_struct: ContentMeta = match serde_json::from_str(&content_meta) {
        Err(why) => {
            // TODO This should trigger an integrity check and correct the JSON file with default values if
            // possible while preserving existing values.
            let mut error_meta = ContentMeta::default();
            error_meta.description = format!("JSON Parse Error: {}", why);
            error_meta.title = "Error parsing metadata file".to_string();
            error_meta
        }
        Ok(value) => value,
    };
    return_struct
}

// For a given piece of content pull the directory menu_meta file as section meta or return a default
pub fn read_section_meta(content_location: &String) -> MenuItemMeta {
    let config = load_config();
    let mut this_path = PathBuf::from(format!("{}{}", config.local_path(), content_location));
    this_path.pop();
    this_path.set_extension("menu_meta");
    if this_path.exists() {
        let this_menu_meta = read_menu_meta_file(this_path);
        return this_menu_meta;
    } else {
        return MenuItemMeta::default();
    }
}

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

pub fn read_file_modified_time(path: &std::path::Path) -> chrono::DateTime<chrono::Utc> {
    //NaiveDateTime {
    let metadata = fs::metadata(path).expect("Not found");

    let _ = match metadata.modified() {
        Err(why) => panic!("Couldn't get file metadata: {}", why), // TODO Remove panic
        Ok(_time) => {
            let _temp_time = _time.duration_since(UNIX_EPOCH).unwrap().as_secs() as f64;
            return unix_time_to_iso(_temp_time); //NaiveDateTime::from_timestamp(_temp_time, 0);
        }
    };
    // unix_time_to_iso()
}

//
// INFO Potential section of file system operations to move to a module
//

fn localpath_to_webpath(this_localpath: &std::path::PathBuf) -> String {
    let config = load_config();
    let mut extensionless_path = this_localpath.clone();
    extensionless_path.set_extension("");
    let mut rel_path = extensionless_path.to_string_lossy().to_string();
    // offset is necessary for replace range, this is the calculation of it
    let offset = rel_path.find(&config.base_dir).unwrap() + config.base_dir.len(); // I think panic here is ok as it will break the site generally anyway to send anything incorrect
    rel_path.replace_range(..offset, "/");
    return rel_path;
}

fn webpath_to_localpath(this_webpath: String) -> String {
    let config = load_config();
    let this_local_path = format!("{}{}", config.local_path(), this_webpath);
    return this_local_path;
}

// This function looks for a given extension variant for a string of a path
// TODO Add an input validation layer here, check for illegal escape attempts and return False if found
// TODO TODO This is probably not even necessary anymore given the PathBuf.set_extension() method now
pub fn check_path_alternatives(this_path: &String, extension: &str) -> bool {
    let mut this_path = PathBuf::from(this_path);
    this_path.set_extension(extension);
    this_path.exists()
}

/// Checks a given webpath to see if the base content exists in one of the three formats by extension
///
/// Parameters:
///     potential_content_webpath (String), should be a web renderable path
/// Returns:
///     bool, does it exist?
pub fn does_content_exist(potential_content_webpath: String) -> bool {
    let mut this_path = PathBuf::from(webpath_to_localpath(potential_content_webpath));
    this_path.set_extension("md");
    if this_path.exists() {
        return true;
    }
    this_path.set_extension("html");
    if this_path.exists() {
        return true;
    }
    this_path.set_extension("json");
    if this_path.exists() {
        return true;
    }
    false
}

pub fn does_directory_exist(potential_content_webpath: String) -> bool {
    // Maybe a good place for a directory blacklist?
    let this_path = webpath_to_localpath(potential_content_webpath);
    return Path::new(&this_path).is_dir();
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

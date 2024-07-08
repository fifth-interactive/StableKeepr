use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{PathBuf};
use config;
use serde_derive;
use dirs_next;
use walkdir::WalkDir;
use comfyui_workflow_parser::Workflow;

#[derive(Debug, Default, serde_derive::Deserialize, serde_derive::Serialize, PartialEq, Eq)]
struct Settings {
    debug: bool,
    // array of directories where images are stored
    image_directories: Vec<String>,
}

struct Chunks {
    workflow: Option<Workflow>
}

fn main() {
    // TODO: make window size, position, and state persistent (https://gtk-rs.org/gtk4-rs/git/book/saving_window_state.html) (save in config file)
    println!("Hello, world!");

    let default_config = Settings {
        debug: false,
        image_directories: vec![],
    };

    let config_file = match dirs_next::config_dir() {
        Some(mut path) => {
            path.push("StableKeeper");
            path.push("config.json");
            path
        },
        None => panic!("Could not determine config directory")
    };

    println!("{:?}", config_file);

    if !config_file.try_exists().expect("Error checking if config file exists!") {
        // Create a default config file
        let json = serde_json::to_string_pretty(&default_config).expect("Error serializing default config!");
        create_dir_all(&config_file.parent().expect("Failed to get config directory!")).expect("Failed to create application config directory!");
        let mut f = File::create(&config_file).expect("Failed to create config file!");
        f.write_all(json.as_bytes()).expect("Failed to write to config file!");
    }

    let s = config::Config::builder()
        .add_source(config::File::with_name(config_file.to_str().expect("Config file not found!")))
        .add_source(config::Environment::with_prefix("SK_APP"))
        .build()
        .expect("Error building config!");

    let settings: Settings = s.try_deserialize().expect("Could not deserialize settings!");

    // debug output the settings
    println!("{:?}", settings);

    // we need at least one path
    if settings.image_directories.len() < 1 {
        panic!("Need at least one image directory set!");
    }

    // load the first directory as a path
    let output_dir = PathBuf::from(settings.image_directories.first().unwrap());

    if !output_dir.try_exists().expect("Error checking if image directory exists!") {
        panic!("Image directory doesn't exist!");
    }

    let mut file_paths: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(&output_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            file_paths.push(entry.path().to_path_buf());
        }
    }

    println!("{} files found", file_paths.len());
    println!("{}", file_paths[14500].display());
    let image = File::open(&file_paths[14500]).unwrap();
    let decoder = png::Decoder::new(image);
    let mut reader = decoder.read_info().unwrap();
    let mut data = Chunks { workflow: None, };
    for text_chunk in &reader.info().uncompressed_latin1_text {
        match text_chunk.keyword.as_str() {
            "workflow" => {
                data.workflow = Some(Workflow::new(text_chunk.text.as_str()).unwrap());
            }
            _ => { println!("Unknown chunk type: {:?}", text_chunk.keyword) }
        }
    }

    if let Some(_) = &data.workflow {
        println!("{:?}", data.workflow.as_ref().unwrap().find_prompts_for_node(data.workflow.as_ref().unwrap().find_outputs().unwrap().first().unwrap()).unwrap().positive);
    }

}
use std::cell::OnceCell;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{PathBuf};
use config;
use serde_derive;
use dirs_next;
use iced::{Element, Sandbox, Settings};
use image::{GenericImageView, ImageResult};
use walkdir::WalkDir;
use comfyui_workflow_parser::{Prompts, Workflow};
use image::io::Reader as ImageReader;
use iced::widget::{button, Column};
use iced::widget::text;
use iced::widget::column;

#[derive(Debug, Default, serde_derive::Deserialize, serde_derive::Serialize, PartialEq, Eq, Clone)]
struct AppSettings {
    debug: bool,
    // array of directories where images are stored
    image_directories: Vec<String>,
    // number of threads for background tasks
    threads: usize,
    // number of threads for blocking tasks
    blocking_threads: usize,
}

struct Chunks {
    workflow: Option<Workflow>
}

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

impl Sandbox for Counter {
    type Message = Message;
    fn new() -> Self {
        Counter { value: 0 }
    }
    fn title(&self) -> String {
        String::from("Counter")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }
    fn view(&self) -> Element<Message> {
        column![
            button("+").on_press(Message::Increment),
            text(self.value),
            button("-").on_press(Message::Decrement),
        ]
            .padding(20)
            .align_items(iced::Alignment::Center)
            .into()
    }
}

fn main() -> iced::Result {
    // TODO: make window size, position, and state persistent (https://gtk-rs.org/gtk4-rs/git/book/saving_window_state.html) (save in config file)
    println!("Hello, world!");

    let mut settings = AppSettings {
        debug: false,
        image_directories: vec![],
        threads: 4,
        blocking_threads: 4,
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
        let json = serde_json::to_string_pretty(&settings).expect("Error serializing default config!");
        create_dir_all(&config_file.parent().expect("Failed to get config directory!")).expect("Failed to create application config directory!");
        let mut f = File::create(&config_file).expect("Failed to create config file!");
        f.write_all(json.as_bytes()).expect("Failed to write to config file!");
    }

    let s = config::Config::builder()
        .add_source(config::Config::try_from(&settings).expect("Error serializing default config!"))
        .add_source(config::File::with_name(config_file.to_str().expect("Config file not found!")))
        .add_source(config::Environment::with_prefix("SK_APP"))
        .build()
        .expect("Error building config!");

    settings = s.try_deserialize().expect("Could not deserialize settings!");

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

    println!("{}", file_paths[10].display());
    let image = File::open(&file_paths[10]).unwrap();
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
        println!("{:?}", data.workflow.as_ref().unwrap().find_prompts_for_node(data.workflow.as_ref().unwrap().find_outputs().unwrap().first().unwrap()).unwrap());
    }

    Counter::run(Settings::default())
}
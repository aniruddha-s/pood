extern crate hyper;
extern crate xml;

use hyper::Client;
use std::env;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::process;
use std::path::Path;
use xml::reader::{EventReader, XmlEvent};

struct Episode {
    title: String,
    description: String,
    url: String,
    date: String,
    duration: String,
}

struct Podcast {
    title: String,
    description: String,
    episodes: Vec<Episode>,
}

impl Episode {
    fn new() -> Episode {
        Episode {
            title: "".to_string(),
            description: "".to_string(),
            url: "".to_string(),
            date: "".to_string(),
            duration: "".to_string(),
        }
    }
}

fn get_data_from_url(url: &String) -> Podcast {
    let mut title = String::new();
    let mut description = String::new();
    let mut episodes = Vec::new();

    // Send http request
    let client = Client::new();
    let mut response = client.get(url)
        .send()
        .unwrap();

    // Read response, init parser
    let mut xml = String::new();
    response.read_to_string(&mut xml).unwrap();
    let parser = EventReader::from_str(&xml);

    // Parse the response
    let mut in_item_tag = false;
    let mut last_tag = String::new();

    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                match name.local_name.as_ref() {
                    "item" => {
                        episodes.push(Episode::new());
                        in_item_tag = true;
                    },
                    "enclosure" => {
                        for attrib in attributes {
                            if attrib.name.local_name != "url" { continue; }
                            episodes.last_mut().unwrap().url = attrib.value;
                            break;
                        }
                    },
                    _ => {}
                }

                last_tag = name.local_name;
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name.local_name == "item" {
                    in_item_tag = false;
                }
            }
            Ok(XmlEvent::Characters(data)) => {
                match last_tag.as_ref() {
                    "title" =>
                        if in_item_tag { episodes.last_mut().unwrap().title = data; }
                        else           { title = data; },
                    "description" =>
                        if in_item_tag { episodes.last_mut().unwrap().description = data; }
                        else           { description = data; },
                    "pubDate" =>
                        episodes.last_mut().unwrap().date = data,
                    "duration" =>
                        episodes.last_mut().unwrap().duration = data,
                    _ => {}
                }
            }
            Err(event) => {
                println!("Error: {}", event);
                break;
            }
            _ => {}
        }
    }

    Podcast { title: title, description: description, episodes: episodes }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // TODO: Remove this and create stuff in the current directory
    let base_path = "/home/apoorvaj/Music/podcasts/".to_string();

    // Parse options
    // TODO: Print friendly error messages on invalid arg count
    match args[1].as_ref() {
        "info" => {
            // TODO: Display info in a sane way
            if args.len() == 3 {
                let podcast = get_data_from_url(&args[2]);
                println!("{}", podcast.title);
                println!("{}", podcast.description);
                for episode in podcast.episodes.iter().rev() {
                    println!("    + {}", episode.title);
                    println!("          {}, {}, {}", episode.duration
                                                   , episode.date
                                                   , episode.url);
                    println!("          {}", episode.description);
                }

            }
        },
        "add" => {
            let podcast = get_data_from_url(&args[2]);
            let podcast_folder: String = base_path + &podcast.title;

            // Create folder if it doesn't exist
            if !Path::new(&podcast_folder).exists() {
                std::fs::create_dir(&podcast_folder).unwrap();
            }

            // Create the pood.yaml file inside the newly created folder
            let file_name: String = podcast_folder + "/pood.yaml";
            if !Path::new(&file_name).exists() {
                let mut file = OpenOptions::new()
                            .create_new(true)
                            .read(true)
                            .write(true)
                            .open(file_name).unwrap();
                let yaml = format!("title : {}\n\
                                    url : {}\n",
                                    podcast.title,
                                    &args[2]);
                file.write_all(yaml.as_bytes()).unwrap();
            } else {
                println!("Podcast already exists in the current folder");
                process::exit(0);
            }
            
        }
        _ => {}
    }
}


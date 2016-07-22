// pood : A command-line podcast manager
//
// TODO:
// - Better error messages
//

extern crate hyper;
extern crate xml;

use hyper::Client;
use std::env;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::process;
use std::path::PathBuf;
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
    url: String,
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
                    "description" | "summary" =>
                        if in_item_tag { episodes.last_mut().unwrap().description = data; }
                        else           { description = data; },
                    "pubDate" =>
                        if in_item_tag { episodes.last_mut().unwrap().date = data; },
                    "duration" =>
                        episodes.last_mut().unwrap().duration = data,
                    _ => {}
                }
            }
            Ok(XmlEvent::CData(data)) => {
                if last_tag == "summary" {
                    description = data;
                } else if last_tag == "description" && in_item_tag {
                    episodes.last_mut().unwrap().description = data;
                }
            }
            Err(event) => {
                println!("Error: {}", event);
                break;
            }
            _ => {}
        }
    }

    episodes.reverse();
    Podcast {
        title       : title,
        description : description,
        url         : url.to_string(),
        episodes    : episodes
    }
}

fn get_data_from_yaml(path: PathBuf) -> Podcast {
    // If yaml file doesn't exist, error and exit
    if !path.exists() {
        println!("pood.yaml not found. Use \"pood add [podcast_url]\" to add a \
                 podcast.");
        process::exit(0);
    }

    let file = File::open(path).unwrap();
    let file = BufReader::new(file);

    let mut title    = String::new();
    let mut url      = String::new();
    let mut episodes = Vec::new();

    for line in file.lines() {
        let line = line.unwrap();
        if line.contains("title :") {
            title = line.replace("title : ", "");
        } else if line.contains("url :") {
            url = line.replace("url : ", "");
        }
    }

    Podcast {
        title       : title,
        description : String::new(),
        url         : url.to_string(),
        episodes    : episodes
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut path = std::env::current_dir().unwrap();

    // Parse options
    // TODO: Print friendly error messages on invalid arg count
    match args[1].as_ref() {
        "info" => {
            // TODO: Display info in a sane way
            if args.len() == 3 {
                let podcast = get_data_from_url(&args[2]);
                println!("{}", podcast.title);
                println!("{}", podcast.description);
                for episode in podcast.episodes {
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
            let sanitized_title = podcast.title.replace(" ", "_").replace("'","");
            path.push(sanitized_title);

            // Create folder if it doesn't exist
            if !path.exists() {
                std::fs::create_dir(&path).unwrap();
            }

            // Create the pood.yaml file inside the newly created folder
            path.push("pood.yaml");
            if !path.exists() {
                let mut file = OpenOptions::new()
                            .create_new(true)
                            .read(true)
                            .write(true)
                            .open(path).unwrap();
                let yaml = format!("title : {}\nurl : {}\nepisodes :\n",
                                    podcast.title,
                                    &args[2]);
                file.write_all(yaml.as_bytes()).unwrap();
            } else {
                println!("Podcast already exists in the current folder");
                process::exit(0);
            }
        }
        "sync" => {
            // Parse local yaml to get podcast url and existing episodes
            path.push("pood.yaml");
            let file_podcast = get_data_from_yaml(path);

            // Fetch podcast from url
            let web_podcast = get_data_from_url(&file_podcast.url);
            println!("{}", web_podcast.description);
        },
        _ => {}
    }
}


extern crate hyper;
extern crate xml;

use hyper::Client;
use std::env;
use std::io::Read;
use std::io::Write;
use xml::reader::{EventReader, XmlEvent};

struct Episode {
    title: String,
    description: String,
    url: String,
    date: String,
    duration: String,
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

fn main() {
    let args: Vec<String> = env::args().collect();
    // let base_path = "~/Music/podcasts";

    // Parse options
    if args.len() == 3 && args[1] == "info" {
        let ref url = args[2];

        // Send http request
        let client = Client::new();
        let mut response = client.get(url)
            .send()
            .unwrap();

        // Read response, init parser
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();
        let parser = EventReader::from_str(&body);

        // Parse the response
        let mut tags: Vec<String> = Vec::new();
        let mut title = "".to_string();
        let mut episodes: Vec<Episode> = Vec::new();

        for event in parser {
            match event {
                Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                    if name.local_name == "item" {
                        episodes.push(Episode::new());
                    }
                    tags.push(name.local_name);
                }
                Ok(XmlEvent::EndElement { .. }) => {
                    tags.pop();
                }
                Ok(XmlEvent::Characters(data)) => {
                    let parent: &str = match tags.last() {
                        None => "",
                        Some(x) => x
                    };
                    let grandparent: &str = match tags.get(tags.len() - 2) {
                        None => "",
                        Some(x) => x
                    };

                    if grandparent == "channel" && parent == "title" {
                        title = data;
                    } else if grandparent == "item" && episodes.len() > 0 {
                        match parent.as_ref() {
                            "title" => episodes.last_mut().unwrap().title = data,
                            _ => {}
                        }
                        // if parent == "title" {
                        //     println!("EPISODE: {}", data);
                        //     episodes.push(data);
                        // } else if parent == 
                    }
                }
                Err(event) => {
                    println!("Error: {}", event);
                    break;
                }
                _ => {}
            }
        }

        println!("{}", title);
        for episode in episodes {
            println!("    {}", episode.title);
        }
        // let mut file = OpenOptions::new().write(true)
        //                                  .create(true)
        //                                  .open(name).unwrap();
        // let _ = file.write(body.as_bytes());
    }

    println!("==============");
}

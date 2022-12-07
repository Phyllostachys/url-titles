use std::io::BufRead;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use rayon::prelude::*;
use select::{document::Document, predicate::Name};
use structopt::StructOpt;
use tokio::runtime::Runtime;
use ytextract::video;

fn process_yt(url: &String, client: &ytextract::Client) -> String {
    let mut result = String::new();

    // if it's a URL to a video...
    if let Some(index) = url.find("v=") {
        let (_, video_id) = url.split_at(index + 2);
        let video_id: video::Id = match video_id.parse() {
            Ok(i) => i,
            Err(_) => return String::new(),
        };
        let rt = Runtime::new().unwrap();
        let video = match rt.block_on(client.video(video_id)) {
            Ok(v) => v,
            Err(_) => return String::new(),
        };

        result.push_str(video.channel().name());
        result.push(' ');
        result.push_str(format!("({})", video.date()).as_str());
        result.push(' ');
        result.push_str(url);
        result.push_str(" -- ");
        result.push_str(video.title());
    }

    result
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Options {
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    #[structopt(short, long, default_value = "3")]
    chunk_size: usize,

    #[structopt(short, long, default_value = "1.5")]
    delay_time: f32,
}

fn main() {
    let opt = Options::from_args();
    let file = std::fs::File::open(opt.input).unwrap();
    let client = ytextract::Client::new();
    let buf = std::io::BufReader::new(file);
    let mut urls: Vec<String> = buf
        .lines()
        .filter_map(|line| match line {
            Ok(s) => {
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            }
            Err(_) => None,
        })
        .collect();

    urls.chunks_mut(opt.chunk_size).for_each(|url_group| {
        let result: Vec<String> = url_group
            .par_iter()
            .map(|url| {
                if url.contains("youtube") {
                    process_yt(url, &client)
                } else {
                    let res = reqwest::blocking::get(url).unwrap();
                    let content = res.text().unwrap();
                    let document = Document::from(content.as_str());

                    let mut result = String::new();
                    result.push_str(url);
                    result.push_str(" --- ");
                    if let Some(node) = document.find(Name("title")).next() {
                        result.push_str(node.text().as_str());
                    } else {
                        result.push_str("Title not found");
                    }
                    result
                }
            })
            .collect();

        result.iter().for_each(|url| println!("{}", url));
        thread::sleep(Duration::from_secs_f32(opt.delay_time));
    });
}

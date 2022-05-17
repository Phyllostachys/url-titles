use std::io::BufRead;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use rayon::prelude::*;
use select::{document::Document, predicate::Name};
use structopt::StructOpt;
use tokio::runtime::Runtime;
use ytextract::video;

#[derive(Clone)]
struct UrlTitle {
    url: String,
    title: String,
}

impl UrlTitle {
    pub fn new(url: &String) -> UrlTitle {
        UrlTitle {
            url: String::from(url),
            title: String::new(),
        }
    }

    pub fn process_yt(&mut self, client: &ytextract::Client) {
        if let Some(index) = self.url.find("v=") {
            let (_, video_id) = self.url.split_at(index + 2);
            let video_id: video::Id = match video_id.parse() {
                Ok(i) => i,
                Err(_) => return,
            };
            let rt = Runtime::new().unwrap();
            let video = match rt.block_on(client.video(video_id)) {
                Ok(v) => v,
                Err(_) => return,
            };
            self.title
                .push_str(format!("({})", video.date().to_string()).as_str());
            self.title.push_str(" ");
            self.title.push_str(video.title());
        }
    }
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
        .filter_map(|line| line.ok())
        .map(|url| String::from(url))
        .collect();

    for group in urls.chunks_exact_mut(opt.chunk_size) {
        let result: Vec<UrlTitle> = group
            .par_iter()
            .map(|url| {
                if url.contains("youtube") {
                    let mut ut = UrlTitle::new(url);
                    ut.process_yt(&client);
                    ut
                } else {
                    let res = reqwest::blocking::get(url).unwrap();
                    let content = res.text().unwrap();
                    let document = Document::from(content.as_str());
                    UrlTitle {
                        url: String::from(url),
                        title: if let Some(node) = document.find(Name("title")).next() {
                            node.text()
                        } else {
                            String::new()
                        },
                    }
                }
            })
            .collect();

        for url in result {
            println!("{} --- {}", url.url, url.title);
        }

        thread::sleep(Duration::from_secs_f32(opt.delay_time));
    }
}

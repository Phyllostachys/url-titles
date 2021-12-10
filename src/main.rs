use std::io::prelude::*;
use std::path::PathBuf;

use structopt::StructOpt;
use rayon::prelude::*;
use select::{document::Document, predicate::Name};

#[derive(Clone)]
struct UrlTitle {
    url: String,
    title: String,
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

    let file = match std::fs::File::open(opt.input) {
        Ok(f) => f,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let buf = std::io::BufReader::new(file);
    let mut urls: Vec<UrlTitle> = Vec::new();
    for line in buf.lines() {
        match line {
            Ok(l) => urls.push(UrlTitle {
                url: l,
                title: String::new(),
            }),
            Err(e) => println!("Error on getting line... {}", e),
        }
    }

    for group in urls.chunks_exact_mut(opt.chunk_size) {
        let result: Vec<_> = group
            .par_iter()
            .map(|url| {
                let res = reqwest::blocking::get(String::from(&url.url)).unwrap();
                let content = res.text().unwrap();
                let document = Document::from(content.as_str());
                UrlTitle {
                    url: String::from(&url.url),
                    title: if let Some(node) = document.find(Name("title")).next() {
                        //url.title = node.text();
                        node.text()
                    } else {
                        //url.title = String::new();
                        String::new()
                    },
                }
            })
            .collect();

        for url in result {
            println!("{} --- {}", url.url, url.title);
        }

        std::thread::sleep(std::time::Duration::from_secs_f32(opt.delay_time));
    }
}

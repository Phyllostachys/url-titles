use std::io::BufRead;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use rayon::prelude::*;
use select::{document::Document, predicate::Name};
use structopt::StructOpt;
use youtube_dl::YoutubeDl;

enum ProcessError {
    FailedToProcessWithYtDlp,
}

fn process_yt(url: &String) -> Result<String, ProcessError> {
    let mut result = String::new();

    let ytdlp_out = match YoutubeDl::new(url)
        .socket_timeout("15")
        .run()
    {
        Ok(o) => o,
        Err(e) => {
            println!("Failed to get result from yt-dlp - {e}");
            return Err(ProcessError::FailedToProcessWithYtDlp);
        }
    };
    let video = ytdlp_out.into_single_video().unwrap();

    result.push_str(video.channel.unwrap().as_str());
    result.push(' ');
    result.push_str(video.upload_date.unwrap().as_str());
    result.push(' ');
    result.push_str(url);
    result.push_str(" -- ");
    result.push_str(video.title.unwrap().as_str());
    Ok(result)
}

fn process_normal(url: &String) -> String {
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
                    if let Ok(r) = process_yt(url) {
                        r
                    } else {
                        process_normal(url)
                    }
                } else {
                    process_normal(url)
                }
            })
            .collect();

        result.iter().for_each(|url| println!("{}", url));
        thread::sleep(Duration::from_secs_f32(opt.delay_time));
    });
}

use clap::Parser;
use image::DynamicImage;
use lcs_png_diff::diff;
use rusty_pool::ThreadPool;
use serde::Deserialize;
use std::error::Error;
use std::fs::create_dir_all;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct DiffPair {
    before: String,
    after: String,
    result: Option<String>,
}

#[derive(Parser, Debug)]
#[clap(about, author, long_about = None, version)]
struct Args {
    /// Path to the before png
    #[clap(short, long)]
    before_png: Option<String>,

    /// Path to the after png
    #[clap(short, long)]
    after_png: Option<String>,

    /// Path to the diff result png
    #[clap(short, long)]
    diff_png: Option<String>,

    /// Path to the batch diff json file
    #[clap(short = 'j', long)]
    batch_json: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let before_png = args.before_png;
    let after_png = args.after_png;
    let diff_png = args.diff_png;
    let batch_json = args.batch_json;

    let pairs: Vec<DiffPair>;
    if let Some(batch) = batch_json.as_deref() {
        let file = File::open(batch)?;
        let reader = BufReader::new(file);
        pairs = serde_json::from_reader(reader).unwrap();
    } else {
        pairs = vec![DiffPair {
            before: before_png.unwrap(),
            after: after_png.unwrap(),
            result: Some(diff_png.unwrap()),
        }]
    }
    let pool = ThreadPool::default();
    for pair in pairs.into_iter() {
        pool.execute(move || generate_diff(pair));
    }
    pool.shutdown_join();
    Ok(())
}

fn generate_diff(pair: DiffPair) {
    let timer = Instant::now();
    let result_filename = match pair.result {
        Some(p) => p,
        None => add_suffix_to_file_name(&pair.before, &"_result"),
    };
    let before: DynamicImage = image::open(&pair.before).unwrap();
    let after: DynamicImage = image::open(&pair.after).unwrap();
    let result_png: DynamicImage = diff(&before, &after).unwrap();
    save_png(&result_png, &result_filename);
    println!("{}: {:?}", result_filename, timer.elapsed());
}

fn save_png(image: &DynamicImage, filename: &str) {
    let path = Path::new(filename).parent().unwrap();
    let _result = mkdirp(path);
    image.save(filename).unwrap();
}

fn mkdirp<P: AsRef<Path>>(p: P) -> io::Result<()> {
    if let Err(e) = create_dir_all(p) {
        if e.kind() != io::ErrorKind::AlreadyExists {
            return Ok(());
        }
        return Err(e);
    }
    Ok(())
}

pub fn add_suffix_to_file_name(file_name: &str, suffix: &str) -> String {
    let path = Path::new(file_name);
    let file_basename = path.file_stem().unwrap();
    let dir = path.parent().unwrap();
    if dir.to_str().unwrap().is_empty() {
        return format!("{}{}{}", file_basename.to_str().unwrap(), suffix, ".png");
    }
    format!(
        "{}/{}{}{}",
        dir.to_str().unwrap(),
        file_basename.to_str().unwrap(),
        suffix,
        ".png"
    )
}

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
#[clap(about, author, long_about = None, version, arg_required_else_help=true)]
struct Args {
    /// Path to the after png
    #[clap(short, long)]
    after_png: Option<String>,

    /// Path to the before png
    #[clap(short, long)]
    before_png: Option<String>,

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

    let pairs = if let Some(batch) = batch_json {
        let file = File::open(batch)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)?
    } else {
        vec![DiffPair {
            after: after_png.unwrap(),
            before: before_png.unwrap(),
            result: diff_png,
        }]
    };
    let pool = ThreadPool::default();
    for pair in pairs {
        pool.execute(move || generate_diff(pair));
    }
    pool.shutdown_join();
    Ok(())
}

/// Generate the png diff image from the input pair
fn generate_diff(pair: DiffPair) {
    let timer = Instant::now();
    let result_filename = match pair.result {
        Some(p) => p,
        None => add_suffix_to_file_name(&pair.before, "_result"),
    };
    let before = image::open(&pair.before).expect("Unable to parse before png bitmap");
    let after = image::open(&pair.after).expect("Unable to parse after png bitmap");
    let result_png =
        diff(&before, &after).expect("Error occurred while processing the diff result");
    save_png(&result_png, &result_filename);
    println!("{}: {:?}", result_filename, timer.elapsed());
}

/// Save the png to a file
fn save_png(image: &DynamicImage, filename: &str) {
    let path = Path::new(filename).parent().unwrap();
    let _ = mkdirp(path);
    image
        .save(filename)
        .expect("Unable to save the diff result bitmap as a png file");
}

/// Create the whole path if it doesn't exist
fn mkdirp<P: AsRef<Path>>(p: P) -> io::Result<()> {
    if let Err(e) = create_dir_all(p) {
        if e.kind() != io::ErrorKind::AlreadyExists {
            return Ok(());
        }
        return Err(e);
    }
    Ok(())
}

/// Add a suffix to the path
fn add_suffix_to_file_name(file_name: &str, suffix: &str) -> String {
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

#[test]
fn happy_path() {
    let pair = DiffPair {
        before: "tests/fixtures/backstopjs_pricing.png".to_owned(),
        after: "tests/fixtures/backstopjs_pricing_after.png".to_owned(),
        result: None,
    };

    generate_diff(pair);

    let result = image::open("tests/fixtures/backstopjs_pricing_result.png");
    println!("{:?}", result);
}

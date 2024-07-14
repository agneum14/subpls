use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use clap::Parser;
use operations::extract_srt;
use rust_translate::supported_languages::get_languages;
use translate::translated_srt;

use crate::operations::tracks;

mod operations;
mod translate;

#[derive(Parser)]
#[command(version, about)]
/// Automatically generate translated SRT files for your media
struct Cli {
    /// The target language ISO 639-1 Code. Defaults to "en".
    target_language: Option<String>,

    /// List of MKV files / directories with MKV files
    files: Vec<PathBuf>,
}

fn file_paths(path: PathBuf) -> Vec<PathBuf> {
    if path.is_file() {
        vec![path]
    } else if path.is_dir() {
        let paths = fs::read_dir(path).unwrap();
        paths
            .into_iter()
            .map(|x| x.unwrap().path())
            .flat_map(|x| file_paths(x))
            .collect::<Vec<_>>()
    } else {
        vec![]
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let target_language = cli.target_language.unwrap_or("en".to_owned());
    let mkv_paths = cli
        .files
        .into_iter()
        .flat_map(|x| file_paths(x))
        .filter(|x| x.file_name().unwrap().to_str().unwrap().ends_with(".mkv"))
        .collect::<Vec<_>>();

    for mkv_path in mkv_paths.iter() {
        let tracks = tracks(mkv_path).unwrap();
        let track = match tracks.iter().filter(|x| x.language == "en").last() {
            Some(v) => Some(v),
            None => tracks
                .iter()
                .filter(|x| get_languages().iter().any(|y| x.language == *y))
                .last(),
        };
        let track = track.unwrap();

        let original_srt = extract_srt(mkv_path, track).unwrap();
        let srt = translated_srt(&target_language, &original_srt)
            .await
            .unwrap();

        let srt_path = mkv_path.to_str().unwrap().to_owned();
        let srt_path = srt_path
            .chars()
            .take(srt_path.len() - 4)
            .chain(format!(".{}.srt", &target_language).chars())
            .collect::<String>();

        let mut srt_file = File::create(srt_path).unwrap();
        srt_file.write_all(srt.as_bytes()).unwrap();
    }
}

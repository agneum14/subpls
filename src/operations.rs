use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug)]
pub struct Track {
    id: usize,
    pub language: String,
}

pub struct Srt {
    pub language: String,
    pub content: String,
}

pub fn tracks(mkv_path: &Path) -> Result<Vec<Track>> {
    let stdout = Command::new("mkvinfo")
        .arg(mkv_path.to_str().context("invalid path")?)
        .output()?
        .stdout;
    let output = String::from_utf8(stdout).unwrap();

    let data = output
        .split("|+ Tracks")
        .last()
        .context("invalid mkvinfo")?
        .split("| + Track")
        .filter(|x| x.contains("|  + Track type: subtitles"))
        .map(|x| x.lines().skip(1).collect::<Vec<_>>())
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>();

    let tracks = data
        .into_iter()
        .map(|x| {
            let id = x
                .iter()
                .filter(|x| x.starts_with("|  + Track number:"))
                .last()
                .unwrap()
                .split_whitespace()
                .collect::<Vec<_>>()[4]
                .parse::<usize>()
                .unwrap()
                - 1;

            let language = if x
                .iter()
                .any(|y| y.starts_with("|  + Language (IETF BCP 47):"))
            {
                x.iter()
                    .filter(|x| x.starts_with("|  + Language (IETF BCP 47):"))
                    .last()
                    .unwrap()
                    .split_whitespace()
                    .collect::<Vec<_>>()[6]
                    .to_owned()
            } else {
                "en".to_owned()
            };

            Track { id, language }
        })
        .collect::<Vec<_>>();

    Ok(tracks)
}

pub fn extract_srt(mkv: &PathBuf, track: &Track) -> Result<Srt> {
    let mut srt_path = PathBuf::new();
    srt_path.push(mkv.parent().context("invalid mkv parent")?);
    let srt_file_name = format!(
        "{}.tmpsrt",
        mkv.file_name()
            .context("invalid mkv")?
            .to_str()
            .context("invalid mkv file name")?
    );
    srt_path.push(Path::new(&srt_file_name));

    let _ = Command::new("mkvextract")
        .arg(mkv.to_str().context("invalid mkv path").unwrap())
        .arg("tracks")
        .arg(format!("{}:{}", track.id, srt_path.to_str().unwrap()))
        .output()?;

    let content = String::from_utf8(fs::read(srt_path.clone())?)?;
    let _ = fs::remove_file(srt_path);

    Ok(Srt {
        content,
        language: track.language.clone(),
    })
}

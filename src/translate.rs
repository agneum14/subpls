use std::collections::VecDeque;

use anyhow::{anyhow, Result};
use futures::future::try_join_all;
use rust_translate::translate;

const DOUBLE_NEWLINE: &str = "\n\n";
const NEWLINE: &str = "\n";

async fn translated_srt(source_language: &str, srt: &str) -> Result<String> {
    let sections = srt
        .split(DOUBLE_NEWLINE)
        .into_iter()
        .map(|x| x.lines().collect::<Vec<_>>())
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>();

    let words = sections
        .iter()
        .map(|x| x.iter().skip(2).copied().collect::<Vec<_>>().join(NEWLINE))
        .collect::<Vec<_>>();

    let translated_words = words.into_iter().map(|x| {
        let source_language = source_language.to_owned();
        tokio::spawn(async move {
            translate(&x, &source_language, "zh")
                .await
                .map_err(|_| anyhow!(""))
        })
    });
    let translated_words = try_join_all(translated_words)
        .await?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    let new_srt = sections
        .into_iter()
        .map(|x| x.into_iter().take(2).collect::<Vec<_>>().join(NEWLINE))
        .zip(translated_words.iter())
        .map(|(mut x, y)| {
            x.push_str(NEWLINE);
            x.push_str(y);
            x
        })
        .collect::<Vec<_>>()
        .join(DOUBLE_NEWLINE);

    Ok(new_srt)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::fs;
//
//     #[tokio::test]
//     async fn translate_squid_game() {
//         let srt = fs::read_to_string(
//             "test_data/Squid Game - The Challenge - S01E09 - Circle of Trust WEBRip-1080p.en.srt",
//         )
//         .unwrap();
//         translated_srt("en", &srt).await;
//     }
// }

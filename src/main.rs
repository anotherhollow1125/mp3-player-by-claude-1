use anyhow::{Context, Result};
use clap::Parser;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "mp3-player")]
#[command(about = "A simple MP3 CLI player")]
struct Args {
    #[arg(help = "Path to MP3 file or directory containing MP3 files")]
    path: PathBuf,

    #[arg(short, long, help = "Play files recursively from subdirectories")]
    recursive: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mp3_files = find_mp3_files(&args.path, args.recursive)?;

    if mp3_files.is_empty() {
        println!("No MP3 files found in the specified path.");
        return Ok(());
    }

    println!("Found {} MP3 file(s)", mp3_files.len());

    let (_stream, stream_handle) =
        OutputStream::try_default().context("Failed to create audio output stream")?;

    for (i, file_path) in mp3_files.iter().enumerate() {
        println!(
            "Playing [{}/{}]: {}",
            i + 1,
            mp3_files.len(),
            file_path.display()
        );

        if let Err(e) = play_mp3_file(file_path, &stream_handle) {
            eprintln!("Error playing {}: {}", file_path.display(), e);
            continue;
        }
    }

    Ok(())
}

fn find_mp3_files(path: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    if is_mp3_file(path) {
        return Ok(vec![path.to_path_buf()]);
    }

    if !path.is_dir() {
        return Ok(vec![]);
    }

    // path is directory

    let musics: Vec<_> = std::fs::read_dir(path)
        .context("Failed to read directory")?
        .map(|entry| -> Result<Vec<PathBuf>> {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            let musics = if recursive && path.is_dir() {
                find_mp3_files(&path, recursive)?
            } else if is_mp3_file(&path) {
                vec![path.to_path_buf()]
            } else {
                vec![]
            };

            Ok(musics)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(musics)
}

fn is_mp3_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase() == "mp3")
            .unwrap_or(false)
}

fn play_mp3_file(file_path: &Path, stream_handle: &rodio::OutputStreamHandle) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

    let source = Decoder::new(BufReader::new(file))
        .with_context(|| format!("Failed to decode MP3 file: {}", file_path.display()))?;

    let sink = Sink::try_new(stream_handle).context("Failed to create audio sink")?;

    sink.append(source);

    sink.sleep_until_end();

    thread::sleep(Duration::from_millis(100));

    Ok(())
}

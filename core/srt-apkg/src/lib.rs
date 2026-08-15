//! Minimal helpers for reading and writing Anki `.apkg` ZIP archives.
//!
//! `.apkg` files are plain ZIP archives. This crate provides two functions
//! that are shared by `srt-flashcards` and `srt-refine` to avoid duplication:
//!
//! - [`unzip_to`]: extract a ZIP archive into a directory.
//! - [`zip_from_dir`]: create a ZIP archive from the flat contents of a directory.

use std::fs;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

fn is_media_extension(ext: &str) -> bool {
    matches!(
        ext,
        "mp3"
            | "m4a"
            | "wav"
            | "ogg"
            | "opus"
            | "flac"
            | "aac"
            | "jpg"
            | "jpeg"
            | "png"
            | "webp"
            | "gif"
            | "mp4"
            | "mkv"
            | "avi"
            | "mov"
            | "webm"
            | "ttf"
            | "otf"
            | "woff"
            | "woff2"
    )
}

/// Extract the ZIP archive at `zip_path` into `dest_dir`.
///
/// Existing files in `dest_dir` are overwritten. Subdirectories found inside
/// the archive are created as needed.
pub fn unzip_to(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("Cannot open ZIP archive: {e}"))?;
    let reader = BufReader::with_capacity(128 * 1024, file);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("Invalid ZIP archive: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("ZIP index error: {e}"))?;
        let outpath = match entry.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if entry.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = outpath.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Cannot create extracted file: {e}"))?;
            let mut writer = BufWriter::with_capacity(128 * 1024, outfile);
            io::copy(&mut entry, &mut writer)
                .map_err(|e| format!("Error writing extracted file: {e}"))?;
        }
    }
    Ok(())
}

/// Create a ZIP archive at `zip_path` containing every *file* (non-recursive)
/// directly inside `src_dir`.
///
/// Pre-compressed media files use Stored (0 compression) to save CPU/time,
/// while other files use Deflate.
pub fn zip_from_dir(src_dir: &Path, zip_path: &Path) -> Result<(), String> {
    let file = fs::File::create(zip_path).map_err(|e| format!("Cannot create output ZIP: {e}"))?;
    let writer = BufWriter::with_capacity(256 * 1024, file);
    let mut zip = zip::ZipWriter::new(writer);
    let options_deflated = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let options_stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let entries =
        fs::read_dir(src_dir).map_err(|e| format!("Cannot read source directory: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| "Invalid filename in source directory".to_string())?;
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let opt = if is_media_extension(&ext) {
                options_stored
            } else {
                options_deflated
            };
            zip.start_file(filename, opt)
                .map_err(|e| format!("ZIP start_file error: {e}"))?;
            let f = fs::File::open(&path).map_err(|e| format!("Cannot read source file: {e}"))?;
            let mut reader = BufReader::with_capacity(128 * 1024, f);
            io::copy(&mut reader, &mut zip).map_err(|e| format!("ZIP copy error: {e}"))?;
        }
    }

    zip.finish().map_err(|e| format!("ZIP finish error: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_media_extension() {
        assert!(is_media_extension("mp3"));
        assert!(is_media_extension("wav"));
        assert!(is_media_extension("webp"));
        assert!(is_media_extension("jpg"));
        assert!(is_media_extension("mp4"));
        assert!(is_media_extension("ttf"));

        assert!(!is_media_extension("txt"));
        assert!(!is_media_extension("json"));
        assert!(!is_media_extension("anki2"));
        assert!(!is_media_extension(""));
    }

    #[test]
    fn round_trip() {
        let src = tempfile::tempdir().unwrap();
        let mut f = fs::File::create(src.path().join("hello.txt")).unwrap();
        f.write_all(b"hello apkg").unwrap();

        let mut f_media = fs::File::create(src.path().join("audio.mp3")).unwrap();
        f_media.write_all(b"fake audio data").unwrap();

        let zip_path = src.path().join("out.zip");
        zip_from_dir(src.path(), &zip_path).unwrap();
        assert!(zip_path.exists());

        // Verify compression methods in the created zip archive
        let zip_file = fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(zip_file).unwrap();
        for i in 0..archive.len() {
            let entry = archive.by_index(i).unwrap();
            if entry.name() == "audio.mp3" {
                assert_eq!(entry.compression(), zip::CompressionMethod::Stored);
            } else if entry.name() == "hello.txt" {
                assert_eq!(entry.compression(), zip::CompressionMethod::Deflated);
            }
        }

        let dest = tempfile::tempdir().unwrap();
        unzip_to(&zip_path, dest.path()).unwrap();
        let content = fs::read_to_string(dest.path().join("hello.txt")).unwrap();
        assert_eq!(content, "hello apkg");
        let media_content = fs::read(dest.path().join("audio.mp3")).unwrap();
        assert_eq!(media_content, b"fake audio data");
    }
}

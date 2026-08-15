//! Minimal helpers for reading and writing Anki `.apkg` ZIP archives.
//!
//! `.apkg` files are plain ZIP archives. This crate provides two functions
//! that are shared by `srt-flashcards` and `srt-refine` to avoid duplication:
//!
//! - [`unzip_to`]: extract a ZIP archive into a directory.
//! - [`zip_from_dir`]: create a ZIP archive from the flat contents of a directory.

use std::fs;
use std::io;
use std::path::Path;

/// Extract the ZIP archive at `zip_path` into `dest_dir`.
///
/// Existing files in `dest_dir` are overwritten. Subdirectories found inside
/// the archive are created as needed.
pub fn unzip_to(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| format!("Cannot open ZIP archive: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP archive: {e}"))?;

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
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Cannot create extracted file: {e}"))?;
            io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Error writing extracted file: {e}"))?;
        }
    }
    Ok(())
}

/// Create a ZIP archive at `zip_path` containing every *file* (non-recursive)
/// directly inside `src_dir`.
///
/// Files are stored with Deflate compression using their plain filename (no
/// directory path) as the archive entry name.
pub fn zip_from_dir(src_dir: &Path, zip_path: &Path) -> Result<(), String> {
    let file = fs::File::create(zip_path).map_err(|e| format!("Cannot create output ZIP: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

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
            zip.start_file(filename, options)
                .map_err(|e| format!("ZIP start_file error: {e}"))?;
            let mut f =
                fs::File::open(&path).map_err(|e| format!("Cannot read source file: {e}"))?;
            io::copy(&mut f, &mut zip).map_err(|e| format!("ZIP copy error: {e}"))?;
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
    fn round_trip() {
        let src = tempfile::tempdir().unwrap();
        let mut f = fs::File::create(src.path().join("hello.txt")).unwrap();
        f.write_all(b"hello apkg").unwrap();

        let zip_path = src.path().join("out.zip");
        zip_from_dir(src.path(), &zip_path).unwrap();
        assert!(zip_path.exists());

        let dest = tempfile::tempdir().unwrap();
        unzip_to(&zip_path, dest.path()).unwrap();
        let content = fs::read_to_string(dest.path().join("hello.txt")).unwrap();
        assert_eq!(content, "hello apkg");
    }
}

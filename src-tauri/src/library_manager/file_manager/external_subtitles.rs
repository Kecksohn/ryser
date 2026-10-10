//! Discovery of external subtitle files named `<video name>[.<lang>].<ext>`,
//! either next to the video or in a dedicated subtitle folder.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::super::lang_map::to_iso_639_1;

// ponytail: hard-coded; becomes a user-chosen list of subtitle folders per library.
const SUBTITLE_FOLDER: &str = "F:/mov/_subtitles";

// VobSub is picked up via its .idx; a .sub without one loads in neither VLC nor MPC-HC.
const SUBTITLE_EXTENSIONS: [&str; 5] = ["srt", "vtt", "ass", "ssa", "idx"];

pub(crate) struct ExternalSubtitle {
    pub path: String,
    /// ISO 639-1, or `"und"` when the filename carries no language.
    pub language: String,
    pub title: String,
}

/// Directory listings by folder, so refreshing a whole library reads each folder once.
pub(crate) type DirCache = HashMap<PathBuf, Vec<PathBuf>>;

fn list_dir<'a>(dir: &Path, cache: &'a mut DirCache) -> &'a [PathBuf] {
    cache.entry(dir.to_path_buf()).or_insert_with(|| {
        let mut files: Vec<PathBuf> = fs::read_dir(dir)
            .map(|entries| entries.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        files.sort();
        files
    })
}

/// Language of a subtitle whose (lowercased) stem is `<video_stem>[.<lang>]`:
/// `"und"` without a suffix, `None` if the file does not belong to this video.
fn match_language(sub_stem: &str, video_stem: &str) -> Option<String> {
    let rest = sub_stem.strip_prefix(video_stem)?;
    if rest.is_empty() {
        return Some("und".to_string());
    }
    to_iso_639_1(rest.strip_prefix('.')?).map(|l| l.to_string())
}

/// Language of the first stream declared in a VobSub index (`id: en, index: 0`).
fn idx_language(path: &Path) -> Option<String> {
    let content = fs::read(path).ok()?;
    let content = String::from_utf8_lossy(&content);
    let id = content.lines().find_map(|l| l.strip_prefix("id: "))?;
    to_iso_639_1(id.split(',').next()?).map(|l| l.to_string())
}

pub(crate) fn find_external_subtitles(video_path: &str, cache: &mut DirCache) -> Vec<ExternalSubtitle> {
    let video = Path::new(video_path);
    let (Some(video_dir), Some(video_stem)) =
        (video.parent(), video.file_stem().and_then(|s| s.to_str()))
    else {
        return vec![];
    };
    let video_stem = video_stem.to_lowercase();

    let mut found = vec![];
    for (dir, origin) in [(video_dir, ""), (Path::new(SUBTITLE_FOLDER), ", subtitle folder")] {
        for path in list_dir(dir, cache) {
            let (Some(stem), Some(ext), Some(path_str)) = (
                path.file_stem().and_then(|s| s.to_str()),
                path.extension().and_then(|e| e.to_str()),
                path.to_str(),
            ) else {
                continue;
            };
            let ext = ext.to_lowercase();
            if !SUBTITLE_EXTENSIONS.contains(&ext.as_str()) {
                continue;
            }
            let Some(mut language) = match_language(&stem.to_lowercase(), &video_stem) else {
                continue;
            };
            if language == "und" && ext == "idx" {
                language = idx_language(path).unwrap_or(language);
            }
            found.push(ExternalSubtitle {
                // Forward slashes, like the library's video paths.
                path: path_str.replace('\\', "/"),
                language,
                title: format!("External {}{}", ext, origin),
            });
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_video_name_with_optional_language() {
        let video = "those.who.are.fine.2017-[yts.am]";
        assert_eq!(match_language(video, video).as_deref(), Some("und"));
        assert_eq!(match_language(&format!("{video}.en"), video).as_deref(), Some("en"));
        assert_eq!(match_language(&format!("{video}.eng"), video).as_deref(), Some("en"));
        // Not a language suffix / a different, longer-named video.
        assert_eq!(match_language(&format!("{video}_sub.eng"), video), None);
        assert_eq!(match_language(&format!("{video} part 2"), video), None);
        assert_eq!(match_language("another film 1999.en", video), None);
    }
}

use regex::Regex;
use once_cell::sync::Lazy;

use unicode_segmentation::UnicodeSegmentation;
use chrono::Datelike;
use std::path::Path;
use std::fs;

use crate::library_manager::file_manager::file_utils::remove_extension_and_path;

// REGEXs to look for to find end of Movie name
static YEAR_IN_BRACKETS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:[\(\[\{])(\d{4})(?:[\)\]\}])").unwrap());
static YEAR_IN_SPACES_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|\s)(\d{4})(?:$|\s)").unwrap());
static YEAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|\D)(\d{4})(?:$|[^pPxX\d])").unwrap()); // Exactly 4 digits, no trailing p/x (resolution)
// Or TV Show name
static SEASON_EPISODE_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)s\d{1,2}(?i)e\d+").unwrap());
static EPISODE_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"($|\s)(?i)e\d+($|\s)").unwrap());
static SEASON_EPISODE_X_FORMAT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d{1,2}x\d+").unwrap()); // Matches: 1x05, 12x3
static FOLDER_SEASON_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)season\s*\d+|(?i)s\d{1,2}").unwrap()); // Matches: Season 1, season 01, S01, s1
static DASH_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+-\s+0?\d{1,3}(?:\s+|$)").unwrap()); // Matches: " - 01 ", " - 1 ", " - 001"
static BRACKETED_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[(?:CM)?\d{1,3}\]").unwrap()); // Matches: [01], [CM01]
static EP_PREFIX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)ep\.?\s*\d+").unwrap()); // Matches: Ep. 01, Ep 01, ep.01
static SPECIAL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)special").unwrap()); // Matches: SPECIAL, Special, special

// If those fail, we try to find other patterns and take the left-most to find end of movie name
static FILENAME_NOISE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Resolutions
        Regex::new(r"\d{3,5}(?i)p").unwrap(),
        Regex::new(r"\d{3,5}(?i)x\d{3,5}").unwrap(),
        Regex::new(r"($|\s)4K($|\s)").unwrap(),
        Regex::new(r"($|\s)6K($|\s)").unwrap(),
        Regex::new(r"($|\s)8K($|\s)").unwrap(),
        Regex::new(r"($|\s)HQ($|\s)").unwrap(),
        Regex::new(r"($|\s)UHD($|\s)").unwrap(),
        Regex::new(r"($|\s)HDR($|\s)").unwrap(),
        Regex::new(r"($|\s)SDR($|\s)").unwrap(),
        Regex::new(r"($|\s)TrueHD($|\s)").unwrap(),

        // Codecs
        Regex::new(r"($|\s)x264($|\s)").unwrap(),
        Regex::new(r"($|\s)H 264($|\s)").unwrap(),
        Regex::new(r"($|\s)x265($|\s)").unwrap(),
        Regex::new(r"($|\s)H 265($|\s)").unwrap(),
        Regex::new(r"($|\s)AAC($|\s)").unwrap(),
        Regex::new(r"($|\s)AVC($|\s)").unwrap(),
        Regex::new(r"($|\s)DTS($|\s)").unwrap(),
        Regex::new(r"($|\s)HEVC($|\s)").unwrap(),
        Regex::new(r"($|\s)FLAC($|\s)").unwrap(),

        // Rip-Type
        Regex::new(r"($|\s)WEBDL($|\s)").unwrap(),
        Regex::new(r"($|\s)WEB dl($|\s)").unwrap(),
        Regex::new(r"($|\s)WEBRip($|\s)").unwrap(),
        Regex::new(r"($|\s)BluRay($|\s)").unwrap(),
        Regex::new(r"($|\s)BRRip($|\s)").unwrap(),
        Regex::new(r"($|\s)BDRemux($|\s)").unwrap(),
        Regex::new(r"($|\s)REMUX($|\s)").unwrap(),
        Regex::new(r"($|\s)DVD($|\s)").unwrap(),
        Regex::new(r"($|\s)HDTV($|\s)").unwrap(),
        Regex::new(r"($|\s)REMASTERED($|\s)").unwrap(),
        Regex::new(r"($|\s)UPSCALED($|\s)").unwrap(),
        Regex::new(r"($|\s)Criterion Collection($|\s)").unwrap(),
        Regex::new(r"($|\s)AMZN($|\s)").unwrap(),

        // Ripper
        Regex::new(r"($|\s)CINEFILE($|\s)").unwrap(),
        Regex::new(r"($|\s)FGT($|\s)").unwrap(),
        Regex::new(r"($|\s)USURY($|\s)").unwrap(),
        Regex::new(r"($|\s)YTS($|\s)").unwrap(),
        Regex::new(r"($|\s)SADPANDA($|\s)").unwrap(),
        Regex::new(r"($|\s)AMIABLE($|\s)").unwrap(),
        Regex::new(r"($|\s)SELENZEN($|\s)").unwrap(),
        Regex::new(r"($|\s)NAHOM($|\s)").unwrap(),
    ]
});

static CURRENT_YEAR: Lazy<i32> = Lazy::new(|| chrono::Utc::now().year());


// This function tries its best to get a title and year from a filename by looking for non-title patterns
// However, due to the impossible amount of naming conventions, the result might not actually be the title
// Worse, if the title includes a valid year*, it could be detected as the release year
// Therefore, after calling the function you only have an estimate and should
//  1) Strip your result
//  2) If you don't get good results using the returned name and year, try "name year" as title
//  3) If that fails try removing one word at a time from the end of "name", perhaps reducing the amount of noise
//  4) If that fails, do the same from the front
//  5) If that fails, do both, or random, or idk - good luck!
//
// *1888 - CURRENT_YEAR
pub(super) fn get_movie_title_and_year_from_filename(filename_or_path: &str) -> (String, Option<i32>)
{
    let mut filename = remove_extension_and_path(filename_or_path);
    let mut title_start_index = 0;
    let mut title_end_index = filename.len();
    let mut year: Option<i32> = Option::None;


    // First we try to find a single year in brackets e.g. "Persona (1966).mkv"
    let mut year_matches: Vec<(i32, usize, usize)> = regex_find_year(&YEAR_IN_BRACKETS_REGEX, &filename);

    if year_matches.is_empty()
    {
        // If that fails, we first remove anything in [] because if it's not the year it's probably crap (e.g. anime subbers)
        filename = remove_square_bracket_text(&filename);
        // Then we exchange special characters for spaces altogether
        filename = make_alphanumeric_with_spaces(&filename);
        title_end_index = filename.len();

        // Now we want to find a year seperated by spaces or at start/end of file e.g. "Caché 2005.mkv"
        year_matches = regex_find_year(&YEAR_IN_SPACES_REGEX, &filename);
    }

    // Last chance, maybe it's just after the title like "Nostalghia1983.mkv"
    if year_matches.is_empty()
    {
        year_matches = regex_find_year(&YEAR_REGEX, &filename);
    }

    // If we found something in the above, we hope the year is not at the start of the filename
    if !year_matches.is_empty()
    {
        // If we find 2 years - pick the last one, e.g. "2001 - A Space Odyssey 1968.mkv"
        year = Some(year_matches[year_matches.len()-1].0);

        let start_index = year_matches[year_matches.len()-1].1;
        if start_index > 0 {
            title_end_index = start_index;
            return (filename[title_start_index..title_end_index].to_owned(), year);
        }

        // If year is at beginning of file, at least we have the year, which is nice
        // We must still look for a second identifier, and only parse the title from the year to before that
        title_start_index = year_matches[year_matches.len()-1].2;
    }


    // If we can't find a year after the title, the filename's pretty shit.
    // But we can still try to catch typical filename noise, and extract the substring before the left-most
    for regex in FILENAME_NOISE_PATTERNS.iter() {
        if let Some(noise_match) = regex.find(&filename) {
            if noise_match.start() < title_end_index {
                title_end_index = noise_match.start();
            }
        }
    }


    // If the title_end_index hasn't changed, we can't do anything
    // Hopefully, the filename already equals the movie title
    //      otherwise, the calling function needs to guess by e.g. only taking the first x words
    (filename[title_start_index..title_end_index].to_owned(), year)

    //
    // A different approach could be to create a bool map of all characters in the filename
    // Then set each character to true, as soon as it is hit by one of the regex
    // After passing all regexes, see where the first semi-consecutive string of "false"-chars is
    // Return that as title
    //      This would take much more time, as we hope most filenames are caught by the year regex
    //      But we would be more resistant to even the stupidest naming conventions
    //      For now, no one that dense uses this software anyway :)
}

fn regex_find_year(regex: &Regex, filename: &str) -> Vec<(i32, usize, usize)>
{
    regex.captures_iter(filename)
        .filter_map(|cap| {
            if let Some(year_match) = cap.get(1) {
                let year_str = year_match.as_str();
                if let Ok(year) = year_str.parse::<i32>() {
                    if 1888 <= year && year <= *CURRENT_YEAR {
                        // Get the full match's start and end positions (index 0)
                        let full_match = cap.get(0).unwrap();
                        return Some((year, full_match.start(), full_match.end()));
                    }
                }
                else {
                    println!("Could not parse year from '{}'", year_str);
                }
            }
            None
        })
        .collect()
}


// Input:   I.Do.Not.Care.If.We.Go.Down.In.History.As.Barbarians.2018.1080p.BluRay.x264.AAC5.1-[YTS.MX]
// Output:  I Do Not Care If We Go Down In History As Barbarians 2018 1080p BluRay x264 AAC5 1 YTS MX
fn make_alphanumeric_with_spaces(filename: &str) -> String
{
    let mut result = String::with_capacity(filename.len());
    let mut prev_was_alphanumeric = false;
    let mut has_content = false;

    for grapheme in filename.graphemes(true)
    {
        // Check if the grapheme contains any alphanumeric character
        let is_valid = grapheme.chars().any(|c|
        {
            c.is_alphanumeric() ||
            c.is_alphabetic() // Additional check for non-ASCII strings
        });

        if is_valid {
            // If we previously saw whitespace and already have content,
            // add exactly one space before the new valid character
            if !prev_was_alphanumeric && has_content {
                result.push(' ');
            }
            result.push_str(grapheme);
            prev_was_alphanumeric = true;
            has_content = true;
        }
        else {
            prev_was_alphanumeric = false;
        }
    }

    result
}

// Input:   [Commie] Neon Genesis Evangelion - The End of Evangelion [BD 1080p AAC] [9EF369E6]
// Output:  Neon Genesis Evangelion - The End of Evangelion
fn remove_square_bracket_text(filename: &str) -> String
{
    let mut result = String::new();
    let mut inside_brackets = false;

    for c in filename.chars() {
        match c {
            '[' => inside_brackets = true,
            ']' => inside_brackets = false,
            _ if !inside_brackets => result.push(c),
            _ => {}
        }
    }

    // Trim any extra whitespace that might remain
    result.trim().to_string()
}


// Checks if a file is likely a TV episode based on filename and folder context
// Returns true if high confidence it's a TV episode, false otherwise
pub(super) fn is_tv_episode(filepath: &str) -> bool
{
    let filename = remove_extension_and_path(filepath);

    // Step 1: Check high confidence patterns
    // S01E05 format (case-insensitive)
    if SEASON_EPISODE_NUMBER_REGEX.is_match(&filename) {
        return true;
    }

    // 1x05 format
    if SEASON_EPISODE_X_FORMAT_REGEX.is_match(&filename) {
        return true;
    }

    // Step 2: Check weak patterns + folder context
    if EPISODE_NUMBER_REGEX.is_match(&filename) {
        // Extract folder path from filepath
        let path = Path::new(filepath);
        if let Some(parent) = path.parent() {
            // Check folder for similar files with different episode numbers
            if verify_with_folder_context(&filename, parent) {
                return true;
            }

            // Check if parent folder has season indicators
            if let Some(folder_name) = parent.file_name() {
                if let Some(folder_str) = folder_name.to_str() {
                    if FOLDER_SEASON_REGEX.is_match(folder_str) {
                        return true;
                    }
                }
            }
        }
    }

    // Step 3: Check parent folder for season indicators even without episode marker
    // This catches cases like: /Show Name/Season 1/Episode Title.mkv
    let path = Path::new(filepath);
    if let Some(parent) = path.parent() {
        if let Some(folder_name) = parent.file_name() {
            if let Some(folder_str) = folder_name.to_str() {
                if FOLDER_SEASON_REGEX.is_match(folder_str) {
                    // Also verify this folder has multiple video files (likely episodes)
                    if count_video_files_in_folder(parent) > 1 {
                        return true;
                    }
                }
            }
        }
    }

    // Step 4: Ep. prefix pattern (e.g., "Ep. 01", "Episode 5")
    if EP_PREFIX_REGEX.is_match(&filename) {
        return true;
    }

    // Step 5: Dash-number pattern with folder context (requires 3+ matching files)
    // e.g., "Show Name - 01", "Show Name - 001"
    if DASH_NUMBER_REGEX.is_match(&filename) {
        let path = Path::new(filepath);
        if let Some(parent) = path.parent() {
            if verify_dash_number_with_folder_context(&filename, parent, 3) {
                return true;
            }
        }
    }

    // Step 6: Bracketed number pattern with folder context
    // e.g., "[01]", "[CM01]"
    if BRACKETED_NUMBER_REGEX.is_match(&filename) {
        let path = Path::new(filepath);
        if let Some(parent) = path.parent() {
            if verify_bracketed_number_with_folder_context(&filename, parent) {
                return true;
            }
        }
    }

    // Step 7: "Special" keyword with folder context
    // e.g., "Show Name Special", "Show Name - Special Episode"
    if SPECIAL_REGEX.is_match(&filename) {
        let path = Path::new(filepath);
        if let Some(parent) = path.parent() {
            if verify_special_with_folder_context(&filename, parent) {
                return true;
            }
        }
    }

    false
}

// Verifies TV episode detection by checking for similar filenames in the same folder
// with different episode numbers
fn verify_with_folder_context(filename: &str, folder_path: &Path) -> bool
{
    // Extract base title by removing episode number pattern
    let base_title = EPISODE_NUMBER_REGEX.replace(filename, " ").to_string();
    let base_title = base_title.trim();

    // Read files in the same directory
    let Ok(entries) = fs::read_dir(folder_path) else {
        return false;
    };

    let mut matching_files = 0;

    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
                if let Some(entry_filename) = entry.file_name().to_str() {
                    let entry_name = remove_extension_and_path(entry_filename);

                    // Skip the current file itself
                    if entry_name == filename {
                        continue;
                    }

                    // Check if this file also has an episode pattern
                    if EPISODE_NUMBER_REGEX.is_match(&entry_name) {
                        // Remove episode number and compare base titles
                        let entry_base = EPISODE_NUMBER_REGEX.replace(&entry_name, " ").to_string();
                        let entry_base = entry_base.trim();

                        // If base titles are similar, it's likely the same show
                        if similarity(base_title, entry_base) > 0.8 {
                            matching_files += 1;
                            // Found at least one similar file with different episode
                            if matching_files >= 1 {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

// Counts video files in a folder (used to verify season folders have multiple episodes)
fn count_video_files_in_folder(folder_path: &Path) -> usize
{
    let Ok(entries) = fs::read_dir(folder_path) else {
        return 0;
    };

    entries.flatten()
        .filter(|entry| {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(ext) = entry.path().extension() {
                        let ext_str = ext.to_str().unwrap_or("").to_lowercase();
                        return matches!(ext_str.as_str(), "mkv" | "mp4" | "avi" | "mov" | "m2ts");
                    }
                }
            }
            false
        })
        .count()
}

// Simple string similarity comparison (normalized Levenshtein distance)
// Returns value between 0.0 (completely different) and 1.0 (identical)
fn similarity(s1: &str, s2: &str) -> f32
{
    if s1 == s2 {
        return 1.0;
    }

    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let distance = levenshtein_distance(s1, s2);
    let max_len = len1.max(len2);

    1.0 - (distance as f32 / max_len as f32)
}

// Calculates Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize
{
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

// Verifies dash-number pattern by requiring minimum number of similar files
// This helps avoid false positives with movie series (Fast & Furious 1-8, etc.)
fn verify_dash_number_with_folder_context(filename: &str, folder_path: &Path, min_matches: usize) -> bool
{
    // Extract base title by removing dash-number pattern
    let base_title = DASH_NUMBER_REGEX.replace(filename, " ").to_string();
    let base_title = base_title.trim();

    let Ok(entries) = fs::read_dir(folder_path) else {
        return false;
    };

    let mut matching_files = 0;

    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
                if let Some(entry_filename) = entry.file_name().to_str() {
                    let entry_name = remove_extension_and_path(entry_filename);

                    // Skip the current file itself
                    if entry_name == filename {
                        continue;
                    }

                    // Check if this file also has dash-number pattern
                    if DASH_NUMBER_REGEX.is_match(&entry_name) {
                        // Remove dash-number and compare base titles
                        let entry_base = DASH_NUMBER_REGEX.replace(&entry_name, " ").to_string();
                        let entry_base = entry_base.trim();

                        // If base titles are similar, it's likely the same show
                        if similarity(base_title, entry_base) > 0.8 {
                            matching_files += 1;
                            // Need min_matches to confirm it's a TV show
                            if matching_files >= min_matches {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

// Verifies bracketed number pattern with folder context
fn verify_bracketed_number_with_folder_context(filename: &str, folder_path: &Path) -> bool
{
    // Extract base title by removing bracketed number pattern
    let base_title = BRACKETED_NUMBER_REGEX.replace(filename, " ").to_string();
    let base_title = base_title.trim();

    let Ok(entries) = fs::read_dir(folder_path) else {
        return false;
    };

    let mut matching_files = 0;

    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
                if let Some(entry_filename) = entry.file_name().to_str() {
                    let entry_name = remove_extension_and_path(entry_filename);

                    // Skip the current file itself
                    if entry_name == filename {
                        continue;
                    }

                    // Check if this file also has bracketed number pattern
                    if BRACKETED_NUMBER_REGEX.is_match(&entry_name) {
                        // Remove bracketed number and compare base titles
                        let entry_base = BRACKETED_NUMBER_REGEX.replace(&entry_name, " ").to_string();
                        let entry_base = entry_base.trim();

                        // If base titles are similar, it's likely the same show
                        if similarity(base_title, entry_base) > 0.8 {
                            matching_files += 1;
                            // Found at least one similar file with different episode
                            if matching_files >= 1 {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

// Verifies "special" keyword with folder context
// If other files in the same folder have similar names (without "special" or episode numbers),
// then this file with "special" is likely also a TV episode
fn verify_special_with_folder_context(filename: &str, folder_path: &Path) -> bool
{
    // Extract base title by removing "special" keyword and normalizing spaces
    let mut base_title = SPECIAL_REGEX.replace(filename, " ").to_string();
    base_title = normalize_spaces(&base_title);

    println!("[DEBUG] Special file base_title: '{}'", base_title);

    let Ok(entries) = fs::read_dir(folder_path) else {
        return false;
    };

    let mut matching_files = 0;

    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
                if let Some(entry_filename) = entry.file_name().to_str() {
                    let entry_name = remove_extension_and_path(entry_filename);

                    // Skip the current file itself
                    if entry_name == filename {
                        continue;
                    }

                    // For comparison, remove common episode patterns from the other file
                    let mut entry_base = entry_name.clone();

                    // Remove various episode patterns to get base show name
                    if SEASON_EPISODE_NUMBER_REGEX.is_match(&entry_base) {
                        entry_base = SEASON_EPISODE_NUMBER_REGEX.replace_all(&entry_base, " ").to_string();
                    }
                    if EPISODE_NUMBER_REGEX.is_match(&entry_base) {
                        entry_base = EPISODE_NUMBER_REGEX.replace_all(&entry_base, " ").to_string();
                    }
                    if SEASON_EPISODE_X_FORMAT_REGEX.is_match(&entry_base) {
                        entry_base = SEASON_EPISODE_X_FORMAT_REGEX.replace_all(&entry_base, " ").to_string();
                    }
                    // Also remove standalone season markers (S01, S1, etc.)
                    if FOLDER_SEASON_REGEX.is_match(&entry_base) {
                        entry_base = FOLDER_SEASON_REGEX.replace_all(&entry_base, " ").to_string();
                    }
                    if DASH_NUMBER_REGEX.is_match(&entry_base) {
                        entry_base = DASH_NUMBER_REGEX.replace_all(&entry_base, " ").to_string();
                    }
                    if BRACKETED_NUMBER_REGEX.is_match(&entry_base) {
                        entry_base = BRACKETED_NUMBER_REGEX.replace_all(&entry_base, " ").to_string();
                    }
                    if EP_PREFIX_REGEX.is_match(&entry_base) {
                        entry_base = EP_PREFIX_REGEX.replace_all(&entry_base, " ").to_string();
                    }

                    // Normalize spaces after all replacements
                    entry_base = normalize_spaces(&entry_base);

                    // Calculate similarity and debug
                    let sim_score = similarity(&base_title, &entry_base);
                    println!("[DEBUG] Comparing '{}' vs '{}' = {:.3}", base_title, entry_base, sim_score);

                    // If base titles are similar, it's likely the same show
                    if sim_score > 0.8 {
                        matching_files += 1;
                        // Found at least one similar file - confirms this is a TV episode
                        if matching_files >= 1 {
                            println!("[DEBUG] Match found! Confirmed as TV episode.");
                            return true;
                        }
                    }
                }
            }
        }
    }

    println!("[DEBUG] No matches found for special file.");
    false
}

// Helper function to normalize multiple spaces into single spaces and trim
fn normalize_spaces(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// Extracts TV show name, season number, and episode number from filename
// Returns: (show_name, season_number, episode_number)
// Returns None for season/episode if not found
pub(super) fn get_tv_show_info_from_filename(filepath: &str) -> (String, Option<i32>, Option<i32>)
{
    let filename = remove_extension_and_path(filepath);
    let mut show_name = filename.clone();
    let mut season: Option<i32> = None;
    let mut episode: Option<i32> = None;

    // Try to find S01E05 format (highest confidence)
    if let Some(capture) = SEASON_EPISODE_NUMBER_REGEX.captures(&filename) {
        if let Some(matched) = capture.get(0) {
            let matched_str = matched.as_str();

            // Extract season and episode numbers from the match
            // Format is like "S01E05" or "s2e10"
            let upper = matched_str.to_uppercase();
            if let Some(s_pos) = upper.find('S') {
                if let Some(e_pos) = upper.find('E') {
                    // Extract season number between 'S' and 'E'
                    let season_str = &upper[s_pos + 1..e_pos];
                    season = season_str.parse::<i32>().ok();

                    // Extract episode number after 'E'
                    let episode_str = &upper[e_pos + 1..];
                    episode = episode_str.parse::<i32>().ok();
                }
            }

            // Extract show name (everything before the match)
            show_name = filename[..matched.start()].to_string();
        }
    }
    // Try 1x05 format
    else if let Some(capture) = SEASON_EPISODE_X_FORMAT_REGEX.captures(&filename) {
        if let Some(matched) = capture.get(0) {
            let matched_str = matched.as_str();

            // Split on 'x' to get season and episode
            let parts: Vec<&str> = matched_str.split('x').collect();
            if parts.len() == 2 {
                season = parts[0].parse::<i32>().ok();
                episode = parts[1].parse::<i32>().ok();
            }

            show_name = filename[..matched.start()].to_string();
        }
    }
    // Try E05 format with folder context for season
    else if let Some(capture) = EPISODE_NUMBER_REGEX.captures(&filename) {
        if let Some(matched) = capture.get(0) {
            let matched_str = matched.as_str().trim();

            // Extract episode number (after 'E')
            let upper = matched_str.to_uppercase();
            if let Some(e_pos) = upper.find('E') {
                let episode_str = &upper[e_pos + 1..].trim();
                episode = episode_str.parse::<i32>().ok();
            }

            show_name = filename[..matched.start()].to_string();

            // Try to extract season from folder path
            let path = Path::new(filepath);
            if let Some(parent) = path.parent() {
                if let Some(folder_name) = parent.file_name() {
                    if let Some(folder_str) = folder_name.to_str() {
                        season = extract_season_from_folder(folder_str);
                    }
                }
            }
        }
    }
    // Try dash-number format (e.g., "Show Name - 01")
    else if let Some(capture) = DASH_NUMBER_REGEX.captures(&filename) {
        if let Some(matched) = capture.get(0) {
            let matched_str = matched.as_str().trim();

            // Extract episode number from the dash pattern
            let number_str = matched_str.trim_start_matches('-').trim();
            episode = number_str.parse::<i32>().ok();

            show_name = filename[..matched.start()].to_string();

            // Try to extract season from folder path
            let path = Path::new(filepath);
            if let Some(parent) = path.parent() {
                if let Some(folder_name) = parent.file_name() {
                    if let Some(folder_str) = folder_name.to_str() {
                        season = extract_season_from_folder(folder_str);
                    }
                }
            }
        }
    }
    // Try bracketed number format (e.g., "[01]")
    else if let Some(capture) = BRACKETED_NUMBER_REGEX.captures(&filename) {
        if let Some(matched) = capture.get(0) {
            let matched_str = matched.as_str();

            // Extract number from brackets, removing any prefix like "CM"
            let number_str = matched_str.trim_matches(|c| c == '[' || c == ']');
            let number_str = number_str.trim_start_matches(|c: char| !c.is_numeric());
            episode = number_str.parse::<i32>().ok();

            show_name = filename[..matched.start()].to_string();

            // Try to extract season from folder path
            let path = Path::new(filepath);
            if let Some(parent) = path.parent() {
                if let Some(folder_name) = parent.file_name() {
                    if let Some(folder_str) = folder_name.to_str() {
                        season = extract_season_from_folder(folder_str);
                    }
                }
            }
        }
    }
    // Try Ep. format (e.g., "Ep. 01", "Episode 5")
    else if let Some(capture) = EP_PREFIX_REGEX.captures(&filename) {
        if let Some(matched) = capture.get(0) {
            let matched_str = matched.as_str();

            // Extract episode number from the match
            let number_str: String = matched_str.chars()
                .filter(|c| c.is_numeric())
                .collect();
            episode = number_str.parse::<i32>().ok();

            show_name = filename[..matched.start()].to_string();

            // Try to extract season from folder path
            let path = Path::new(filepath);
            if let Some(parent) = path.parent() {
                if let Some(folder_name) = parent.file_name() {
                    if let Some(folder_str) = folder_name.to_str() {
                        season = extract_season_from_folder(folder_str);
                    }
                }
            }
        }
    }

    // Clean up show name
    show_name = show_name.trim().to_string();
    // Remove square brackets content from show name
    show_name = remove_square_bracket_text(&show_name);
    // Convert special chars to spaces
    show_name = make_alphanumeric_with_spaces(&show_name);
    show_name = show_name.trim().to_string();

    (show_name, season, episode)
}

// Helper function to extract season number from folder name
// Handles formats like: "Season 1", "Season 01", "S01", "s1"
fn extract_season_from_folder(folder_name: &str) -> Option<i32>
{
    if let Some(capture) = FOLDER_SEASON_REGEX.captures(folder_name) {
        if let Some(matched) = capture.get(0) {
            let matched_str = matched.as_str();

            // Extract numbers from the match
            let number_str: String = matched_str.chars()
                .filter(|c| c.is_numeric())
                .collect();

            return number_str.parse::<i32>().ok();
        }
    }
    None
}
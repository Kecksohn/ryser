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
static SEASON_EPISODE_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)s(\d{1,2})[ ._-]?e(\d+)").unwrap()); // Matches: S01E05, s2e10, S01 E05, S01.E05
static EPISODE_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"($|\s)(?i)e\d+($|\s)").unwrap());
static SEASON_EPISODE_X_FORMAT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d{1,2}x\d+").unwrap()); // Matches: 1x05, 12x3
static FOLDER_SEASON_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)season\s*\d+|(?i)s\d{1,2}").unwrap()); // Matches: Season 1, season 01, S01, s1
static DASH_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+-\s+0?\d{1,3}(?:\s+|$)").unwrap()); // Matches: " - 01 ", " - 1 ", " - 001"
static BRACKETED_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[(?:CM)?\d{1,3}\]").unwrap()); // Matches: [01], [CM01]
static EP_PREFIX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(?:^|[^a-z0-9])ep\.?\s*\d+").unwrap()); // Matches: Ep. 01, Ep 01, ep.01 - but not the "ep 1946" in "Sleep 1946"
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

    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

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
    // Lengths in characters, the matrix below is filled per character
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

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
// Returns None for season/episode if no episode number is found
// With an episode number but no season in filename or folder, the season is 1
pub(super) fn get_tv_show_info_from_filename(filepath: &str) -> (String, Option<i32>, Option<i32>)
{
    let filename = remove_extension_and_path(filepath);
    let mut show_name = filename.clone();
    let mut season: Option<i32> = None;
    let mut episode: Option<i32> = None;

    // Try to find S01E05 format (highest confidence)
    if let Some(capture) = SEASON_EPISODE_NUMBER_REGEX.captures(&filename) {
        // Format is like "S01E05", "s2e10" or "S01 E05"
        season = capture[1].parse::<i32>().ok();
        episode = capture[2].parse::<i32>().ok();

        // Extract show name (everything before the match)
        show_name = filename[..capture.get(0).unwrap().start()].to_string();
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

    // An episode number without any season marker: most likely a show with a single season
    if episode.is_some() && season.is_none() {
        season = Some(1);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::library_manager::file_manager::is_video_file;

    // A folder of empty files below the system temp dir, removed on drop.
    // `folder` is the name the parser sees as the parent folder.
    struct TestFolder {
        root: PathBuf,
        folder: PathBuf,
    }

    impl TestFolder {
        fn new(unique_name: &str, folder: &str, files: &[&str]) -> Self
        {
            let root = std::env::temp_dir()
                .join(format!("ryser_parser_{}_{}", std::process::id(), unique_name));
            let folder = root.join(folder);
            fs::create_dir_all(&folder).unwrap();
            for file in files {
                fs::write(folder.join(file), "").unwrap();
            }
            TestFolder { root, folder }
        }

        fn path(&self, file: &str) -> String
        {
            self.folder.join(file).to_str().unwrap().to_owned()
        }
    }

    impl Drop for TestFolder {
        fn drop(&mut self)
        {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn tv_info(filepath: &str) -> (String, Option<i32>, Option<i32>)
    {
        get_tv_show_info_from_filename(filepath)
    }

    // The title is trimmed here because the parser leaves that to the caller.
    fn movie(filepath: &str) -> (String, Option<i32>)
    {
        let (title, year) = get_movie_title_and_year_from_filename(filepath);
        (title.trim().to_owned(), year)
    }

    #[test]
    fn tv_show_info_from_filename()
    {
        let cases: &[(&str, &str, Option<i32>, Option<i32>)] = &[
            // S01E02
            ("Breaking.Bad.S01E02.720p.mkv", "Breaking Bad", Some(1), Some(2)),
            ("the.wire.s2e10.mkv", "the wire", Some(2), Some(10)),
            ("Brass Eye S01 E02 x264 DVDRip - Drugs.mkv", "Brass Eye", Some(1), Some(2)),
            ("Show.S02.E03.mkv", "Show", Some(2), Some(3)),
            // 1x05
            ("Seinfeld 1x05 The Stock Tip.avi", "Seinfeld", Some(1), Some(5)),
            ("Show 12x03.mkv", "Show", Some(12), Some(3)),
            // E05, season from the folder
            ("Show/Season 2/Show E05.mkv", "Show", Some(2), Some(5)),
            ("Show/S03/Show e7 Title.mkv", "Show", Some(3), Some(7)),
            // No season in filename or folder: season 1
            ("Show/Show E05.mkv", "Show", Some(1), Some(5)),
            // " - 01 "
            ("Monster - 12 - Title.mkv", "Monster", Some(1), Some(12)),
            ("Monster - 001.mkv", "Monster", Some(1), Some(1)),
            ("Monster/Season 2/Monster - 01.mkv", "Monster", Some(2), Some(1)),
            // [01]
            ("[Grp] Show [01].mkv", "Show", Some(1), Some(1)),
            ("[Grp] Show [CM02].mkv", "Show", Some(1), Some(2)),
            // Ep. 01
            ("Show Ep. 01.mkv", "Show", Some(1), Some(1)),
            ("Show ep03.mkv", "Show", Some(1), Some(3)),
            ("Show/Season 4/Show Ep 7.mkv", "Show", Some(4), Some(7)),
            // No pattern
            ("Some Movie.mkv", "Some Movie", None, None),
        ];

        for (filepath, name, season, episode) in cases {
            assert_eq!(tv_info(filepath), (name.to_string(), *season, *episode), "{}", filepath);
        }
    }

    #[test]
    fn similarity_counts_characters_not_bytes()
    {
        assert_eq!(levenshtein_distance("Städte", "Stadte"), 1);
        assert_eq!(levenshtein_distance("Caché", "Caché"), 0);
        assert_eq!(levenshtein_distance("ä", ""), 1);
        assert!(similarity("Star Wars", "Aufzeichnungen zu Kleidern und Städten 1989") < 0.5);
        // One of six characters differs
        assert!((similarity("Städte", "Stadte") - 5.0 / 6.0).abs() < 0.001);
    }

    #[test]
    fn is_tv_episode_special_ignores_unrelated_non_ascii_sibling()
    {
        let films = TestFolder::new("special_non_ascii", "films", &[
            "Nothing Special 2022.mkv", "Aufzeichnungen zu Kleidern und Städten 1989.mkv",
        ]);
        assert!(!is_tv_episode(&films.path("Nothing Special 2022.mkv")));
    }

    // The ignored tests below fail because the parser is wrong, not the test.

    #[test]
    #[ignore = "parser bug: SEASON_EPISODE_X_FORMAT_REGEX matches inside 1920x1080, gives season 20 episode 1080"]
    fn tv_show_info_resolution_is_not_season_x_episode()
    {
        assert_eq!(tv_info("Show - 01 [1920x1080].mkv"), ("Show".to_owned(), None, Some(1)));
    }

    #[test]
    #[ignore = "parser bug: EP_PREFIX_REGEX does not match 'Episode 5', although its comments say so"]
    fn tv_show_info_episode_spelled_out()
    {
        assert_eq!(tv_info("Show Episode 5.mkv"), ("Show".to_owned(), None, Some(5)));
    }

    #[test]
    fn is_tv_episode_by_filename_alone()
    {
        for filepath in [
            "shows/Breaking.Bad.S01E02.720p.mkv",
            "shows/the.wire.s2e10.mkv",
            "shows/Seinfeld 1x05 The Stock Tip.avi",
            "shows/Show Ep. 01.mkv",
        ] {
            assert!(is_tv_episode(filepath), "{}", filepath);
        }
    }

    #[test]
    fn is_tv_episode_false_for_movies()
    {
        for filepath in [
            "movies/Persona (1966).mkv",
            "movies/Se7en.1995.1080p.BluRay.x264.mkv",
            "movies/2001 - A Space Odyssey 1968.mkv",
            "movies/Ocean's Eleven (2001).mkv",
            "movies/S1m0ne (2002).mkv",
            "movies/Apollo 13 (1995).mkv",
            "movies/District 9.mkv",
            "movies/E.T. the Extra-Terrestrial (1982).mkv",
            "movies/[Commie] Neon Genesis Evangelion - The End of Evangelion [BD 1080p AAC] [9EF369E6].mkv",
        ] {
            assert!(!is_tv_episode(filepath), "{}", filepath);
        }
    }

    #[test]
    fn is_tv_episode_e_number_needs_sibling_or_season_folder()
    {
        let siblings = TestFolder::new("e_siblings", "Show", &["Show E01.mkv", "Show E02.mkv"]);
        assert!(is_tv_episode(&siblings.path("Show E01.mkv")));

        let season = TestFolder::new("e_season", "Season 1", &["Show E05.mkv"]);
        assert!(is_tv_episode(&season.path("Show E05.mkv")));

        let alone = TestFolder::new("e_alone", "Show", &["Show E05.mkv"]);
        assert!(!is_tv_episode(&alone.path("Show E05.mkv")));
    }

    #[test]
    fn is_tv_episode_season_folder_needs_several_videos()
    {
        let several = TestFolder::new("season_several", "Season 1", &["Pilot.mkv", "The Second One.mkv"]);
        assert!(is_tv_episode(&several.path("Pilot.mkv")));

        let single = TestFolder::new("season_single", "Season 1", &["Pilot.mkv", "Pilot.srt"]);
        assert!(!is_tv_episode(&single.path("Pilot.mkv")));
    }

    #[test]
    fn is_tv_episode_dash_number_needs_three_siblings()
    {
        let show = TestFolder::new("dash_show", "Monster", &[
            "Monster - 01.mkv", "Monster - 02.mkv", "Monster - 03.mkv", "Monster - 04.mkv",
        ]);
        assert!(is_tv_episode(&show.path("Monster - 01.mkv")));

        let trilogy = TestFolder::new("dash_trilogy", "Mad Max", &[
            "Mad Max - 1.mkv", "Mad Max - 2.mkv", "Mad Max - 3.mkv",
        ]);
        assert!(!is_tv_episode(&trilogy.path("Mad Max - 1.mkv")));
    }

    #[test]
    fn is_tv_episode_bracketed_number_needs_sibling()
    {
        let show = TestFolder::new("bracket_show", "Show", &["[Grp] Show [01].mkv", "[Grp] Show [02].mkv"]);
        assert!(is_tv_episode(&show.path("[Grp] Show [01].mkv")));

        let alone = TestFolder::new("bracket_alone", "Show", &["[Grp] Show [01].mkv"]);
        assert!(!is_tv_episode(&alone.path("[Grp] Show [01].mkv")));
    }

    #[test]
    fn is_tv_episode_special_needs_sibling()
    {
        let show = TestFolder::new("special_show", "Show", &["Show Special.mkv", "Show S01E01.mkv"]);
        assert!(is_tv_episode(&show.path("Show Special.mkv")));

        let alone = TestFolder::new("special_alone", "Show", &["Show Special.mkv"]);
        assert!(!is_tv_episode(&alone.path("Show Special.mkv")));
    }

    #[test]
    #[ignore = "parser bug: SEASON_EPISODE_X_FORMAT_REGEX matches inside 1920x1080"]
    fn is_tv_episode_false_for_resolution()
    {
        assert!(!is_tv_episode("movies/Movie.1920x1080.mkv"));
    }

    #[test]
    fn is_tv_episode_false_for_title_ending_in_ep_before_year()
    {
        assert!(!is_tv_episode("movies/The Big Sleep 1946.mkv"));
        assert!(!is_tv_episode("movies/Black.Sheep.2006.mkv"));
    }

    #[test]
    #[ignore = "parser bug: FOLDER_SEASON_REGEX matches any s<digit>, so a folder 'films2' with two videos counts as a season"]
    fn is_tv_episode_false_for_movies_in_folder_ending_in_s_digit()
    {
        let films = TestFolder::new("folder_s_digit", "films2", &["Alien (1979).mkv", "Heat (1995).mkv"]);
        assert!(!is_tv_episode(&films.path("Alien (1979).mkv")));
    }

    #[test]
    fn movie_title_and_year_from_filename()
    {
        let cases: &[(&str, &str, Option<i32>)] = &[
            ("Persona (1966).mkv", "Persona", Some(1966)),
            ("C:/films/Persona (1966).mkv", "Persona", Some(1966)),
            ("Ran [1985].mkv", "Ran", Some(1985)),
            ("Caché 2005.mkv", "Caché", Some(2005)),
            ("2001 - A Space Odyssey 1968.mkv", "2001 A Space Odyssey", Some(1968)),
            ("1917 (2019).mkv", "1917", Some(2019)),
            ("Blade Runner 2049 (2017).mkv", "Blade Runner 2049", Some(2017)),
            (
                "I.Do.Not.Care.If.We.Go.Down.In.History.As.Barbarians.2018.1080p.BluRay.x264.AAC5.1-[YTS.MX].mkv",
                "I Do Not Care If We Go Down In History As Barbarians",
                Some(2018),
            ),
            (
                "[Commie] Neon Genesis Evangelion - The End of Evangelion [BD 1080p AAC] [9EF369E6].mkv",
                "Neon Genesis Evangelion The End of Evangelion",
                None,
            ),
            // Resolutions are not years
            ("Movie.1920x1080.mkv", "Movie", None),
            ("Movie 2160p.mkv", "Movie", None),
            // No year: cut at the first noise pattern
            ("Stalker.720p.BluRay.x264.mkv", "Stalker", None),
            ("Stalker.mkv", "Stalker", None),
        ];

        for (filepath, title, year) in cases {
            assert_eq!(movie(filepath), (title.to_string(), *year), "{}", filepath);
        }
    }

    #[test]
    #[ignore = "parser bug: YEAR_REGEX match starts one character before the year, title loses its last letter"]
    fn movie_year_directly_after_title()
    {
        assert_eq!(movie("Nostalghia1983.mkv"), ("Nostalghia".to_owned(), Some(1983)));
    }

    #[test]
    #[ignore = "parser bug: the first year's match consumes the separator, so the second year is not found"]
    fn movie_two_adjacent_years_picks_the_last()
    {
        assert_eq!(movie("Death.Race.2000.1975.mkv"), ("Death Race 2000".to_owned(), Some(1975)));
    }

    #[test]
    #[ignore = "parser bug: a title that is itself a valid year comes back empty"]
    fn movie_title_that_is_a_year()
    {
        assert_eq!(movie("1917.1080p.BluRay.mkv").0, "1917");
    }

    // Corpus tests: run the parser over the real media folders of this machine.
    // Ignored because they need those folders. They only list directories.
    // Run by name:
    //   cargo test --manifest-path src-tauri/Cargo.toml corpus_ -- --ignored
    // Each writes a report to .claude/scratch/ to diff before and after a parser change.
    const MOVIE_FOLDER: &str = "F:/mov";
    const TV_FOLDER: &str = "T:/tv";

    fn video_files_below(folder: &Path) -> Vec<PathBuf>
    {
        let mut found = Vec::new();
        let mut pending = vec![folder.to_path_buf()];
        while let Some(dir) = pending.pop() {
            for entry in fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                } else if is_video_file(&path) {
                    found.push(path);
                }
            }
        }
        found.sort();
        found
    }

    fn write_report(name: &str, lines: &[String])
    {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.claude/scratch");
        fs::create_dir_all(&scratch).unwrap();
        fs::write(scratch.join(name), lines.join("\n") + "\n").unwrap();
    }

    #[test]
    #[ignore = "needs the movie folder of this machine"]
    fn corpus_movies_are_not_tv_episodes()
    {
        let folder = Path::new(MOVIE_FOLDER);
        if !folder.is_dir() {
            println!("skipped: {} not found", MOVIE_FOLDER);
            return;
        }

        let mut report = Vec::new();
        let mut seen_as_tv = Vec::new();
        for path in video_files_below(folder) {
            let filepath = path.to_string_lossy().into_owned();
            let is_tv = is_tv_episode(&filepath);
            let (title, year) = movie(&filepath);
            report.push(format!("{}\t{}\t{}\t{:?}", filepath, if is_tv { "TV" } else { "movie" }, title, year));
            if is_tv {
                seen_as_tv.push(filepath);
            }
        }
        write_report("parser-corpus-movies.txt", &report);

        assert!(
            seen_as_tv.is_empty(),
            "{} of {} movies seen as TV episode:\n{}",
            seen_as_tv.len(), report.len(), seen_as_tv.join("\n")
        );
    }

    // Bonus material is not expected to parse as an episode. The show folder itself is not
    // checked, there is a show called "Extras".
    fn is_in_extras_folder(path: &Path, root: &Path) -> bool
    {
        let folders: Vec<String> = path.parent().unwrap().strip_prefix(root).unwrap()
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
            .collect();
        folders.iter().skip(1).any(|f| {
            matches!(f.as_str(), "extras" | "extra" | "bonus" | "featurettes" | "deleted scenes")
        })
    }

    // The app needs season and episode to fetch an episode, so a missing one counts as a failure.
    #[test]
    #[ignore = "needs the TV folder of this machine"]
    fn corpus_tv_files_are_tv_episodes()
    {
        let folder = Path::new(TV_FOLDER);
        if !folder.is_dir() {
            println!("skipped: {} not found", TV_FOLDER);
            return;
        }

        let mut report = Vec::new();
        let mut not_ok = 0;
        for path in video_files_below(folder) {
            let filepath = path.to_string_lossy().into_owned();
            let (show, season, episode) = tv_info(&filepath);
            let status = if is_in_extras_folder(&path, folder) {
                "extra"
            } else if !is_tv_episode(&filepath) {
                "NOT_TV"
            } else if episode.is_none() {
                "NO_EPISODE"
            } else if season.is_none() {
                "NO_SEASON"
            } else {
                "ok"
            };
            if status != "ok" && status != "extra" {
                not_ok += 1;
            }
            report.push(format!("{}\t{}\t{}\t{:?}\t{:?}", filepath, status, show, season, episode));
        }
        write_report("parser-corpus-tv.txt", &report);

        assert!(not_ok == 0, "{} of {} TV files not parsed, see parser-corpus-tv.txt", not_ok, report.len());
    }
}
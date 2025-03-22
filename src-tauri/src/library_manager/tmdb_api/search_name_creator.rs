use regex::Regex;
use once_cell::sync::Lazy;

use unicode_segmentation::UnicodeSegmentation;
use chrono::Datelike;

use crate::library_manager::file_utils::remove_extension_and_path;

// REGEXs to look for to find end of Movie name
static YEAR_IN_BRACKETS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\(\[\{]\d{4}[\)\]\}]").unwrap());
static YEAR_IN_SPACES_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"($|\s)\d{4}($|\s)").unwrap()); // also captures start/end of string
static YEAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?<!\d)\d{4}(?![\dPpXx])").unwrap()); // exactly 4 digits, no trailing p/x (resolution)
// Or TV Show name
static SEASON_EPISODE_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)s\d{1,2}(?i)e\d+").unwrap());
static EPISODE_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"($|\s)(?i)e\d+($|\s)").unwrap());

// If those fail, we try to find other patterns and take the left-most to find end of movie name
static RESOLUTION_P_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d{3,5}(?i)p").unwrap());
static RESOLUTION_X_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d{3,5}(?i)x\d{3,5}").unwrap());


// Create regexes from these as well, and check for SPACE BEFORE/START OF FILE + SPACE BEFORE/END OF FILE
static FILENAME_NOISE_PATTERNS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        String::from("x264"),
        String::from("H 264"),
        String::from("x265"),
        String::from("H 265"),
        String::from("4K"),
        String::from("6K"),
        String::from("8K"),
        String::from("HQ"),
        String::from("UHD"),
        String::from("HDR"),
        String::from("SDR"),

        String::from("WEBDL"),
        String::from("WEB dl"),
        String::from("WEBRip"),
        String::from("BluRay"),
        String::from("BRRip"),
        String::from("BDRemux"),
        String::from("REMUX"),
        String::from("DVD"),
        String::from("HDTV"),
        String::from("TrueHD"),
        String::from("REMASTERED"),
        String::from("UPSCALED"),

        String::from("AAC"),
        String::from("AVC"),
        String::from("DTS"),
        String::from("HEVC"),
        String::from("FLAC"),

        String::from("Criterion Collection"),
        String::from("AMZN"),
        String::from("CINEFILE"),
        String::from("FGT"),
        String::from("USURY"),
        String::from("YTS"),
        String::from("SADPANDA"),
        String::from("AMIABLE"),
        String::from("SELENZEN"),
        String::from("NAHOM"),
    ]
});

static CURRENT_YEAR: Lazy<i32> = Lazy::new(|| chrono::Utc::now().year());


pub fn get_movie_title_and_year_from_filename(filename_or_path: &str) -> (String, Option<i32>)
{
    let filename = &remove_extension_and_path(filename_or_path);
    let mut title_start_index = 0;
    let mut title_end_index = filename.len();
    let mut year = Option::None;

    // Wenn ein Jahr im Filmtitel selbst vorkommt kann man glaub ich einfach links von returnen
    // und die caller funktion muss halt nochmal mit jahr im titel callen wenn sie mit (titel [from year: jahr]) nichts findet

    // First we try to find a single year in brackets e.g. "Persona (1966).mkv"
    let year_matches: Vec<_> = regex_find_year(&*YEAR_IN_BRACKETS_REGEX, filename);

    if year_matches.is_empty()
    {
        // If that fails, we first remove anything in [] because if it's not the year it's probably crap
        let filename = &remove_square_bracket_text(filename);

        // Then we exchange special characters for spaces altogether
        let filename = &make_alphanumeric_with_spaces(filename);

        // Now we want to find a year seperated by spaces or at start/end of file e.g. "Caché 2005.mkv"
        let matches: Vec<_> = regex_find_year(&*YEAR_IN_SPACES_REGEX, filename);
    }

    // Last chance, maybe it's just after the title like "Nostalghia1983.mkv"
    if year_matches.is_empty()
    {
        let matches: Vec<_> = regex_find_year(&*YEAR_REGEX, filename);
    }

    // If we found something in the above, we hope the year is not at the start of the filename
    if !year_matches.is_empty()
    {
        // If we find 2 years - pick the last one, e.g. "2001 - A Space Odyssey 1968.mkv"
        let Some(year) = year_matches[year_matches.len()-1].0.parse::<i32>().unwrap();

        let start_index = year_matches[year_matches.len()-1].1;
        if (start_index > 1) {
            title_end_index = start_index;
            return (filename[title_start_index..title_end_index].to_owned(), Some(year));
        }

        // If year is at beginning of file, at least we have the year, which is nice
        // We must still look for a second identifier, and only parse the title from the year to before that
        title_start_index = year_matches[year_matches.len()-1].2;
    }


    // If we can't find a year after the title, the filename's pretty shit.
    // But we can still try to catch typical filename noise, and extract the substring before the left-most
    let regexes = [&*RESOLUTION_P_REGEX, &*RESOLUTION_X_REGEX];
    for regex in regexes {
        if let Some(match_result) = regex.find(filename) {
            if match_result.start() < title_end_index {
                title_end_index = match_result.start();
            }
        }
    }

    // TODO: Add space around each pattern?
    for pattern in FILENAME_NOISE_PATTERNS.iter() {
        if let Some(index) = filename.find(pattern) {
            if index < title_end_index {
                title_end_index = index;
            }
        }
    }


    // If the title_end_index hasn't changed, we can't do anything
    // Hopefully, the filename already equals the movie title
    //      otherwise, the calling function needs to guess by e.g. only taking the first x words
    (filename[title_start_index..title_end_index].to_owned(), None)
}

fn regex_find_year(regex: &Regex, filename: &str) -> Vec<(String, usize, usize, i32)>
{
    regex.find_iter(filename)
        .filter_map(|mat| {
            let year_str = mat.as_str();
            if let Ok(year) = year_str.parse::<i32>() {
                if 1888 <= year && year <= *CURRENT_YEAR {
                    return Some((year_str.to_string(), mat.start(), mat.end(), year));
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
use regex::Regex;
use once_cell::sync::Lazy;

use unicode_segmentation::UnicodeSegmentation;
use chrono::Datelike;

use crate::library_manager::file_utils::remove_extension_and_path;

// REGEXs to look for to find end of Movie name
static YEAR_IN_BRACKETS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:[\(\[\{])(\d{4})(?:[\)\]\}])").unwrap());
static YEAR_IN_SPACES_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|\s)(\d{4})(?:$|\s)").unwrap());
static YEAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|\D)(\d{4})(?:$|[^pPxX\d])").unwrap()); // Exactly 4 digits, no trailing p/x (resolution)
// Or TV Show name
static SEASON_EPISODE_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)s\d{1,2}(?i)e\d+").unwrap());
static EPISODE_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"($|\s)(?i)e\d+($|\s)").unwrap());

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
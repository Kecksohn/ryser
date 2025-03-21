pub fn create_valid_filename(input: &str, remove_whitespace: Option<bool>, ascii_only: Option<bool>) -> String {
    
    let remove_whitespace: bool = remove_whitespace.unwrap_or(false);
    let ascii_only = ascii_only.unwrap_or(false);
    
    // Replace characters that are invalid across operating systems
    let mut sanitized = input.replace(|c: char| {
        matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || 
        c.is_control() || 
        (remove_whitespace && c.is_whitespace()) ||
        (ascii_only && !c.is_ascii())
    }, "_");
    
    // Replace leading/trailing spaces and periods
    sanitized = sanitized.trim().to_string();
    sanitized = sanitized.trim_start_matches('.').trim_end_matches('.').to_string();
    
    // Handle reserved names in Windows
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", 
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
    ];
    
    if reserved_names.iter().any(|&name| sanitized.eq_ignore_ascii_case(name)) {
        sanitized = format!("_{}", sanitized);
    }
    
    // Ensure the filename isn't empty after sanitization
    if sanitized.is_empty() {
        sanitized = "unnamed".to_string();
    }
    
    // Limit filename length (255 is generally safe across systems)
    if sanitized.chars().count() > 255 {
        sanitized = sanitized.chars().take(255).collect();
    }
    
    sanitized
}


use regex::Regex;
pub fn get_search_strings_from_filename(filename: &str) -> Vec<String>
{
    
    let mut search_strings: Vec<String> = vec![];


    /* WIP 
    let regex_year_brackets = Regex::new(r"(\d{4})").unwrap();
    
    // Find the first match
    if let Some(mat) = re.find(input) {
        // Return the matched 4-digit number
        Some(&input[mat.start()..mat.end()])
    } 

    if let Some(mat) = re.find(input) {
        // Return everything before the match
        Some(&input[..mat.start()])
    }

    let alphanumeric_filename: String = filename.chars()
                                        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                                        .collect();


    */
    search_strings
}
//! Language code handling for audio/subtitle track selection.
//!
//! ffmpeg emits ISO 639-2 tags, predominantly the bibliographic (/B) variant
//! (`fre`, `ger`, `chi`, `rum`), but some muxers leak the terminologic (/T)
//! variant (`fra`, `deu`, `zho`, `ron`). TMDB's `original_language` is ISO
//! 639-1 (`fr`, `de`). To compare them we normalise every track tag to 639-1.

/// Normalise an ISO 639-2 tag (either /B or /T) to its ISO 639-1 code.
/// Returns `None` for non-language tags (`und`, `zxx`, `mul`, `mis`) or
/// unknown codes.
pub(crate) fn to_iso_639_1(tag: &str) -> Option<&'static str> {
    let t = tag.trim().to_ascii_lowercase();
    let code = match t.as_str() {
        // Already 639-1 (some files store it directly)
        "en" | "eng" => "en",
        "ru" | "rus" => "ru",
        "fr" | "fre" | "fra" => "fr",
        "de" | "ger" | "deu" | "gem" => "de", // gem = Germanic collective, seen mistagged for German
        "ja" | "jpn" => "ja",
        "it" | "ita" => "it",
        "es" | "spa" => "es",
        "zh" | "chi" | "zho" => "zh",
        "uk" | "ukr" => "uk",
        "pl" | "pol" => "pl",
        "cs" | "cze" | "ces" => "cs",
        "da" | "dan" => "da",
        "pt" | "por" => "pt",
        "hu" | "hun" => "hu",
        "tr" | "tur" => "tr",
        "fa" | "per" | "fas" => "fa",
        "sk" | "slo" | "slk" => "sk",
        "sl" | "slv" => "sl",
        "th" | "tha" => "th",
        "sv" | "swe" => "sv",
        "ko" | "kor" => "ko",
        "ro" | "rum" | "ron" => "ro",
        "el" | "gre" | "ell" => "el",
        "fi" | "fin" => "fi",
        "hi" | "hin" => "hi",
        "hy" | "arm" | "hye" => "hy",
        "hr" | "hrv" => "hr",
        "ka" | "geo" | "kat" => "ka",
        "id" | "ind" => "id",
        "nl" | "dut" | "nld" => "nl",
        "no" | "nor" | "nob" => "no",
        "ar" | "ara" => "ar",
        "he" | "heb" => "he",
        "ca" | "cat" => "ca",
        "vi" | "vie" => "vi",
        "et" | "est" => "et",
        "lt" | "lit" => "lt",
        "lv" | "lav" => "lv",
        "bg" | "bul" => "bg",
        "is" | "ice" | "isl" => "is",
        "mk" | "mac" | "mkd" => "mk",
        "sr" | "srp" => "sr",
        "ms" | "may" | "msa" => "ms",
        "nv" | "nav" => "nv",
        "ta" | "tam" => "ta",
        "tl" | "tgl" | "phi" => "tl", // phi = Philippine collective, seen for Tagalog
        "grc" => "grc",               // Ancient Greek - distinct, keep as-is
        // Non-language / undetermined
        "und" | "zxx" | "mul" | "mis" => return None,
        _ => return None,
    };
    Some(code)
}

/// Does a TMDB `original_language` value match a track's normalised 639-1 code?
///
/// Handles TMDB's non-ISO `cn`, which denotes Cantonese (not the ISO `zh`).
/// We treat any Chinese track (`zh`, normalised from `chi`/`zho`/`yue`) as a
/// match for a TMDB `cn` original language.
pub(crate) fn tmdb_matches(tmdb_lang: &str, track_iso1: &str) -> bool {
    let tmdb = tmdb_lang.trim().to_ascii_lowercase();
    if tmdb == "cn" {
        return track_iso1 == "zh" || track_iso1 == "yue";
    }
    tmdb == track_iso1
}

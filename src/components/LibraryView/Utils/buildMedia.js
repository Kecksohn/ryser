// What TMDB said about one episode. Wrong once the file is another episode.
const EPISODE_TMDB_FIELDS = [
  "tmdb_id",
  "name",
  "overview",
  "air_date",
  "still_path",
  "director",
];

// The media the edit view saves: the element's current media with the input values written in.
// inputs: { kind, title, original_title, countries, image, show_id, season, episode }
// kind, show_id, season and episode default to the current media's.
export function build_media(media, inputs) {
  const was_episode = media.kind === "episode";
  const {
    kind = was_episode ? "episode" : "film",
    show_id = media.show_id,
    season = media.season,
    episode = media.episode,
  } = inputs;

  if (kind === "episode") {
    // Original title and countries belong to the show and are not edited here
    const moved =
      !was_episode ||
      show_id !== media.show_id ||
      season !== media.season ||
      episode !== media.episode;
    const new_media = {
      ...(was_episode ? media : {}),
      kind: "episode",
      show_id,
      season,
      episode,
    };
    // The backend looks up an episode without tmdb_id
    if (moved) for (const field of EPISODE_TMDB_FIELDS) delete new_media[field];
    // A name or image the user entered stays, an unchanged one belonged to the old entry
    const old_title = (was_episode ? media.name : media.title) ?? "";
    const old_image = was_episode ? media.still_path : media.poster_path;
    if (!moved || inputs.title !== old_title)
      new_media.name = inputs.title || null;
    if (!moved || inputs.image !== old_image)
      new_media.still_path = inputs.image ?? null;
    return new_media;
  }

  // Saving an unmatched file or an episode makes it a film titled by the user
  const countries = inputs.countries.filter(Boolean);
  return {
    ...(media.kind === "film" ? media : {}),
    kind: "film",
    title: inputs.title || null,
    original_title: inputs.original_title || null,
    countries: countries.length > 0 ? countries : null,
    poster_path: inputs.image ?? null,
  };
}

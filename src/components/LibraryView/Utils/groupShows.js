// Episodes grouped into shows and seasons, and the state derived from them.
// Nothing here is stored: it is computed from the loaded elements.

// Library tiles: films and unmatched files as they are, plus one tile per show
// holding its episode files, at the place of the show's first episode.
export function group_library(elements) {
  const tiles = [];
  const show_tiles = new Map();
  for (const element of elements) {
    if (element.media?.kind !== "episode") {
      tiles.push(element);
      continue;
    }
    let tile = show_tiles.get(element.media.show_id);
    if (!tile) {
      tile = { kind: "show", show_id: element.media.show_id, episodes: [] };
      show_tiles.set(tile.show_id, tile);
      tiles.push(tile);
    }
    tile.episodes.push(element);
  }

  // The fields the library sort reads
  for (const tile of show_tiles.values()) {
    tile.filepath = tile.episodes.map((file) => file.filepath).sort()[0];
    tile.length_in_seconds = tile.episodes.reduce(
      (sum, file) => sum + (file.length_in_seconds ?? 0),
      0
    );
    tile.timestamp_modified = date_added(tile.episodes);
  }
  return tiles;
}

const by_filepath = (a, b) => a.filepath.localeCompare(b.filepath);

// The seasons a show has files of, specials (season 0) last, and the show's
// extras (files without season). Per season: its numbered episodes in order
// and its extras (files without episode number).
// show: the show's entry in the show table, may be missing
export function group_seasons(show, episodes) {
  const files_by_season = new Map();
  const extras = [];
  for (const file of episodes) {
    const season_number = file.media.season;
    if (season_number == null) {
      extras.push(file);
      continue;
    }
    if (!files_by_season.has(season_number)) files_by_season.set(season_number, []);
    files_by_season.get(season_number).push(file);
  }

  const seasons = [...files_by_season].map(([season_number, files]) => {
    const tmdb_season =
      show?.seasons?.find((season) => season.season_number === season_number) ?? {};
    const earliest_air_date = files
      .map((file) => file.media.air_date)
      .filter(Boolean)
      .sort()[0];
    return {
      season_number,
      name:
        tmdb_season.name ||
        (season_number === 0 ? "Specials" : "Season " + season_number),
      overview: tmdb_season.overview,
      poster_path: tmdb_season.poster_path ?? show?.poster_path,
      year: (tmdb_season.air_date || earliest_air_date)?.substring(0, 4),
      episode_count: tmdb_season.episode_count,
      episodes: files
        .filter((file) => file.media.episode != null)
        .sort((a, b) => a.media.episode - b.media.episode || by_filepath(a, b)),
      extras: files.filter((file) => file.media.episode == null).sort(by_filepath),
    };
  });
  seasons.sort(
    (a, b) =>
      (a.season_number === 0) - (b.season_number === 0) ||
      a.season_number - b.season_number
  );

  return { seasons, extras: extras.sort(by_filepath) };
}

// All files of a season or show, extras included
export const season_files = (season) => [...season.episodes, ...season.extras];

export const all_watched = (files) =>
  files.length > 0 && files.every((file) => file.watched);

const is_numbered = (file) =>
  file.media.season != null && file.media.episode != null;

// The files that decide whether a show or season is watched: its numbered
// episodes, or all its files when none has a number. Extras do not count.
export function episode_files(files) {
  const episodes = files.filter(is_numbered);
  return episodes.length > 0 ? episodes : files;
}

const watched_of = (files) =>
  `${files.filter((file) => file.watched).length}/${files.length}`;

// "Watched 3/10, Extras: 1/2"
export function watched_label(files) {
  const episodes = episode_files(files);
  const extras = files.filter((file) => !episodes.includes(file));
  return (
    `Watched ${watched_of(episodes)}` +
    (extras.length > 0 ? `, Extras: ${watched_of(extras)}` : "")
  );
}

// "have 8 of 10"; null when TMDB does not know the season's size
export function owned_label(season) {
  if (season.episode_count == null) return null;
  const owned = new Set(season.episodes.map((file) => file.media.episode)).size;
  return `have ${owned} of ${season.episode_count}`;
}

// Newest file of the show
export const date_added = (files) =>
  Math.max(...files.map((file) => file.timestamp_modified));

// A show counts as unwatched while any of its episodes is
export const is_watched = (tile) =>
  tile.kind === "show" ? all_watched(episode_files(tile.episodes)) : tile.watched;

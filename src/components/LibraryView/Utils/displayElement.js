export function filename_of(filepath) {
  return filepath.substring(
    Math.max(filepath.lastIndexOf("/"), filepath.lastIndexOf("\\")) + 1
  );
}

// "S01E02", without the parts that are not known
function episode_number(media) {
  const pad = (number) => String(number).padStart(2, "0");
  return (
    (media.season != null ? "S" + pad(media.season) : "") +
    (media.episode != null ? "E" + pad(media.episode) : "")
  );
}

// An episode's title inside its season: "S01E02 · Name"
export function episode_title(media) {
  return [episode_number(media), media.name].filter(Boolean).join(" · ");
}

// What the views show for an element, whatever kind its media is.
// A show tile from group_library() is shown from its show.
// shows: the library's show table, keyed by show ID
export function display(element, shows) {
  const filename = filename_of(element.filepath);

  if (element.kind === "show") {
    const show = shows?.[element.show_id] ?? {};
    return {
      kind: "show",
      title: show.name,
      original_title: show.original_name ?? show.name,
      sort_title: show.name || filename,
      image: show.poster_path,
      date: show.first_air_date,
      countries: show.countries,
    };
  }

  const media = element.media ?? { kind: "unmatched" };

  if (media.kind === "film") {
    return {
      kind: media.kind,
      title: media.title,
      original_title: media.original_title,
      sort_title: media.title || filename,
      image: media.poster_path,
      date: media.release_date,
      director: media.director,
      countries: media.countries,
    };
  }

  if (media.kind === "episode") {
    const show = shows?.[media.show_id] ?? {};
    const title = [show.name, episode_number(media), media.name]
      .filter(Boolean)
      .join(" ");
    return {
      kind: media.kind,
      title: title,
      original_title: title,
      sort_title: title || filename,
      image: media.still_path ?? show.poster_path,
      date: media.air_date,
      director: media.director,
      countries: show.countries,
    };
  }

  return { kind: "unmatched", sort_title: filename };
}

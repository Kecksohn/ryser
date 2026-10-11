// What the views show for an element, whatever kind its media is.
// shows: the library's show table, keyed by show ID
export function display(element, shows) {
  const media = element.media ?? { kind: "unmatched" };
  const filename = element.filepath.substring(
    Math.max(
      element.filepath.lastIndexOf("/"),
      element.filepath.lastIndexOf("\\")
    ) + 1
  );

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
    const pad = (number) => String(number).padStart(2, "0");
    const number =
      (media.season != null ? "S" + pad(media.season) : "") +
      (media.episode != null ? "E" + pad(media.episode) : "");
    const title = [show.name, number, media.name].filter(Boolean).join(" ");
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

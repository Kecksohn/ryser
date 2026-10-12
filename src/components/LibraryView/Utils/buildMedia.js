// The media the edit view saves: the element's current media with the input values written in.
// inputs: { title, original_title, countries, image }
export function build_media(media, inputs) {
  // An episode has its own name and image. Original title and countries
  // belong to its show and are not edited here.
  const is_episode = media.kind === "episode";
  const new_media = {
    ...media,
    [is_episode ? "name" : "title"]: inputs.title || null,
    [is_episode ? "still_path" : "poster_path"]: inputs.image ?? null,
  };
  if (!is_episode) {
    // Saving an unmatched file makes it a film titled by the user
    new_media.kind = "film";
    new_media.original_title = inputs.original_title || null;
    const countries = inputs.countries.filter(Boolean);
    new_media.countries = countries.length > 0 ? countries : null;
  }
  return new_media;
}

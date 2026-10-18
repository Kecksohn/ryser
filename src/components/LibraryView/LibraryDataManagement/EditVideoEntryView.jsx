import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import { CoverPickerTooltip } from "./CoverPickerTooltip.jsx";
import { TmdbSearch } from "./TmdbSearch.jsx";
import { display } from "../Utils/displayElement.js";
import { build_media } from "../Utils/buildMedia.js";

import editVideoStyles from "./EditVideoEntry.module.css";
import { useNavigate } from "react-router-dom";

// Number input value: "" = no number
const to_number = (text) => (text === "" ? null : Number(text));
const to_text = (number) => (number == null ? "" : String(number));

// save_media(entry, media, new_show_tmdb_id): with new_show_tmdb_id the show is added
// to the library first and media.show_id set to it
export const EditVideoEntryView = ({ save_media, video_entry, shows }) => {
  if (!video_entry) {
    // TODO: Get from Backend
    const { library_id } = useParams();
    const { video_element_id } = useParams();
  }

  const navigate = useNavigate();

  const media = video_entry.media ?? { kind: "unmatched" };
  // An episode has its own name and image. Original title and countries
  // belong to its show and are not edited here.
  const was_episode = media.kind === "episode";
  const title_field = was_episode ? "name" : "title";
  const image_field = was_episode ? "still_path" : "poster_path";
  const shown = display(video_entry, shows);

  const initial_kind = media.kind === "unmatched" ? "film" : media.kind;
  const [kind_input, set_kind_input] = useState(initial_kind);
  const is_episode = kind_input === "episode";
  // Episodes and extras belong to a show. An extra has nothing but show and season.
  const is_extra = kind_input === "extra";
  const has_show = is_episode || is_extra;
  // A show ID of the library, or "new" for new_show
  const [show_input, set_show_input] = useState(to_text(media.show_id));
  const [new_show, set_new_show] = useState(null);
  const [season_input, set_season_input] = useState(to_text(media.season));
  const [episode_input, set_episode_input] = useState(to_text(media.episode));

  const [original_title_input, set_original_title_input] = useState(
    media.original_title ? media.original_title : ""
  );
  const [english_title_input, set_english_title_input] = useState(
    media[title_field] ? media[title_field] : ""
  );
  const [countries_input, set_countries_input] = useState(
    media.countries ? media.countries : [""]
  );
  const [new_poster_url_or_path, set_new_poster_url_or_path] = useState(
    media[image_field]
  );

  const show =
    show_input === "new" ? new_show : has_show ? shows?.[show_input] : null;

  function was_changed() {
    const listed = (countries) => countries.filter(Boolean).join();
    return (
      kind_input !== initial_kind ||
      (has_show &&
        (show_input !== to_text(media.show_id) ||
          season_input !== to_text(media.season) ||
          episode_input !== to_text(media.episode))) ||
      (media.original_title ?? "") !== original_title_input ||
      (media[title_field] ?? "") !== english_title_input ||
      listed(media.countries ?? []) !== listed(countries_input) ||
      media[image_field] !== new_poster_url_or_path
    );
  }

  // An episode or extra needs a show
  const can_save = was_changed() && (!has_show || show_input !== "");

  function commit_changes() {
    if (!can_save) {
      console.log("No changes found.");
      return;
    }
    const media_to_save = build_media(media, {
      kind: kind_input,
      title: english_title_input,
      original_title: original_title_input,
      countries: countries_input,
      image: new_poster_url_or_path,
      ...(has_show && {
        show_id: show_input === "new" ? null : Number(show_input),
        season: to_number(season_input),
        episode: to_number(episode_input),
      }),
    });
    save_media(
      video_entry,
      media_to_save,
      has_show && show_input === "new" ? new_show.tmdb_id : undefined
    );
  }

  function update_element_with_tmdb(tmdb_result) {
    set_english_title_input(tmdb_result.title);
    // TODO: take over the rest of the result, including its tmdb_id
    set_new_poster_url_or_path(tmdb_result.poster_path);
  }

  // A show the library has already is picked from the library
  function pick_show(tmdb_show) {
    const known = Object.entries(shows ?? {}).find(
      ([, library_show]) => library_show.tmdb_id === tmdb_show.tmdb_id
    );
    set_new_show(known ? null : tmdb_show);
    set_show_input(known ? known[0] : "new");
  }

  const [is_cover_picker_open, set_is_cover_picker_open] = useState(false);
  const [tmdb_cover_results, set_tmdb_cover_results] = useState([]);

  async function get_tmdb_covers() {
    // An episode's own tmdb_id has no posters, its show's has
    const tmdb_id = is_episode
      ? show?.tmdb_id
      : media.kind === "film"
        ? media.tmdb_id
        : null;
    if (tmdb_cover_results.length === 0 && tmdb_id != null) {
      // TODO: Also send language sort order when they're in video_element
      await invoke("get_covers_from_tmdb", {
        tmdb_id: tmdb_id,
        is_tv: is_episode,
      })
        .then((poster_paths) => {
          set_tmdb_cover_results(poster_paths);
        })
        .catch((e) => {
          console.log("Error when trying to get TMDB Covers: " + e);
        });
    }
  }

  // [show_id, show], by name
  const sorted_shows = Object.entries(shows ?? {}).sort(([, a], [, b]) =>
    (a.name ?? "").localeCompare(b.name ?? "")
  );

  return (
    <div>
      {/* Back Button */}
      <div
        style={{ cursor: "pointer" }}
        onClick={() => {
          navigate(-1);
        }}
      >
        <i
          className="fa fa-angle-left"
          style={{ fontSize: "48px", color: "white" }}
        ></i>
      </div>
      <div className={editVideoStyles.container}>
        {/* Cover */}
        <div
          className={editVideoStyles.containerBox}
          style={{ textAlign: "right" }}
        >
          <CoverPickerTooltip
            images={tmdb_cover_results}
            onImageSelect={set_new_poster_url_or_path}
            position="bottom"
            isOpen={is_cover_picker_open}
            setIsOpen={set_is_cover_picker_open}
          >
            <img
              src={new_poster_url_or_path}
              style={{
                height: "500px",
                display: "block",
                marginLeft: "auto",
              }}
              onMouseEnter={() => get_tmdb_covers()}
            />
          </CoverPickerTooltip>
        </div>
        {/* Edit Info Box */}
        <div className={editVideoStyles.containerBox}>
          Filepath: {video_entry.filepath}
          <div>
            Kind:{" "}
            <select
              id="kind_input"
              value={kind_input}
              onChange={(e) => set_kind_input(e.target.value)}
            >
              <option value="film">Film</option>
              <option value="episode">Episode</option>
              <option value="extra">Extra</option>
            </select>
          </div>
          {has_show && (
            <>
              <div>
                Show:{" "}
                <select
                  id="show_input"
                  value={show_input}
                  onChange={(e) => set_show_input(e.target.value)}
                >
                  <option value="">(pick a show)</option>
                  {sorted_shows.map(([id, library_show]) => (
                    <option key={id} value={id}>
                      {library_show.name}
                    </option>
                  ))}
                  {new_show && (
                    <option value="new">
                      {new_show.name} (new from TMDB)
                    </option>
                  )}
                </select>
              </div>
              <div>
                Season:{" "}
                <input
                  id="season_input"
                  type="number"
                  min="0"
                  className={editVideoStyles.inputField}
                  value={season_input}
                  onChange={(e) => set_season_input(e.target.value)}
                />{" "}
                {is_episode && (
                  <>
                    Episode:{" "}
                    <input
                      id="episode_input"
                      type="number"
                      min="0"
                      className={editVideoStyles.inputField}
                      value={episode_input}
                      onChange={(e) => set_episode_input(e.target.value)}
                    />
                  </>
                )}
              </div>
            </>
          )}
          {!has_show && (
            <div>
              Original Title:{" "}
              <input
                id="og_title_input"
                type="text"
                className={editVideoStyles.inputField}
                value={original_title_input}
                onChange={(e) => {
                  set_original_title_input(e.target.value);
                }}
              />
            </div>
          )}
          {!is_extra && (
            <div>
              {is_episode ? "Episode Name" : "English Title"}:{" "}
              <input
                id="en_title_input"
                type="text"
                className={editVideoStyles.inputField}
                value={english_title_input}
                onChange={(e) => {
                  set_english_title_input(e.target.value);
                }}
              />
            </div>
          )}
          <div>Year: {shown.date?.substring(0, 4)}</div>
          <div>Director: {shown.director}</div>
          {!has_show && (
            <div>
              Countries:{" "}
              <input
                id="country_input"
                type="text"
                className={editVideoStyles.inputField}
                value={countries_input.join(", ")}
                onChange={(e) => {
                  set_countries_input(e.target.value.split(/, |,/));
                }}
              />
            </div>
          )}
          <div>Watched: {video_entry.watched}</div>
          <br />
          {can_save && (
            <>
              <div
                style={{ cursor: "pointer" }}
                onClick={() => {
                  commit_changes();
                  navigate(-1);
                }}
              >
                Save Changes
              </div>
              <div
                style={{ cursor: "pointer" }}
                onClick={() => {
                  navigate(-1);
                }}
              >
                Discard Changes
              </div>
            </>
          )}
        </div>
      </div>
      <br />
      {/* Remounted per kind: film results are no show results */}
      <TmdbSearch
        key={kind_input}
        tv={has_show}
        on_pick={has_show ? pick_show : update_element_with_tmdb}
      />
    </div>
  );
};

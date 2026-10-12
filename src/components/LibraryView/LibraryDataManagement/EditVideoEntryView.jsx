import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

import { CoverPickerTooltip } from "./CoverPickerTooltip.jsx";
import { display } from "../Utils/displayElement.js";
import { build_media } from "../Utils/buildMedia.js";

import editVideoStyles from "./EditVideoEntry.module.css";
import tmdbResultsStyles from "./TMDBResults.module.css";
import { useNavigate } from "react-router-dom";

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
  const is_episode = media.kind === "episode";
  const show = is_episode ? shows?.[media.show_id] : null;
  const title_field = is_episode ? "name" : "title";
  const image_field = is_episode ? "still_path" : "poster_path";
  const shown = display(video_entry, shows);

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

  function was_changed() {
    const listed = (countries) => countries.filter(Boolean).join();
    return (
      (media.original_title ?? "") !== original_title_input ||
      (media[title_field] ?? "") !== english_title_input ||
      listed(media.countries ?? []) !== listed(countries_input) ||
      media[image_field] !== new_poster_url_or_path
    );
  }

  function commit_changes() {
    if (!was_changed()) {
      console.log("No changes found.");
      return;
    }
    save_media(
      video_entry,
      build_media(media, {
        title: english_title_input,
        original_title: original_title_input,
        countries: countries_input,
        image: new_poster_url_or_path,
      })
    );
  }

  function update_element_with_tmdb(tmdb_result) {
    set_english_title_input(tmdb_result.title);
    // TODO: take over the rest of the result, including its tmdb_id
    set_new_poster_url_or_path(tmdb_result.poster_path);
  }

  const [tmdb_searchfield, set_tmdb_searchfield] = useState("");
  const [tmdb_results, set_tmdb_results] = useState([]);

  async function get_tmdb_entries(search_string) {
    await invoke("search_tmdb_from_gui", { search_title: search_string })
      .then((res) => {
        set_tmdb_results(res);
      })
      .catch((e) => {
        console.log("Error when searching TMDB: " + e);
      });
  }

  const [is_cover_picker_open, set_is_cover_picker_open] = useState(false);
  const [tmdb_cover_results, set_tmdb_cover_results] = useState([]);

  async function get_tmdb_covers() {
    // An episode's own tmdb_id has no posters, its show's has
    const tmdb_id = is_episode ? show?.tmdb_id : media.tmdb_id;
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

  return (
    <div>
      {/* Back Button */}
      <div
        style={{ cursor: "pointer" }}
        onClick={() => {
          navigate("../");
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
          {is_episode && <div>Show: {show?.name}</div>}
          {!is_episode && (
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
          <div>Year: {shown.date?.substring(0, 4)}</div>
          <div>Director: {shown.director}</div>
          {!is_episode && (
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
          {was_changed() && (
            <>
              <div
                style={{ cursor: "pointer" }}
                onClick={() => {
                  commit_changes();
                  navigate("../");
                }}
              >
                Save Changes
              </div>
              <div
                style={{ cursor: "pointer" }}
                onClick={() => {
                  navigate("../");
                }}
              >
                Discard Changes
              </div>
            </>
          )}
        </div>
      </div>
      <br />
      Search Input:{" "}
      <input
        id="searchinput"
        type="text"
        className={editVideoStyles.inputField}
        value={tmdb_searchfield}
        onChange={(e) => {
          set_tmdb_searchfield(e.target.value);
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            get_tmdb_entries(tmdb_searchfield);
          }
        }}
      />
      <span
        style={{ cursor: "pointer" }}
        onClick={() => {
          get_tmdb_entries(tmdb_searchfield);
        }}
      >
        {" "}
        (Search TMDB Icon)
      </span>
      <br />
      {tmdb_results.map((result, index) => {
        return (
          <div
            className={tmdbResultsStyles.tmdbresult}
            key={index}
            onClick={() => {
              update_element_with_tmdb(result);
            }}
          >
            <div className={tmdbResultsStyles.tmdbresultSplitter}>
              <div className={tmdbResultsStyles.tmdbresultImg}>
                <img src={result.poster_path} alt={result.title} />
              </div>
              <div className={tmdbResultsStyles.tmdbresultInfo}>
                <div>{result.title}</div>
                <div>{result.release_date?.substring(0, 4)}</div>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
};

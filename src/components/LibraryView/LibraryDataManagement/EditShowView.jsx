import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { CoverPickerTooltip } from "./CoverPickerTooltip.jsx";
import { TmdbSearch } from "./TmdbSearch.jsx";

import editVideoStyles from "./EditVideoEntry.module.css";

// reload_library() loads shows and videos again after the backend changed them
export const EditShowView = ({ shows, reload_library }) => {
  const navigate = useNavigate();
  const { library_id, show_id } = useParams();
  const show = shows[show_id];

  const [name_input, set_name_input] = useState(show?.name ?? "");
  const [poster_input, set_poster_input] = useState(show?.poster_path);
  const [rematch_target, set_rematch_target] = useState(null);

  const [is_cover_picker_open, set_is_cover_picker_open] = useState(false);
  const [cover_results, set_cover_results] = useState([]);

  if (!show) return <div className="container">Loading...</div>;

  const was_changed =
    (show.name ?? "") !== name_input || show.poster_path !== poster_input;

  function get_covers() {
    if (cover_results.length > 0 || show.tmdb_id == null) return;
    invoke("get_covers_from_tmdb", { tmdb_id: show.tmdb_id, is_tv: true })
      .then(set_cover_results)
      .catch((e) => console.log("Error when trying to get TMDB Covers: " + e));
  }

  async function save() {
    await invoke("update_show", {
      library_id,
      show_id: Number(show_id),
      name: name_input || null,
      poster_path: poster_input ?? null,
    });
    await reload_library();
    navigate(-1);
  }

  async function rematch() {
    // Another ID when the show was merged into one the library had already
    const new_show_id = await invoke("rematch_show", {
      library_id,
      show_id: Number(show_id),
      tmdb_id: rematch_target.tmdb_id,
    });
    await reload_library();
    navigate(`/library/${library_id}/show/${new_show_id}`, { replace: true });
  }

  return (
    <div>
      <div style={{ cursor: "pointer" }} onClick={() => navigate(-1)}>
        <i
          className="fa fa-angle-left"
          style={{ fontSize: "48px", color: "white" }}
        ></i>
      </div>
      <div className={editVideoStyles.container}>
        <div
          className={editVideoStyles.containerBox}
          style={{ textAlign: "right" }}
        >
          <CoverPickerTooltip
            images={cover_results}
            onImageSelect={set_poster_input}
            position="bottom"
            isOpen={is_cover_picker_open}
            setIsOpen={set_is_cover_picker_open}
          >
            <img
              src={poster_input}
              style={{ height: "500px", display: "block", marginLeft: "auto" }}
              onMouseEnter={get_covers}
            />
          </CoverPickerTooltip>
        </div>
        <div className={editVideoStyles.containerBox}>
          <div>
            Show Name:{" "}
            <input
              id="show_name_input"
              type="text"
              className={editVideoStyles.inputField}
              value={name_input}
              onChange={(e) => set_name_input(e.target.value)}
            />
          </div>
          <div>First aired: {show.first_air_date?.substring(0, 4)}</div>
          <div>TMDB: {show.tmdb_id ?? "not matched"}</div>
          <br />
          {was_changed && (
            <>
              <div style={{ cursor: "pointer" }} onClick={save}>
                Save Changes
              </div>
              <div style={{ cursor: "pointer" }} onClick={() => navigate(-1)}>
                Discard Changes
              </div>
            </>
          )}
          {rematch_target && (
            <div style={{ cursor: "pointer" }} onClick={rematch}>
              Rematch to {rematch_target.name} (
              {rematch_target.first_air_date?.substring(0, 4)})
            </div>
          )}
        </div>
      </div>
      <br />
      <TmdbSearch tv on_pick={set_rematch_target} />
    </div>
  );
};

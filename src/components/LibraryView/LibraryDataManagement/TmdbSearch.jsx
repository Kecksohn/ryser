import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

import editVideoStyles from "./EditVideoEntry.module.css";
import tmdbResultsStyles from "./TMDBResults.module.css";

// Search field with its TMDB results. tv = shows (TvShow), else films (FilmMeta).
export const TmdbSearch = ({ tv, on_pick }) => {
  const [search_text, set_search_text] = useState("");
  const [results, set_results] = useState([]);

  function search() {
    invoke(tv ? "search_tv_from_gui" : "search_tmdb_from_gui", {
      search_title: search_text,
    })
      .then(set_results)
      .catch((e) => console.log("Error when searching TMDB: " + e));
  }

  return (
    <>
      {tv ? "Search TV show" : "Search film"}:{" "}
      <input
        id="searchinput"
        type="text"
        className={editVideoStyles.inputField}
        value={search_text}
        onChange={(e) => set_search_text(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && search()}
      />
      <span style={{ cursor: "pointer" }} onClick={search}>
        {" "}
        (Search TMDB Icon)
      </span>
      <br />
      {results.map((result, index) => {
        const title = tv ? result.name : result.title;
        const date = tv ? result.first_air_date : result.release_date;
        return (
          <div
            className={tmdbResultsStyles.tmdbresult}
            key={index}
            onClick={() => on_pick(result)}
          >
            <div className={tmdbResultsStyles.tmdbresultSplitter}>
              <div className={tmdbResultsStyles.tmdbresultImg}>
                <img src={result.poster_path} alt={title} />
              </div>
              <div className={tmdbResultsStyles.tmdbresultInfo}>
                <div>{title}</div>
                <div>{date?.substring(0, 4)}</div>
              </div>
            </div>
          </div>
        );
      })}
    </>
  );
};

import React from "react";

import { useContextMenu } from "../../UITools/ContextMenu.jsx";
import { CountryFlags } from "../../UIElements/CountryFlag.jsx";

import { format_duration } from "../Utils/formatDuration.js";

import tmdbResultsStyles from "../LibraryDataManagement/TMDBResults.module.css";

export const LibraryViewScroll = ({
  library_elements,
  get_context_menu_options,
  launch_video,
}) => {
  const { useContextMenuOn } = useContextMenu();

  return library_elements.map((element) => {
    return (
      <div
        key={element.filepath}
        className={tmdbResultsStyles.tmdbresult}
        style={{ cursor: "pointer" }}
        onClick={() => launch_video(element)}
        {...useContextMenuOn(element, get_context_menu_options)}
      >
        <div className={tmdbResultsStyles.tmdbresultSplitter}>
          <div className={tmdbResultsStyles.tmdbresultImg}>
            <img src={element.poster_path} alt={element.title} />
          </div>
          <div className={tmdbResultsStyles.tmdbresultInfo}>
            {element.original_title && element.original_title}
            {element.title && element.title !== element.original_title && (
              <>
                <br /> [{element.title}]
              </>
            )}
            {!element.title && element.filepath}
            <br />
            {element.director && (
              <>
                <br />
                {element.director}
                <br />
              </>
            )}
            {element.release_date && element.release_date.length >= 4 && (
              <>
                {element.release_date.substring(0, 4)}
                <br />
              </>
            )}
            {element.countries && element.countries.length > 0 && (
              <>
                <CountryFlags
                  countries={element.countries}
                  size="small"
                  separator=" "
                />
                <br />
              </>
            )}
            <br />
            {format_duration(element.length_in_seconds)}
            <br />
            {element.watched && <span style={{ color: "green" }}>Watched</span>}
          </div>
        </div>
      </div>
    );
  });
};

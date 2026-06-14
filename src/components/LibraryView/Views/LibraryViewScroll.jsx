import React from "react";

import { useContextMenu } from "../../UITools/ContextMenu.jsx";

import { format_duration } from "../Utils/formatDuration.js";

function format_subtitles(sel) {
  switch (sel.subtitle_status) {
    case "Selected":
      return `${sel.subtitle_lang ?? "?"}${
        sel.subtitle_index != null ? ` [${sel.subtitle_index}]` : ""
      }`;
    case "NotNeeded":
      return "[not needed]";
    case "Unavailable":
    default:
      return "[unavailable]";
  }
}

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
            {element.countries && element.countries.length > 0 && (
              <>
                {element.countries.map((country, i) => {
                  return (
                    <span key={element + country}>
                      {country}
                      {i < element.countries.length - 1 && <>,</>}
                    </span>
                  );
                })}
                <br />
              </>
            )}
            <br />
            {format_duration(element.length_in_seconds)}
            <br />
            {element.playback_selection && (
              <>
                <span>
                  Language: {element.playback_selection.audio_lang ?? "?"}
                  {element.playback_selection.audio_index != null &&
                    ` [${element.playback_selection.audio_index}]`}
                </span>
                <br />
                <span>Subtitles: {format_subtitles(element.playback_selection)}</span>
                <br />
              </>
            )}
            {element.watched && <span style={{ color: "green" }}>Watched</span>}
          </div>
        </div>
      </div>
    );
  });
};

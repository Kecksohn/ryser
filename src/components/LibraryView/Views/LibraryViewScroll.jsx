import React from "react";

import { useContextMenu } from "../../UITools/ContextMenu.jsx";
import { CountryFlags } from "../../UIElements/CountryFlag.jsx";

import { display } from "../Utils/displayElement.js";
import { format_duration } from "../Utils/formatDuration.js";
import { languageName } from "../Utils/languageName.js";
import { watched_label } from "../Utils/groupShows.js";

import tmdbResultsStyles from "../LibraryDataManagement/TMDBResults.module.css";

const NO_SUBTITLES = "none";

function track_label(index, languages, titles) {
  const language = languageName(languages[index]);
  const title = titles?.[index];
  // Prefix with the written-out language (some titles omit it), then the
  // track's own description when present.
  return title && title.trim() ? `${language}: ${title}` : language;
}

// Stop row click (which launches the movie) when interacting with a dropdown.
const swallow = (e) => e.stopPropagation();

const TrackSelect = ({ label, value, onChange, children }) => (
  <div
    className={tmdbResultsStyles.trackSelector}
    onClick={swallow}
    onMouseDown={swallow}
  >
    <span>{label} </span>
    <select value={value} onChange={onChange} onClick={swallow}>
      {children}
    </select>
  </div>
);

export const LibraryViewScroll = ({
  library_elements,
  shows,
  get_context_menu_options,
  launch_video,
  get_track_selection,
  on_audio_change,
  on_subtitle_change,
  // A show tile opens its show instead of playing
  open_show,
  // Replaces display() where the context names the element differently
  display_element = display,
}) => {
  const { useContextMenuOn } = useContextMenu();

  return library_elements.map((element) => {
    const audio_languages = element.audio_languages ?? [];
    const audio_titles = element.audio_titles ?? [];
    const subtitle_languages = element.subtitle_languages ?? [];
    const subtitle_titles = element.subtitle_titles ?? [];
    const selection = get_track_selection(element);
    const shown = display_element(element, shows);
    const is_show = element.kind === "show";

    return (
      <div
        key={is_show ? "show-" + element.show_id : element.filepath}
        className={tmdbResultsStyles.tmdbresult}
        style={{ cursor: "pointer" }}
        onClick={() => (is_show ? open_show(element) : launch_video(element))}
        {...useContextMenuOn(element, get_context_menu_options)}
      >
        <div className={tmdbResultsStyles.tmdbresultSplitter}>
          <div className={tmdbResultsStyles.tmdbresultImg}>
            <img src={shown.image} alt={shown.title} />
          </div>
          <div className={tmdbResultsStyles.tmdbresultInfo}>
            {shown.original_title && shown.original_title}
            {shown.title && shown.title !== shown.original_title && (
              <>
                <br /> [{shown.title}]
              </>
            )}
            {!shown.title && element.filepath}
            <br />
            {shown.director && (
              <>
                <br />
                {shown.director}
                <br />
              </>
            )}
            {shown.date && shown.date.length >= 4 && (
              <>
                {shown.date.substring(0, 4)}
                <br />
              </>
            )}
            {shown.countries && shown.countries.length > 0 && (
              <>
                <CountryFlags
                  countries={shown.countries}
                  size="small"
                  separator=" "
                />
                <br />
              </>
            )}
            <br />
            {is_show && <span>{watched_label(element.episodes)}</span>}
            {!is_show && format_duration(element.length_in_seconds)}
            <br />
            {!is_show && audio_languages.length > 0 && (
              <TrackSelect
                label="Audio:"
                value={String(selection.audio_index)}
                onChange={(e) =>
                  on_audio_change(element, Number(e.target.value))
                }
              >
                {audio_languages.map((_, i) => (
                  <option key={i} value={String(i)}>
                    {track_label(i, audio_languages, audio_titles)}
                  </option>
                ))}
              </TrackSelect>
            )}
            {!is_show && (
              <TrackSelect
                label="Subtitles:"
                value={
                  selection.subtitle_index === NO_SUBTITLES
                    ? NO_SUBTITLES
                    : String(selection.subtitle_index)
                }
                onChange={(e) =>
                  on_subtitle_change(
                    element,
                    e.target.value === NO_SUBTITLES
                      ? NO_SUBTITLES
                      : Number(e.target.value)
                  )
                }
              >
                {subtitle_languages.map((_, i) => (
                  <option key={i} value={String(i)}>
                    {track_label(i, subtitle_languages, subtitle_titles)}
                  </option>
                ))}
                <option value={NO_SUBTITLES}>No subtitles</option>
              </TrackSelect>
            )}
            {element.watched && <span style={{ color: "green" }}>Watched</span>}
          </div>
        </div>
      </div>
    );
  });
};

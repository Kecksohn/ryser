import { useNavigate, useParams } from "react-router-dom";

import { useContextMenu } from "../../UITools/ContextMenu.jsx";
import { HeaderBar } from "../../UIElements/HeaderBar.jsx";
import { LibraryViewScroll } from "./LibraryViewScroll.jsx";

import { display, episode_title, filename_of } from "../Utils/displayElement.js";
import {
  all_watched,
  episode_files,
  group_seasons,
  owned_label,
  season_files,
  watched_label,
} from "../Utils/groupShows.js";

import tmdbResultsStyles from "../LibraryDataManagement/TMDBResults.module.css";

// The show tile of the route's show_id, with its show and seasons
function useShow(tiles, shows) {
  const { library_id, show_id } = useParams();
  const tile = tiles.find(
    (tile) => tile.kind === "show" && String(tile.show_id) === show_id
  );
  const show = shows[show_id] ?? {};
  return {
    library_id,
    tile,
    show,
    show_link: `/library/${library_id}/show/${show_id}`,
    ...(tile
      ? group_seasons(show, tile.episodes)
      : { seasons: [], unnumbered: [], extras: [] }),
  };
}

const WatchedButton = ({ what, files, set_files_watched }) => {
  const watched = all_watched(files);
  return (
    <span
      style={{ cursor: "pointer" }}
      onClick={() => set_files_watched(files, !watched)}
    >
      {watched ? `Mark ${what} unwatched` : `Mark ${what} watched`}
    </span>
  );
};

// Files without number are listed by their filename
const FilesWithoutNumber = ({ heading, files, shows, scroll_props }) =>
  files.length > 0 && (
    <>
      <h3>{heading}</h3>
      <LibraryViewScroll
        library_elements={files}
        shows={shows}
        {...scroll_props}
        display_element={(file) => ({
          title: filename_of(file.filepath),
          original_title: filename_of(file.filepath),
        })}
      />
    </>
  );

export const ShowView = ({ tiles, shows, scroll_props, set_files_watched }) => {
  const navigate = useNavigate();
  const { useContextMenuOn } = useContextMenu();
  const { library_id, tile, show, show_link, seasons, unnumbered, extras } =
    useShow(tiles, shows);
  if (!tile) return <div className="container">Loading...</div>;

  return (
    <div className="container">
      <HeaderBar
        leftside_text={<span className={"headerbar-title"}>{show.name}</span>}
        back_link={`/library/${library_id}`}
      />
      <br />
      {show.overview && <p>{show.overview}</p>}
      <span>{watched_label(tile.episodes)}</span>{" "}
      <WatchedButton
        what="show"
        files={episode_files(tile.episodes)}
        set_files_watched={set_files_watched}
      />{" "}
      <span
        style={{ cursor: "pointer" }}
        onClick={() => navigate(`${show_link}/edit`)}
      >
        Edit show
      </span>
      <br />
      {seasons.map((season) => (
        <div
          key={season.season_number}
          className={tmdbResultsStyles.tmdbresult}
          onClick={() => navigate(`${show_link}/season/${season.season_number}`)}
          {...useContextMenuOn(
            { kind: "season", show_id: tile.show_id, files: season_files(season) },
            scroll_props.get_context_menu_options
          )}
        >
          <div className={tmdbResultsStyles.tmdbresultSplitter}>
            <div className={tmdbResultsStyles.tmdbresultImg}>
              <img src={season.poster_path} alt={season.name} />
            </div>
            <div className={tmdbResultsStyles.tmdbresultInfo}>
              {season.name}
              <br />
              {season.year && (
                <>
                  {season.year}
                  <br />
                </>
              )}
              {owned_label(season) && (
                <>
                  {owned_label(season)}
                  <br />
                </>
              )}
              {watched_label(season_files(season))}
            </div>
          </div>
        </div>
      ))}
      <WithoutNumber
        unnumbered={unnumbered}
        extras={extras}
        shows={shows}
        scroll_props={scroll_props}
      />
    </div>
  );
};

const WithoutNumber = ({ unnumbered, extras, ...props }) => (
  <>
    <FilesWithoutNumber heading="Unnumbered episodes" files={unnumbered} {...props} />
    <FilesWithoutNumber heading="Extras" files={extras} {...props} />
  </>
);

export const SeasonView = ({ tiles, shows, scroll_props, set_files_watched }) => {
  const { season_number } = useParams();
  const { tile, show, show_link, seasons } = useShow(tiles, shows);
  const season = seasons.find(
    (season) => String(season.season_number) === season_number
  );
  if (!tile || !season) return <div className="container">Loading...</div>;

  return (
    <div className="container">
      <HeaderBar
        leftside_text={
          <span className={"headerbar-title"}>
            {show.name} · {season.name}
          </span>
        }
        back_link={show_link}
      />
      <br />
      {season.overview && <p>{season.overview}</p>}
      <span>
        {[owned_label(season), watched_label(season_files(season))]
          .filter(Boolean)
          .join(", ")}
      </span>{" "}
      <WatchedButton
        what="season"
        files={episode_files(season_files(season))}
        set_files_watched={set_files_watched}
      />
      <br />
      <LibraryViewScroll
        library_elements={season.episodes}
        shows={shows}
        {...scroll_props}
        display_element={(file) => {
          const title = episode_title(file.media);
          return { ...display(file, shows), title, original_title: title };
        }}
      />
      <WithoutNumber
        unnumbered={season.unnumbered}
        extras={season.extras}
        shows={shows}
        scroll_props={scroll_props}
      />
    </div>
  );
};

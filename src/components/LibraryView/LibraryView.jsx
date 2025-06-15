import { useState, useEffect, useMemo } from "react";
import { Route, Routes, useNavigate, useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { HeaderBar } from "../UIElements/HeaderBar.jsx";
import { Dropdown } from "../UIElements/Dropdown.jsx";
import { useContextMenu } from "../UITools/ContextMenu.jsx";

import { EditVideoEntryView } from "./LibraryDataManagement/EditVideoEntryView.jsx";

import { sort_video_elements } from "./Utils/sortVideoElements.js";
import { format_duration } from "./Utils/formatDuration.js";
import { video_element_context_menu_options } from "./ContextMenuVideoElement.js";

import tmdbResultsStyles from "./LibraryDataManagement/TMDBResults.module.css";
import react_icon from "../../assets/react.svg";

export const LibraryView = () => {
  const { useContextMenuOn } = useContextMenu();
  const navigate = useNavigate();
  const [, forceRerender] = useState(0);

  const { library_id } = useParams();
  const [library_name, set_library_name] = useState("Loading...");

  // Load Library

  const [library_elements, set_library_elements] = useState([]);
  const [library_elements_loaded, set_library_elements_loaded] =
    useState(false);

  useEffect(() => {
    if (!library_elements_loaded) {
      set_library_elements_loaded(true);

      invoke("get_library_name", { library_id: library_id }).then((res) => {
        set_library_name(res);

        invoke("get_library_videos", {
          library_id: library_id,
        }).then((res) => {
          set_library_elements(res);
        });
      });
    }
  }, [library_elements_loaded]);

  const [watched_filter, set_watched_filter] = useState("");

  const filtered_library_elements = useMemo(() => {
    return library_elements.filter((element) => {
      return watched_filter === "filter_watched"
        ? !element.watched
        : watched_filter === "filter_unwatched"
          ? element.watched
          : true; // no filter
    });
  }, [library_elements, watched_filter]);

  // Functions

  async function launch_video(video) {
    const process_id_option = await invoke("start_video_in_mpc", {
      filepath: video.filepath,
    });
    if (!process_id_option) {
      console.log("Failed to start videoplayer!");
      return;
    }

    if (!video.watched) {
      // After the film's duration, check if the videoplayer wasn't closed, and if so, set the film as watched
      const percentage_needed_to_set_watched = 80; // |TODO: Get from library settings

      await new Promise((resolve) =>
        setTimeout(
          resolve,
          ((video.length_in_seconds * percentage_needed_to_set_watched) / 100) *
            1000
        )
      );

      const is_running = await invoke("is_process_running", {
        process_id: process_id_option,
      });

      if (is_running) {
        set_watched(video);
      }
    }
  }

  async function update_element_in_library(updated_element) {
    await invoke("update_library_entry_from_gui", {
      library_id: library_id,
      updated_element: updated_element,
    });
  }

  function toggle_watched(element) {
    element.watched = !element.watched;
    forceRerender((prev) => prev + 1);
    update_element_in_library(element);
  }
  function set_watched(element) {
    if (!element.watched) toggle_watched(element);
  }
  function set_not_watched(element) {
    if (element.watched) toggle_watched(element);
  }

  // Context Menu
  const [selected_element, set_selected_element] = useState(null);

  const get_context_menu_options = video_element_context_menu_options({
    set_selected_element,
    toggle_watched,
  });

  // Dropdown
  const sort_dropdown_options = () => [
    { label: "Title", onClick: () => sort_library_elements("title") },
    { label: "Duration", onClick: () => sort_library_elements("duration") },
    {
      label: "Date Added",
      onClick: () => sort_library_elements("timestamp"),
    },
    { label: "Filepath", onClick: () => sort_library_elements("filepath") },
  ];
  const [last_sort_order, set_last_sort_order] = useState("filepath");

  function sort_library_elements(order) {
    set_library_elements(
      sort_video_elements(
        library_elements,
        order,
        last_sort_order,
        set_last_sort_order
      )
    );
  }

  return (
    <Routes>
      <Route
        path="/"
        element={
          <div className="container">
            <HeaderBar
              leftside_text={
                <span>
                  <span style={{ fontSize: "1.8em" }}>{library_name}</span>
                  <img
                    src={react_icon}
                    onClick={() => navigate(settings_link)}
                    alt="Change Sort"
                  />
                  <img
                    src={react_icon}
                    onClick={() => navigate(settings_link)}
                    alt="Chage Filters"
                  />
                </span>
              }
              back_link={"/"}
              settings_link={"/library/" + library_id}
            />
            <br />

            {watched_filter !== "filter_watched" &&
              watched_filter !== "filter_unwatched" && (
                <span
                  style={{ cursor: "pointer" }}
                  onClick={() => set_watched_filter("filter_watched")}
                >
                  Filter Watched
                </span>
              )}
            {watched_filter === "filter_watched" && (
              <span
                style={{ cursor: "pointer" }}
                onClick={() => set_watched_filter("filter_unwatched")}
              >
                Filter Unwatched
              </span>
            )}
            {watched_filter === "filter_unwatched" && (
              <span
                style={{ cursor: "pointer" }}
                onClick={() => set_watched_filter("")}
              >
                Remove Filter
              </span>
            )}
            <Dropdown
              buttonText={"Sort"}
              options={sort_dropdown_options()}
              scale={1}
            />
            <br />

            {filtered_library_elements.map((element) => {
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
                      {element.title &&
                        element.title !== element.original_title && (
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
                      {element.watched && (
                        <span style={{ color: "green" }}>Watched</span>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        }
      />
      <Route
        path="/edit_element/:video_element_id"
        element={
          <EditVideoEntryView
            update_element_in_library={update_element_in_library}
            video_entry={selected_element}
          />
        }
      />
    </Routes>
  );
};

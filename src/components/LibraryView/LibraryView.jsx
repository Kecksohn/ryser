import { useState, useEffect, useMemo } from "react";
import { Route, Routes, useNavigate, useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { HeaderBar } from "../UIElements/HeaderBar.jsx";
import { Dropdown } from "../UIElements/Dropdown.jsx";
import { useContextMenu } from "../UITools/ContextMenu.jsx";

import { LibraryViewScroll } from "./Views/LibraryViewScroll.jsx";

import { EditVideoEntryView } from "./LibraryDataManagement/EditVideoEntryView.jsx";

import { sort_video_elements } from "./Utils/sortVideoElements.js";
import { video_element_context_menu_options } from "./ContextMenuVideoElement.js";

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
  const [sort_preference_loaded, set_sort_preference_loaded] = useState(false);

  // Load sort preference when library is loaded
  useEffect(() => {
    if (library_elements_loaded && !sort_preference_loaded && library_elements.length > 0) {
      set_sort_preference_loaded(true);
      invoke("get_library_sort_preference", { library_id: library_id })
        .then((preference) => {
          // Apply the loaded sort preference immediately
          const sorted_elements = sort_video_elements(
            library_elements,
            preference,
            "filepath", // Use filepath as initial state since we haven't sorted yet
            () => {} // Don't update last_sort_order during initial load
          );
          set_library_elements(sorted_elements);
          set_last_sort_order(preference);
        })
        .catch((error) => {
          console.log("Failed to load sort preference:", error);
          // Use default sort if loading fails
          const sorted_elements = sort_video_elements(
            library_elements,
            "title",
            "filepath",
            () => {}
          );
          set_library_elements(sorted_elements);
          set_last_sort_order("title");
        });
    }
  }, [library_elements_loaded, sort_preference_loaded, library_elements]);

  function sort_library_elements_internal(order) {
    set_library_elements(
      sort_video_elements(
        library_elements,
        order,
        last_sort_order,
        set_last_sort_order
      )
    );
  }

  function sort_library_elements(order) {
    sort_library_elements_internal(order);
    // Save the sort preference to backend
    invoke("set_library_sort_preference", {
      library_id: library_id,
      sort_preference: order
    }).catch((error) => {
      console.log("Failed to save sort preference:", error);
    });
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
                  <span className={"headerbar-title"}>{library_name}</span>
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

            <LibraryViewScroll
              library_elements={filtered_library_elements}
              get_context_menu_options={get_context_menu_options}
              launch_video={launch_video}
            />
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

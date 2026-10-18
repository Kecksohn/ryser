import { useNavigate, useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { all_watched, episode_files } from "./Utils/groupShows.js";

// Get OS-specific label for revealing files based on the user agent
const getRevealLabel = () => {
  const userAgent = window.navigator.userAgent.toLowerCase();
  if (userAgent.includes("mac")) {
    return "Reveal in Finder";
  } else if (userAgent.includes("win")) {
    return "Show in Explorer";
  } else {
    return "Show in File Manager";
  }
};

export const video_element_context_menu_options = ({
  set_selected_element,
  toggle_watched,
  set_files_watched,
  refresh_element,
}) => {
  const navigate = useNavigate();
  const { library_id } = useParams();

  async function show_in_explorer(filepath) {
    try {
      await invoke("show_in_explorer", { filepath });
    } catch (error) {
      console.error("Failed to show file in explorer:", error);
    }
  }

  const watched_option = (files, suffix) => {
    const watched = all_watched(files);
    return {
      label: (watched ? "Mark unwatched" : "Mark watched") + suffix,
      action: () => set_files_watched(files, !watched),
      close_after: true,
    };
  };

  // Show tile or season tile: files are all its files, extras included
  const show_options = ({ show_id, files }) => {
    const episodes = episode_files(files);
    return [
      {
        label: "Edit show",
        action: () => navigate(`/library/${library_id}/show/${show_id}/edit`),
        close_after: true,
      },
      watched_option(episodes, ""),
      // Same as the above without extras
      episodes.length < files.length && watched_option(files, " (with Extras)"),
    ].filter(Boolean);
  };

  return (context) =>
    context.kind === "show"
      ? show_options({ show_id: context.show_id, files: context.episodes })
      : context.kind === "season"
        ? show_options(context)
        : [
    {
      label: "Edit",
      action: () => {
        if (set_selected_element) set_selected_element(context);
        // Absolute: the menu is also open in the show and season views
        navigate(`/library/${library_id}/edit_element/${context.media?.tmdb_id}`); // TODO: Change to ID
      },
      close_after: true,
    },
    toggle_watched && {
      label: context.watched ? "Mark unwatched" : "Mark watched",
      action: () => {
        toggle_watched(context);
      },
      close_after: true,
    },
    {
      label: getRevealLabel(),
      action: () => {
        show_in_explorer(context.filepath);
      },
      close_after: true,
    },
    refresh_element && {
      label: "Refresh file info",
      action: () => {
        refresh_element(context);
      },
      close_after: true,
    },
    { label: "no impl: Remove from Library", action: () => {} },
    { label: "no impl: Delete from Storage", action: () => {} },
  ];
};

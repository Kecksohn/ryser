import { useNavigate, useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

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

  return (context) => [
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

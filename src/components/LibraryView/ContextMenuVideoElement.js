import { useNavigate } from "react-router-dom";
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
}) => {
  const navigate = useNavigate();

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
        navigate(`edit_element/${context.year}`); // TODO: Change to ID
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
    { label: "no impl: Remove from Library", action: () => {} },
    { label: "no impl: Delete from Storage", action: () => {} },
  ];
};

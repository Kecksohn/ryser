import { useNavigate } from "react-router-dom";

export const video_element_context_menu_options = ({
  set_selected_element,
  toggle_watched,
}) => {
  const navigate = useNavigate();

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
    { label: "no impl: Show in Windows Explorer", action: () => {} },
    { label: "no impl: Remove from Library", action: () => {} },
    { label: "no impl: Delete from Storage", action: () => {} },
  ];
};

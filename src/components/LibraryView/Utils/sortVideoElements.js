import { display } from "./displayElement.js";

export const sort_video_elements = (
  library_elements,
  order,
  last_sort_order,
  set_last_sort_order,
  shows
) => {
  let library_elements_copy = library_elements.slice();
  switch (order) {
    case "title":
      library_elements_copy = library_elements_copy.sort((a, b) => {
        return display(a, shows).sort_title.localeCompare(
          display(b, shows).sort_title
        );
      });
      if (last_sort_order === "title") {
        library_elements_copy.reverse();
        set_last_sort_order("title_reverse");
      } else {
        set_last_sort_order("title");
      }
      break;

    case "duration":
      library_elements_copy = library_elements_copy.sort(
        (a, b) => a.length_in_seconds - b.length_in_seconds
      );
      if (last_sort_order === "duration") {
        library_elements_copy.reverse();
        set_last_sort_order("duration_reverse");
      } else {
        set_last_sort_order("duration");
      }
      break;

    case "timestamp":
      library_elements_copy = library_elements_copy.sort(
        (a, b) => b.timestamp_modified - a.timestamp_modified
      );
      if (last_sort_order === "timestamp") {
        library_elements_copy.reverse();
        set_last_sort_order("timestamp_reverse");
      } else {
        set_last_sort_order("timestamp");
      }
      break;

    case "filepath":
      library_elements_copy = library_elements_copy.sort((a, b) =>
        a.filepath.localeCompare(b.filepath)
      );
      if (last_sort_order === "filepath") {
        library_elements_copy.reverse();
        set_last_sort_order("filepath_reverse");
      } else {
        set_last_sort_order("filepath");
      }
      break;

    default:
      console.log("Unknown sort type '" + order + "'");
      return;
  }

  return library_elements_copy;
};

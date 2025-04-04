

export const sort_video_elements = (library_elements, order, last_sort_order, set_last_sort_order) => {

    let library_elements_copy = library_elements.slice();
    switch(order) {

        case "title":
            library_elements_copy = library_elements_copy.sort((a, b) => {
                // Use title if available, otherwise fallback to filepath
                const titleA = a.title || a.filepath.substring(a.filepath.lastIndexOf("/")+1);
                const titleB = b.title || b.filepath.substring(b.filepath.lastIndexOf("/")+1);
                return titleA.localeCompare(titleB);
            });
            if (last_sort_order === "title") {
                library_elements_copy.reverse();
                set_last_sort_order("title_reverse")
            }
            else {
                set_last_sort_order("title");
            }
            break;

        case "duration":
            library_elements_copy = library_elements_copy.sort(
                (a,b) => (a.length_in_seconds - b.length_in_seconds)
            )
            if (last_sort_order === "duration") {
                library_elements_copy.reverse();
                set_last_sort_order("duration_reverse")
            }
            else {
                set_last_sort_order("duration");
            }
            break;

        case "timestamp":
            library_elements_copy = library_elements_copy.sort(
                (a,b) => (b.timestamp_modified - a.timestamp_modified)
            )
            if (last_sort_order === "timestamp") {
                library_elements_copy.reverse();
                set_last_sort_order("timestamp_reverse")
            }
            else {
                set_last_sort_order("timestamp");
            }
            break;

        case "filepath":
            library_elements_copy = library_elements_copy.sort(
                (a,b) => (a.filepath.localeCompare(b.filepath)));
            if (last_sort_order === "filepath") {
                library_elements_copy.reverse();
                set_last_sort_order("filepath_reverse");
            }
            else {
                set_last_sort_order("filepath");
            }
            break;

        default:
            console.log("Unknown sort type '" + order + "'");
            return;
    }

    return library_elements_copy;
}


export function format_duration(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (hours > 0) {
        return `${hours}h${minutes}m`;
    } else {
        return `${minutes}m`;
    }
}
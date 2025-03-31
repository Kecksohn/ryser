import { useState, useEffect, useMemo } from "react";
import { useParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { HeaderBar } from "./HeaderBar.jsx";
import { Dropdown } from "./Dropdown.jsx";
import { ContextMenu } from "./ContextMenu.jsx";

import { EditVideoEntryView } from "./EditVideoEntryView.jsx";

import "./TMDBResults.css";
import react_icon from "./assets/react.svg";

export const LibraryView = () => {

    const { library_id } = useParams();
    const [, forceRerender] = useState(0);

    const [library_name, set_library_name] = useState("Loading...");

    // Load Library
    const [library_elements, set_library_elements] = useState([]);
    const [library_elements_loaded, set_library_elements_loaded] = useState(false)
  
    useEffect(() => {
      if (!library_elements_loaded) {
        set_library_elements_loaded(true);
        
        invoke("get_library_name", {library_id: library_id})
        .then(res => {
            set_library_name(res);
            
            invoke("get_library_videos", {library_id: library_id})
            .then(res => { 
                set_library_elements(res);
            });
        })
      }
    });

    const [watched_filter, set_watched_filter] = useState("");

    const filtered_library_elements = useMemo(() => {
        return library_elements.filter(element => {
            const matches_watched =
                watched_filter === "filter_watched" ? !element.watched
                : watched_filter === "filter_unwatched" ? element.watched :
                true // no filter

            return matches_watched;
        })
    }, [library_elements, watched_filter])


    // Functions

    async function launch_video(video) {
        const process_id_option = await invoke("start_video_in_mpc", { filepath: video.filepath });
        if (!process_id_option) {
            console.log("Failed to start videoplayer!");
            return;
        } 

        if (!video.watched) {
            // After the film's duration, check if the videoplayer wasn't closed, and if so, set the film as watched
            const percentage_needed_to_set_watched = 80; // |TODO: Get from library settings
            
            await new Promise(resolve => 
                setTimeout(resolve, (video.length_in_seconds * (percentage_needed_to_set_watched) / 100) * 1000)
            );

            const is_running = await invoke("is_process_running", { process_id: process_id_option });

            if (is_running) {
                set_watched(video);
            }
        }
    }

    async function update_element_in_library(updated_element) {
        await invoke("update_library_entry_from_gui", {library_id: library_id, updated_element: updated_element});
    }

    function toggle_watched(element) {
        element.watched = !element.watched;
        forceRerender((prev) => prev + 1);
        update_element_in_library(element);
    }
    function set_watched(element) { if (!element.watched) toggle_watched(element); }
    function set_not_watched(element) { if (element.watched) toggle_watched(element); }
    

    // Context Menu

    const [context_menu_state, set_context_menu_state] = useState({
        visible: false,
        position: { x: 0, y: 0 },
        context: null // Store what was clicked
    });

    const get_context_menu_options = (context) => {
        return [
            { label: 'Edit', action: () => {set_edit_entry_view_visible(true); close_context_menu();} },
            { label: context.watched ? 'Mark unwatched' : 'Mark watched', action: () => {toggle_watched(context); close_context_menu();} },
            { label: 'no impl: Show in Windows Explorer', action: () => {close_context_menu();} },
            { label: 'no impl: Remove from Library', action: () => {close_context_menu();} },
            { label: 'no impl: Delete from Storage', action: () => {close_context_menu();} }
        ];
    };

    const handle_context_menu = (event, context) => {
        event.preventDefault();
        set_context_menu_state({
          visible: true,
          position: { x: event.clientX, y: event.clientY },
          context
        });
    };

    const close_context_menu = () => {
      set_context_menu_state(prev => ({ ...prev, visible: false }));
    }
    
    // Close Context menu when clicking outside
    useEffect(() => {
      const handleClick = () => close_context_menu();
      document.addEventListener('click', handleClick);
      return () => document.removeEventListener('click', handleClick);
    }, []);


    // Dropdown

    const sort_dropdown_options = () => [
        { label: 'Title', onClick: () => sort_video_elements("title") },
        { label: 'Duration', onClick: () => sort_video_elements("duration") },
        { label: 'Date Added', onClick: () => sort_video_elements("timestamp") },
        { label: 'Filepath', onClick: () => sort_video_elements("filepath") },
    ];
    const [last_sort_order, set_last_sort_order] = useState("filepath");

    const sort_video_elements = (order) => {

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
        set_library_elements(library_elements_copy);
    }
    
  
    const [edit_entry_view_visible, set_edit_entry_view_visible] = useState(false);
    const disable_edit_entry_view = () => {
      set_edit_entry_view_visible(false);
    }


    function format_duration(seconds) {
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);

        if (hours > 0) {
            return `${hours}h${minutes}m`;
        } else {
            return `${minutes}m`;
        }
    }


    return (
      <div className="container">
        <HeaderBar 
            leftside_text={
                <span>
                <span style={{fontSize: "1.8em"}}>{library_name}</span>
                <img src={react_icon} onClick={() => navigate(settings_link)} alt="Change Sort" />
                <img src={react_icon} onClick={() => navigate(settings_link)} alt="Chage Filters" />
                </span>
            } 
            back_link={"/"}
            settings_link={"/library/"+library_id}
        />
            <br/>
          {watched_filter !== "filter_watched" && watched_filter !== "filter_unwatched" &&
              <span style={{cursor: "pointer"}} onClick={() => set_watched_filter("filter_watched")}>Filter Watched</span>}
          {watched_filter === "filter_watched" && <span style={{cursor: "pointer"}} onClick={() => set_watched_filter("filter_unwatched")}>Filter Unwatched</span>}
          {watched_filter === "filter_unwatched" && <span style={{cursor: "pointer"}} onClick={() => set_watched_filter("")}>Remove Filter</span>}
          <Dropdown buttonText={"Sort"} options={sort_dropdown_options()}/>
          <br/>
        {
          !edit_entry_view_visible && filtered_library_elements.map(element => {
            return(
              <div key={element.filepath}
                   className={"tmdbresult"}
                style={{cursor: "pointer"}}
                onClick={() => launch_video(element)}
                onContextMenu={(e) => handle_context_menu(e, element)}>
                  <div className={"tmdbresult-splitter"}>
                      <div className={"tmdbresult-img"}>
                          <img src={element.poster_path} alt={element.title}/>
                      </div>
                      <div className={"tmdbresult-info"}>
                        {element.original_title && element.original_title}
                        {element.title && element.title != element.original_title && <><br/> [{element.title}]</>}
                        {!element.title && element.filepath}
                        <br/>
                        {element.director && <><br/>{element.director}<br/></>}
                        {element.countries && element.countries.length > 0 
                            && <>{element.countries.map((country, i) => {return(<>{country}{i < element.countries.length-1 && <>,</>}</>)})}<br/></>}
                        <br/>
                        {format_duration(element.length_in_seconds)}<br/>
                        {element.watched && <span style={{color: "green"}}>Watched</span>}
                      </div>
                  </div>
              </div>
            )
          })
        }

        {context_menu_state.visible && (
            <ContextMenu 
              menu_items={get_context_menu_options(context_menu_state.context)}
              position={context_menu_state.position}
            />
        )}

        {edit_entry_view_visible && (
          <EditVideoEntryView 
            disable_view={disable_edit_entry_view}
            update_element_in_library={update_element_in_library}
            video_entry={context_menu_state.context}
          />
        )}
        
      </div>
    );

}
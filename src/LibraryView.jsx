import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

import { Dropdown } from "./Dropdown.jsx";
import { ContextMenu } from "./ContextMenu";

import { EditVideoEntryView } from "./EditVideoEntryView";

import "./TMDBResults.css";

export const LibraryView = ({library_id}) => {

  // Load Library
    const [library_elements, set_library_elements] = useState([]);
    const [library_elements_loaded, set_library_elements_loaded] = useState(false)
  
    useEffect(() => {
      if (!library_elements_loaded) {
        set_library_elements_loaded(true);
        invoke("get_library_videos", {library_id: library_id}).then(res => { 
          set_library_elements(res);
        });
      }
    });


    // Functions

    async function launch_video(full_filepath) {
        await invoke("start_video_in_mpc", {filepath: full_filepath});
    }

    async function update_element_in_library(updated_element, library_index = null) {
        /*  
        if (!library_index) {
            for (const [i, element] of library_elements.entries) {
                if (element.filepath === updated_element.filepath) {
                    library_index = i;
                    break;
                }
            }
        }
        if (!library_index) {
            console.log(`Did not find library index for ${updated_element.filepath}`);
            return;
        }
        
        set_library_elements([
          ...library_elements.slice(0, library_index),
          { ...library_elements[targetIndex], updated_element },
          ...library_elements.slice(library_index + 1)
        ]);
        */
        await invoke("update_library_entry_from_gui", {library_id: library_id, updated_element: updated_element});
    }
    

    // Context Menu

    const [context_menu_state, set_context_menu_state] = useState({
        visible: false,
        position: { x: 0, y: 0 },
        context: null // Store what was clicked
    });

    const get_context_menu_options = (context) => {
        return [
            { label: 'Edit', action: () => {set_edit_entry_view_visible(true); close_context_menu();} },
            { label: 'no impl: Mark watched', action: () => {close_context_menu();} },
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
    ];
    const [last_sort_order, set_last_sort_order] = useState("nothing");

    const sort_video_elements = (order) => {

        let library_elements_copy = library_elements.slice();
        switch(order) {

            case "title":
                library_elements_copy = library_elements_copy.sort((a, b) => {
                    // Use title if available, otherwise fallback to filepath
                    const titleA = a.title || a.filepath;
                    const titleB = b.title || b.filepath;
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
          <br/>
        <div style={{fontSize: "2em"}}>Movies</div>
          <br/>
          <Dropdown buttonText={"Sort"} options={sort_dropdown_options()}/>
          <br/>
        {
          !edit_entry_view_visible && library_elements.map(element => {
            return(
              <div key={element.filepath}
                   className={"tmdbresult"}
                style={{cursor: "pointer"}}
                onClick={() => launch_video(element.filepath)}
                onContextMenu={(e) => handle_context_menu(e, element)}>
                  <div className={"tmdbresult-splitter"}>
                      <div className={"tmdbresult-img"}>
                          <img src={element.poster_path} alt={element.title}/>
                      </div>
                  <div className={"tmdbresult-info"}>
                    {element.title && element.title}
                    {!element.title && element.filepath}
                      <br/>{format_duration(element.length_in_seconds)}</div>
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
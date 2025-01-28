import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

import { ContextMenu } from "./ContextMenu";
import { EditVideoEntryView } from "./EditVideoEntryView";

export const LibraryView = ({library_id}) => {

  // Load Library
    const [library_elements, set_library_elements] = useState([]);
    const [library_elements_loaded, set_library_elements_loaded] = useState(false)
  
    useEffect(() => {
      if (!library_elements_loaded) {
        set_library_elements_loaded(true);
        invoke("get_library_videos", {library_id: library_id}).then(res => { 
          set_library_elements(res);
          console.log(res);
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

    const get_menu_items = (context) => {
        return [
            { label: 'no impl: Edit', action: () => {set_edit_entry_view_visible(true); close_context_menu();} },
            { label: 'no impl: Show in Windows Explorer', action: () => {close_context_menu();} },
            { label: 'no impl: Remove from Library', action: () => {close_context_menu();} },
            { label: 'no impl: Delete from Storage', action: () => {invoke("call_public"); close_context_menu();}}
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
    
    // Close menu when clicking outside
    useEffect(() => {
      const handleClick = () => close_context_menu();
      document.addEventListener('click', handleClick);
      return () => document.removeEventListener('click', handleClick);
    }, []);
    
  
    const [edit_entry_view_visible, set_edit_entry_view_visible] = useState(false);
    const disable_edit_entry_view = () => {
      set_edit_entry_view_visible(false);
    }





    return (
      <div className="container">
        
        {
          !edit_entry_view_visible && library_elements.map(element => {
            return(
              <div key={element.filepath} 
                style={{cursor: "pointer"}} onClick={() => launch_video(element.filepath)}
                onContextMenu={(e) => handle_context_menu(e, element)}
              >{element.title && element.title}
                {!element.title && element.filepath}</div>
            )
          })
        }

        {context_menu_state.visible && (
            <ContextMenu 
              menu_items={get_menu_items(context_menu_state.context)}
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
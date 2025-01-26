import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

import { ContextMenu } from "./ContextMenu";

export const LibraryView = ({folder_path}) => {

    const [library_elements, set_library_elements] = useState([]);
    const [library_elements_loaded, set_library_elements_loaded] = useState(false)
  
    useEffect(() => {
      if (!library_elements_loaded) {
        set_library_elements_loaded(true);
        invoke("get_video_files", {folder_path: folder_path}).then(res => { 
          set_library_elements(res);
        });
      }
    });


    async function launch_video(full_filepath) {
        invoke("call_public");
        //await invoke("start_video_in_mpc", {filepath: full_filepath});
    }
    

    const [context_menu_state, set_context_menu_state] = useState({
        visible: false,
        position: { x: 0, y: 0 },
        context: null // Store what was clicked
    });

    const get_menu_items = (context) => {
        switch (context?.type) {
          case 'document':
            return [
              { label: 'Open', action: () => {} },
              { label: 'Download', action: () => {} }
            ];
          case 'image':
            return [
              { label: 'Save Image', action: () => {} },
              { label: 'Copy Image', action: () => {} }
            ];
          default:
            return [
              { label: 'Null Image', action: () => {} },
              { label: 'Copy Image', action: () => {} }
            ];
        }
    };

    const handle_context_menu = (event, context) => {
        event.preventDefault();
        set_context_menu_state({
          visible: true,
          position: { x: event.pageX, y: event.pageY },
          context
        });
    };
    
    // Close menu when clicking outside
    useEffect(() => {
      const handleClick = () => set_context_menu_state(prev => ({ ...prev, visible: false }));
      document.addEventListener('click', handleClick);
      return () => document.removeEventListener('click', handleClick);
    }, []);
    
  
  
    return (
      <div className="container">
        
        {
          library_elements.map(element => {
            return(
              <div key={element.filepath} 
                style={{cursor: "pointer"}} onClick={() => launch_video(element)}
                onContextMenu={(e) => handle_context_menu(e, element)}
              >{element.filepath}</div>
            )
          })
        }

        {context_menu_state.visible && (
            <ContextMenu 
              menu_items={get_menu_items(context_menu_state.context)}
              position={context_menu_state.position}
            />
        )}
        
      </div>
    );

}
import {useState, useEffect, useMemo} from "react";
import { invoke } from "@tauri-apps/api/core";

import { Dropdown } from "./Dropdown.jsx";
import { ContextMenu } from "./ContextMenu";

import "./TMDBResults.css";
import { LibraryView } from "./LibraryView.jsx";
import { AddLibrary } from "./AddLibrary.jsx";

export const MainMenu = () => {

    const [libraries, set_libraries] = useState([]);
    const [libraries_loaded, set_libraries_loaded] = useState([]);
    const [init, set_init] = useState(false)
  
    const [opened_library, set_opened_library] = useState("");

    useEffect(() => {
      if (!init) {
        set_init(true);
        invoke("get_available_libraries").then(res => { 
            set_libraries_loaded(true);
            set_libraries(res);
        });
      }
    });

    function reload_libraries() {
        set_init(false);
        set_libraries_loaded(false);
    }

    return(
        <div>
            {!libraries_loaded && <div>Loading...</div>}
            {libraries_loaded && libraries.length == 0 && 
            <div>ryser could not find any libraries</div>}
            {libraries_loaded && libraries.length > 0 && opened_library === "" &&
                libraries.map(library => {
                    return(
                        <div onClick={() => set_opened_library(library.id)}>{library.name}</div>
                    )
                })
            }
            {libraries_loaded && <AddLibrary reload_libraries_fn={reload_libraries}/>}
            {opened_library !== "" && <LibraryView library_id={opened_library} />}
        </div>
    )

}
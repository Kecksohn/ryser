import {useState, useEffect, useMemo} from "react";
import { invoke } from "@tauri-apps/api/core";

import { Dropdown } from "./Dropdown.jsx";
import { ContextMenu } from "./ContextMenu";

import "./TMDBResults.css";
import { LibraryView } from "./LibraryView.jsx";
import { AddLibrary } from "./AddLibrary.jsx";

import { useNavigate } from "react-router-dom";

export const MainMenu = ({init, set_init, libraries_loaded, set_libraries_loaded}) => {

    const [libraries, set_libraries] = useState([]);
    
  
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

    

    const navigate = useNavigate();

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
            {libraries_loaded && <div onClick={() => navigate("/addlibrary")}>Add Library</div>}
            {opened_library !== "" && <LibraryView library_id={opened_library} />}
        </div>
    )

}
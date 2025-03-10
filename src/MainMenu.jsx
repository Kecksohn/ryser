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

    useEffect(() => {
      if (!init) {
        set_init(true);
        invoke("get_available_libraries").then(res => { 
            set_libraries_loaded(true);
            const library_tuples = res.map(library => ({ id: library[0], name: library[1] }));
            set_libraries(library_tuples);
        });
      }
    });

    const navigate = useNavigate();

    return(
        <div>
            {!libraries_loaded && 
                <div>Loading...</div>}
            {libraries_loaded && libraries.length == 0 && 
                <div>ryser could not find any libraries</div>}
            {libraries_loaded && libraries.length > 0 &&
                libraries.map(library => {
                    return(
                        <div key={library.id} onClick={() => navigate("/library/"+library.id)}>{library.name}</div>
                    )
                })
            }
            <br/>
            {libraries_loaded && 
                <div onClick={() => navigate("/addlibrary/")}>Add Library</div>}
        </div>
    )

}
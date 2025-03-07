import { Route, Routes, useNavigate } from "react-router-dom";
import { useState } from "react";

import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';


export const CreateLibrary = ({reload_libraries_fn}) => {

    const navigate = useNavigate();

    const [libraryPaths, setLibraryPaths] = useState([""]);
    
    const updateLibraryPath = (index, newPath) => {
        setLibraryPaths((libraryPaths) =>
            libraryPaths.map((item, i) => (i === index ? newPath : item))
        );
    };

    const addNewLibraryField = () => {
        setLibraryPaths((libraryPaths) => [...libraryPaths, ""]);
    }

    const addLibrariesAtIndex = (index, newElements, overwrite_index = false) => {
        setLibraryPaths((libraryPaths) => [
          ...libraryPaths.slice(0, index),                             // elements before the index
          ...newElements,                                              // new elements to insert
          ...libraryPaths.slice(index + (overwrite_index ? 1 : 0)),    // elements after the index
        ]);
    };

    const removeLibraryAtIndex = (index) => {
        if (libraryPaths.length > 1) {
            setLibraryPaths((libraryPaths) => [
                ...libraryPaths.slice(0, index),
                ...libraryPaths.slice(index + 1),
            ]);
        }
        else { // Always keep one library
            updateLibraryPath(0, ""); 
        }
    };

    async function addLibraryFromFolder(index) {
        const folders = await open({
            multiple: true,
            directory: true,
        });
        if(folders) {
            addLibrariesAtIndex(index, folders, true);
        }
    }

    return(
        <div>
            <span onClick={() => navigate("../")}>Back</span>
            <h2>Create Library</h2>
            <div>Name: <input></input></div>
            {libraryPaths.map((path, i) => {
                return (
                    <div key={i}>
                        Path: 
                        <input value={path} onChange={(e) => updateLibraryPath(i, e.target.value) }></input> 
                        <span onClick={() => addLibraryFromFolder(i)}>Folder Icon </span> 
                        <span onClick={() => removeLibraryAtIndex(i)}>-</span> 
                    </div>
                )
            })}
            {libraryPaths.at(libraryPaths.length-1) !== "" && <div onClick={() => addNewLibraryField()}>+</div>}
        </div>
    )

}
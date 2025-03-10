import { Route, Routes, useNavigate } from "react-router-dom";
import { useState } from "react";

import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';


export const CreateLibrary = () => {

    const navigate = useNavigate();

    const [libraryName, setLibraryName] = useState("");
    const [libraryPaths, setLibraryPaths] = useState([{path: "", include_subdirectories: false }]);
    
    const updateLibraryPath = (index, new_path) => {
        setLibraryPaths((libraryPaths) =>
            libraryPaths.map((item, i) =>
                i === index ? { ...item, path: new_path } : item
            )
        );
    };

    const updateIncludeSubdirectories = (index, updated_include_subdirectories) => {
        console.log(updated_include_subdirectories);
        setLibraryPaths((libraryPaths) =>
            libraryPaths.map((item, i) =>
                i === index ? { ...item, include_subdirectories: updated_include_subdirectories } : item
            )
        );
    }

    const addNewLibraryField = () => {
        setLibraryPaths((libraryPaths) => [...libraryPaths, {path: "", include_subdirectories: false }]);
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
            const paths = folders.map(str => ({ path: str, include_subdirectories: false }));
            addLibrariesAtIndex(index, paths, true);
        }
    }

    async function createLibrary() {
        if (libraryName === "") {
            // TODO: Send Message to GUI
            return;
        }
        if (libraryPaths.every(item => item.path === "")) {
            // TODO: Send Message to GUI
            return;
        }

        await invoke("create_library", {name: libraryName, paths: libraryPaths})
            .then(res => {
                console.log(res);
            });
    } 

    return(
        <div>
            <span onClick={() => navigate("../")}>Back</span>
            <h2>Create Library</h2>
            <div>Name: <input value={libraryName} onChange={(e) => setLibraryName(e.target.value)}></input></div>
            {libraryPaths.map((path, i) => {
                return (
                    <div key={i}>
                        <div>Path: 
                        <input value={path.path} onChange={(e) => updateLibraryPath(i, e.target.value) }></input> 
                        <span onClick={() => addLibraryFromFolder(i)}>Folder Icon </span> 
                        {(i !== 0 || path.path !== "") && <span onClick={() => removeLibraryAtIndex(i)}>-</span>}
                        </div>
                        { path.path !== "" && <div>
                        <input type="checkbox" value={path.include_subdirectories} onChange={(e) => updateIncludeSubdirectories(i, e.target.checked)}/>
                        Include Subdirectories
                        </div>}
                    </div>
                )
            })}
            {libraryPaths.at(libraryPaths.length-1) !== "" && <div onClick={() => addNewLibraryField()}>+</div>}
        
        <button onClick={() => createLibrary()}>Create Library</button>
        </div>
    )

}
import {useState, useEffect, useMemo} from "react";
import { HashRouter, Route, Routes, useNavigate, Outlet } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { Dropdown } from "./Dropdown.jsx";
import { ContextMenu } from "./ContextMenu.jsx";


export const AddLibrary = ({reload_libraries_fn}) => {

    const navigate = useNavigate();

    const [addLibraryClicked, setAddLibraryClicked] = useState(false);



    return(
        <div>
            <h2>Add Library</h2>

            <Routes>
                <Route path="/" element={
                    <>
                        <div onClick={() => navigate("/addlibrary/create")}>Create New Library</div>
                        <div onClick={() => navigate("/addlibrary/import")}>Import Library</div>
                    </>      
                } />
                <Route path="/create" element={
                    <span onClick={() => navigate("/addlibrary")}>Back</span>      
                } />
                <Route path="/import" element={
                    <span onClick={() => navigate("/addlibrary")}>Back</span>      
                } />
            </Routes>
        </div>
    )


}
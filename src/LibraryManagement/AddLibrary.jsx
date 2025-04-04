import { Route, Routes, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { CreateLibrary } from "./CreateLibrary.jsx";

export const AddLibrary = () => {

    const navigate = useNavigate();

    return(
        <div>
            <Routes>
                <Route path="/" element={
                    <>
                        <span onClick={() => navigate("../")}>Back</span>
                        <h2>Add Library</h2>
                        <div onClick={() => navigate("/addlibrary/create")}>Create New Library</div>
                        <div onClick={() => navigate("/addlibrary/import")}>Import Library</div>
                    </>      
                } />
                <Route path="/create" element={
                    <CreateLibrary/>    
                } />
                <Route path="/import" element={
                    <>
                        <span onClick={() => navigate("../")}>Back</span>
                        <h2>Import Library</h2>
                        <span>soooooon</span>
                    </>     
                } />
            </Routes>
        </div>
    )


}
import {useState, useEffect, useMemo} from "react";
import { invoke } from "@tauri-apps/api/core";

import { Dropdown } from "./Dropdown.jsx";
import { ContextMenu } from "./ContextMenu.jsx";

export const AddLibrary = ({reload_libraries_fn}) => {

    const [addLibraryClicked, setAddLibraryClicked] = useState(false);

    return(
        <div>
        {!addLibraryClicked && <span onClick={() => setAddLibraryClicked(true)}>Add Library</span>}
        {addLibraryClicked && <><span>Create New Library</span><span>Import Library</span></>}
        </div>
    )


}
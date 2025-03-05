import {useState, useEffect, useMemo} from "react";
import { invoke } from "@tauri-apps/api/core";

import { Dropdown } from "./Dropdown.jsx";
import { ContextMenu } from "./ContextMenu.jsx";

export const AddLibrary = ({reload_libraries_fn}) => {

    const [addLibraryClicked, setAddLibraryClicked] = useState(false);

    return(
        <div>
         Hello pls add library
        </div>
    )


}
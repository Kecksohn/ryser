import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { useContextMenu } from "../UITools/ContextMenu.jsx";

export const MainMenu = () => {
  const navigate = useNavigate();
  const { useContextMenuOn } = useContextMenu();

  const [libraries_loaded, set_libraries_loaded] = useState(false);
  const [libraries, set_libraries] = useState([]);

  useEffect(() => {
    if (!libraries_loaded) {
      invoke("get_available_libraries").then((res) => {
        set_libraries_loaded(true);
        const library_tuples = res.map((library) => ({
          id: library[0],
          name: library[1],
        }));
        set_libraries(library_tuples);
      });
    }
  }),
    [libraries_loaded];

  function update_libraries() {
    invoke("rescan_all_libraries").then((res) => {
      set_libraries_loaded(false);
    });
  }

  const library_context_menu_options = (context) => [
    {
      label: "no impl: Edit",
      action: () => {},
    },
    {
      label: "no impl: Rescan ALL",
      action: () => {
        update_libraries();
      },
    },
    {
      label: "no impl: Export",
      action: () => {},
    },
    {
      label: "Delete",
      action: () => {
        invoke("delete_library_gui", { library_id: context.id })
          .then(() => {
            set_libraries_loaded(false);
          })
          .catch((error) => {
            console.log(error);
          });
      },
    },
  ];

  return (
    <div>
      {!libraries_loaded && <div>Loading...</div>}
      {libraries_loaded && libraries.length == 0 && (
        <div>ryser could not find any libraries</div>
      )}
      {libraries_loaded &&
        libraries.length > 0 &&
        libraries.map((library) => {
          return (
            <div
              key={library.id}
              onClick={() => navigate("/library/" + library.id)}
              {...useContextMenuOn(library, library_context_menu_options)}
            >
              {library.name}
            </div>
          );
        })}
      <br />
      {libraries_loaded && (
        <>
          <div onClick={() => navigate("/addlibrary/")}>Add Library</div>
          <br />

          {/* <div onClick={() => updateLibraries()}>Update Libraries</div>*/}
        </>
      )}
    </div>
  );
};

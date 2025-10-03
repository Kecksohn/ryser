import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { useContextMenu } from "../UITools/ContextMenu.jsx";

import { HeaderBar } from "../UIElements/HeaderBar.jsx";

import mainMenuStyles from "./MainMenu.module.css";

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
    invoke("rescan_all_libraries_gui").then((res) => {
      set_libraries_loaded(false);
    });
  }

  function reparse_libraries() {
    invoke("reparse_all_libraries_preserve_covers_gui").then((res) => {
      console.log("Libraries reparsed successfully");
      set_libraries_loaded(false); // Refresh the view
    }).catch((error) => {
      console.error("Error reparsing libraries:", error);
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
      {/* Header */}
      <HeaderBar
        leftside_text={
          <span>
            <span className={"headerbar-title"}>ryser</span>
          </span>
        }
      />

      {/* Loading & Error Message */}
      {!libraries_loaded && <div>Loading...</div>}
      {libraries_loaded && libraries.length == 0 && (
        <div>ryser could not find any libraries</div>
      )}

      {/* Libraries On Disk*/}
      <div className={mainMenuStyles.librariesContainer}>
        {libraries_loaded &&
          libraries.length > 0 &&
          libraries.map((library) => {
            return (
              <div
                key={library.id}
                className={mainMenuStyles.libraryElement}
                onClick={() => navigate("/library/" + library.id)}
                {...useContextMenuOn(library, library_context_menu_options)}
              >
                {library.name}
              </div>
            );
          })}
      </div>

      {/* Add/Update Libraries*/}
      {libraries_loaded && (
        <div className={mainMenuStyles.libraryManagementContainer}>
          <div
            className={`${mainMenuStyles.libraryElement} ${libraries.length > 0 ? mainMenuStyles.scaledButton : ''}`}
            onClick={() => navigate("/addlibrary/")}
          >
            Add Library
          </div>

          {libraries.length > 0 && (
            <>
              <div
                className={`${mainMenuStyles.libraryElementAdd} ${mainMenuStyles.scaledButton}`}
                onClick={() => update_libraries()}
              >
                Update Libraries
              </div>

              <div
                className={`${mainMenuStyles.libraryElementAdd} ${mainMenuStyles.scaledButton}`}
                onClick={() => reparse_libraries()}
              >
                Reparse Libraries
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
};

import React from "react";
import { useState, useEffect } from "react";

import "@/styles/global.css";
import "@/styles/colors.css";

import { ScaleWrapperGlobal } from "./UITools/ScaleWrapper.jsx";
import { ContextMenuProvider } from "./UITools/ContextMenu.jsx";
import { NotificationManager } from "./UITools/NotificationManager.jsx";
import { HashRouter, Route, Routes } from "react-router-dom";

import { MainMenu } from "./MainMenu/MainMenu.jsx";
import { LibraryView } from "./LibraryView/LibraryView.jsx";
import { AddLibrary } from "./LibraryManagement/AddLibrary.jsx";

function App() {
  return (
    <>
      <ScaleWrapperGlobal>
        <ContextMenuProvider>
          <NotificationManager />

          <HashRouter>
            <Routes>
              <Route path="/" element={<MainMenu />} />
              <Route path="/library/:library_id/*" element={<LibraryView />} />
              <Route path="/addlibrary/*" element={<AddLibrary />} />
              <Route path="/settings/*" element={<MainMenu />} />
            </Routes>
          </HashRouter>
        </ContextMenuProvider>
      </ScaleWrapperGlobal>
    </>
  );
}

export default App;

import React from "react";
import { useState, useEffect } from "react";

import { ScaleProvider } from "./components/UITools/ScaleProvider.jsx";
import { ContextMenuProvider } from "./components/UITools/ContextMenu.jsx";
import { NotificationManager } from "./components/UITools/NotificationManager.jsx";

import { HashRouter, Route, Routes } from "react-router-dom";
import { MainMenu } from "./components/MainMenu/MainMenu.jsx";
import { LibraryView } from "./components/LibraryView/LibraryView.jsx";
import { AddLibrary } from "./components/LibraryManagement/AddLibrary.jsx";

import "./styles/global.css";

function App() {
  return (
    <>
      <ScaleProvider>
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
      </ScaleProvider>
    </>
  );
}

export default App;

import React from "react";
import { useState, useEffect } from "react";

import { NotificationManager } from "./NotificationManager";

import { HashRouter, Route, Routes } from "react-router-dom";
import { MainMenu } from "./MainMenu";
import { LibraryView } from "./LibraryView";
import { AddLibrary } from "./AddLibrary";

import "./App.css";


function App() {

  return(<>

    <NotificationManager/>

    <HashRouter>
      <Routes>
        <Route path="/" element={ 
          <MainMenu /> 
        } />
        <Route path="/library/:library_id" element={
          <LibraryView />
        } />
        <Route path="/addlibrary/*" element={
          <AddLibrary/>
        } />
        <Route path="/settings" element={<MainMenu />} />
      </Routes>
    </HashRouter>
    
  </>)
  
}

export default App;

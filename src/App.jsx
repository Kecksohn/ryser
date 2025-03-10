import React from "react";
import { useState, useEffect } from "react";

import { HashRouter, Route, Routes } from "react-router-dom";
import { MainMenu } from "./MainMenu";
import { LibraryView } from "./LibraryView";
import { AddLibrary } from "./AddLibrary";

import "./App.css";


function App() {

  const [libraries_loaded, set_libraries_loaded] = useState([]);
  const [init, set_init] = useState(false);

  function reload_libraries() {
    set_init(false);
    set_libraries_loaded(false);
  }

  return(
    <HashRouter>
      <Routes>
        <Route path="/" element={ 
          <MainMenu init={init} set_init={set_init} 
                    libraries_loaded={libraries_loaded} set_libraries_loaded={set_libraries_loaded}/> 
        } />
        <Route path="/library/:library_id" element={
          <LibraryView />
        } />
        <Route path="/addlibrary/*" element={
          <AddLibrary reload_libraries_fn={reload_libraries}/>
        } />
        <Route path="/settings" element={<MainMenu />} />
      </Routes>
    </HashRouter>
  )
  
}

export default App;

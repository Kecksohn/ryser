import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { LibraryView } from "./LibraryView";


function App() {

  return(
    <LibraryView folder_path="F:\\mov"/>
  )
  
}

export default App;

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { MainMenu } from "./MainMenu";


function App() {

  return(
    <MainMenu />
  )
  
}

export default App;

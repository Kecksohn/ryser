import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import {
  warn,
  debug,
  trace,
  info,
  error,
  attachConsole,
  attachLogger,
} from '@tauri-apps/plugin-log';


function App() {

  const [library_elements, set_library_elements] = useState([]);
  const [library_elements_loaded, set_library_elements_loaded] = useState(false)

  useEffect(() => {
    if (!library_elements_loaded) {
      set_library_elements_loaded(true);
      invoke("get_video_files").then(res => { 
        set_library_elements(res);
        setGreetMsg(res[0]);
      });
    }
  });

  return (
    <main className="container">
      
      {
        library_elements.map(element => {
          return(
            <p>{element}</p>
          )
        })
      }
      
    </main>
  );
}

/*
<form
className="row"
onSubmit={(e) => {
  e.preventDefault();
  greet();
}}
>
<input
  id="greet-input"
  onChange={(e) => setName(e.currentTarget.value)}
  placeholder="Enter a name..."
/>
<button type="submit">Greet</button>
</form>
*/

export default App;

import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import { Command } from '@tauri-apps/plugin-shell';

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
      });
    }
  });

  async function launch_video() {
    let result = await Command.create('exec-sh', ["C:Program Files (x86)/K-Lite Codec Pack/MPC-HC64/mpc-hc64.exe"]).execute();
    console.log(result);
    await invoke("start_video_in_mpc");
  }



  return (
    <main className="container">
      
      {
        library_elements.map(element => {
          return(
            <div key={element} style={{cursor: "pointer"}} onClick={() => launch_video()}
            >{element}</div>
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

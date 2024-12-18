import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";


function App() {

  const [library_elements, set_library_elements] = useState([]);
  const [library_elements_loaded, set_library_elements_loaded] = useState(false)

  useEffect(() => {
    if (!library_elements_loaded) {
      set_library_elements_loaded(true);
      invoke("get_video_files", {folder_path: "F:\\mov"}).then(res => { 
        set_library_elements(res);
      });
    }
  });

  async function launch_video(full_filepath) {
    await invoke("start_video_in_mpc", {filepath: full_filepath});
  }



  return (
    <main className="container">
      
      {
        library_elements.map(element => {
          return(
            <div key={element.filepath} style={{cursor: "pointer"}} onClick={() => launch_video(element)}
            >{element.filepath}</div>
          )
        })
      }
      
    </main>
  );
}

export default App;

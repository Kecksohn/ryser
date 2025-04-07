import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

import { invoke } from "@tauri-apps/api/core";

document.addEventListener("DOMContentLoaded", () => {
  // This will wait for the window to load, but you could
  // run this function on whatever trigger you want
  invoke("open_window");
});

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import "./HeaderBar.css";
import react_icon from "../assets/react.svg";

export const HeaderBar = ({
  leftside_text,
  rightside_text,
  back_link,
  settings_link,
}) => {
  const navigate = useNavigate();

  return (
    <div className="header-container">
      <div className="header-section">
        {back_link && (
          <img
            src={react_icon}
            onClick={() => navigate(back_link)}
            alt="Back"
          />
        )}
        {leftside_text && (
          <span style={{ marginLeft: 10 }}>{leftside_text}</span>
        )}
      </div>
      <div className="header-section">
        {rightside_text && <span>{rightside_text}</span>}
        {<img src={react_icon} onClick={() => {}} alt="Toggle Fullscreen" />}
        {settings_link && (
          <img
            src={react_icon}
            onClick={() => navigate(settings_link)}
            alt="Open Library Settings"
          />
        )}
      </div>
    </div>
  );
};

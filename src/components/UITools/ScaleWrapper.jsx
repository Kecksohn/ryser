import React, { useContext } from "react";
import { ScaleContext } from "./ScaleProvider";

export const useScale = () => {
  const context = useContext(ScaleContext);
  if (!context) {
    throw new Error("useScale must be used within ScaleProvider");
  }
  return context;
};

export const ScaleWrapper = ({
  children,
  componentScale = 1,
  className = "",
  style = {},
}) => {
  const wrapperStyle = {
    "--component-scale": componentScale,
    ...style,
  };

  return (
    <div className={`scalable-wrapper ${className}`} style={wrapperStyle}>
      {children}
    </div>
  );
};

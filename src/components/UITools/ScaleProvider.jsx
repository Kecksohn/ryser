import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
} from "react";

import "@/styles/scaling.css";

const ScaleContext = createContext();

export const ScaleProvider = ({ children }) => {
  const [globalScale, setGlobalScale] = useState(1);
  const [componentScales, setComponentScales] = useState({});

  // Apply global scale to CSS custom property
  useEffect(() => {
    document.documentElement.style.setProperty("--global-scale", globalScale);
  }, [globalScale]);

  // Apply component-specific scales
  useEffect(() => {
    Object.entries(componentScales).forEach(([component, scale]) => {
      document.documentElement.style.setProperty(`--${component}-scale`, scale);
    });
  }, [componentScales]);

  // Memoize this function to prevent infinite re-renders
  const setComponentScale = useCallback((componentName, scale) => {
    setComponentScales((prev) => ({
      ...prev,
      [componentName]: scale,
    }));
  }, []);

  return (
    <ScaleContext.Provider
      value={{
        globalScale,
        setGlobalScale,
        componentScales,
        setComponentScale,
      }}
    >
      {children}
    </ScaleContext.Provider>
  );
};

export const getScaleContext = () => {
  const context = useContext(ScaleContext);
  if (!context) {
    throw new Error("useScale must be used within ScaleProvider");
  }
  return context;
};

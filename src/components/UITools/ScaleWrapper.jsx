/* 
  In your component put:
  
  export const ComponentName = ({ scale = 1 }) => {
    const { setComponentScale } = useScale();
    useEffect(() => {
      setComponentScale("unique-component-name", scale);
    }, [scale, setComponentScale]);
 
  Then warp your returned component like:
  
  return (
    <ScaleWrapper componentScale={scale} (optional className for whole component)>
      <Your Component/>
    </ScaleWrapper>
  );

  Now your ComponentName.module.css can (and should!) use all variables in /styles/scale.css
  as well as --unique-component-name-scale to calculate its sizes

*/

import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
} from "react";

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

// Scale Wrapper Global (Put a singleton of this in your App.jsx)
import "@/styles/scaling.css";

const ScaleContext = createContext();

export const ScaleWrapperGlobal = ({ children }) => {
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

export const useScale = () => {
  const context = useContext(ScaleContext);
  if (!context) {
    throw new Error("useScale must be used within ScaleProvider");
  }
  return context;
};

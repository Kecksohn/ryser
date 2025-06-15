import React, { useContext } from "react";
import { getScaleContext } from "./ScaleProvider";

/* 
  In your component put:
  
  export const ComponentName = ({ scale = 1 }) => {
    const { setComponentScale } = useScale();
    useEffect(() => {
      setComponentScale("unique-component-name", scale);
    }, [scale, setComponentScale]);
*/
export const useScale = () => getScaleContext();

/* 
  Then warp your returned component like:
  
  return (
    <ScaleWrapper componentScale={scale} (optional className for whole component)>
      <Your Component/>
    </ScaleWrapper>
  );
*/
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

/*
  Now your ComponentName.module.css can (and should!) use all variables in /styles/scale.css
  as well as --unique-component-name-scale to calculate its sizes
*/

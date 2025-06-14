import { useScale, ScaleWrapper } from "../UITools/ScaleWrapper";
import { useState, useEffect, useRef } from "react";
import "./Dropdown.css";

export const Dropdown = ({ buttonText, options, scale = 1 }) => {
  const [isOpen, setIsOpen] = useState(false);
  const { setComponentScale } = useScale();
  const closeTimeoutRef = useRef(null);

  useEffect(() => {
    setComponentScale("dropdown", scale);
  }, [scale, setComponentScale]);

  const handleMouseEnter = () => {
    // Clear any pending close timeout
    if (closeTimeoutRef.current) {
      clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = null;
    }
    setIsOpen(true);
  };

  const handleMouseLeave = () => {
    // Add a small delay before closing to allow mouse movement between elements
    closeTimeoutRef.current = setTimeout(() => {
      setIsOpen(false);
    }, 150); // 150ms delay
  };

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (closeTimeoutRef.current) {
        clearTimeout(closeTimeoutRef.current);
      }
    };
  }, []);

  return (
    <ScaleWrapper componentScale={scale} className="dropdown-container">
      <div
        className="dropdown-container"
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      >
        <button className="dropdown-button">{buttonText}</button>
        {isOpen && (
          <div className="dropdown-menu">
            {options.map((option, index) => (
              <button
                key={index}
                className="dropdown-menu-item"
                onClick={() => option.onClick()}
              >
                {option.label}
              </button>
            ))}
          </div>
        )}
      </div>
    </ScaleWrapper>
  );
};

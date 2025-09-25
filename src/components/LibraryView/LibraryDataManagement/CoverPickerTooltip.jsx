import React, { useRef, useEffect, cloneElement } from "react";
import styles from "./CoverPickerTooltip.module.css";

export const CoverPickerTooltip = ({
  children,
  images = [], // Array of image sources
  selectedImage = null, // Optional: currently selected image
  onImageSelect = () => {}, // Optional: callback when image is selected
  additionalContent = null, // Optional content to display below the images
  position = "bottom",
  isOpen,
  setIsOpen,
}) => {
  const tooltipRef = useRef(null);

  // Close tooltip when clicking outside
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (tooltipRef.current && !tooltipRef.current.contains(event.target)) {
        setIsOpen(false);
      }
    };
    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen, setIsOpen]);

  // Reference to the scroll container
  const scrollContainerRef = useRef(null);

  // Toggle tooltip on click
  const handleToggle = (e) => {
    setIsOpen(!isOpen);
  };

  // Handle image selection
  const handleImageClick = (imageSrc) => {
    onImageSelect(imageSrc);
  };

  // Clone the child element and add our click handler
  const triggerElement = cloneElement(children, {
    onClick: (e) => {
      // Call the original onClick if it exists
      if (children.props.onClick) {
        children.props.onClick(e);
      }
      handleToggle(e);
    },
    ref: tooltipRef,
  });

  // Get position-specific class name using camelCase
  const positionClass = `tooltip${position.charAt(0).toUpperCase() + position.slice(1)}`;

  // Construct className using CSS modules
  const tooltipClassName = [
    styles.tooltip,
    styles[positionClass],
    isOpen ? styles.active : "",
  ]
    .filter(Boolean)
    .join(" ");

  // Set up wheel event listener for horizontal scrolling
  useEffect(() => {
    const scrollContainer = scrollContainerRef.current;

    if (scrollContainer && isOpen) {
      const handleWheel = (e) => {
        // Prevent the default vertical scroll
        e.preventDefault();

        // Scroll horizontally based on the wheel delta
        scrollContainer.scrollLeft += e.deltaY;
      };

      // Add event listener with passive: false to ensure preventDefault works
      scrollContainer.addEventListener("wheel", handleWheel, {
        passive: false,
      });

      return () => {
        scrollContainer.removeEventListener("wheel", handleWheel);
      };
    }
  }, [isOpen]);

  return (
    <div className={styles.tooltipContainer} ref={tooltipRef}>
      {triggerElement}

      <div className={tooltipClassName}>
        {/* Horizontal scrolling image container */}
        <div className={styles.imageScroll} ref={scrollContainerRef}>
          {images.map((imageSrc, index) => (
            <div
              key={`${imageSrc}-${index}`}
              className={`${styles.imageContainer} ${selectedImage === imageSrc ? styles.selected : ""}`}
              onClick={() => handleImageClick(imageSrc)}
            >
              <img
                src={imageSrc}
                alt={`Cover option ${index + 1}`}
                className={styles.coverImage}
              />
            </div>
          ))}
        </div>

        {/* Additional content area below images */}
        {additionalContent && (
          <div className={styles.additionalContent}>{additionalContent}</div>
        )}
      </div>
    </div>
  );
};

export default CoverPickerTooltip;

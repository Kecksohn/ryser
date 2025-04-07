import React, {
  useEffect,
  useRef,
  useContext,
  createContext,
  useState,
} from "react";
import ReactDOM from "react-dom";

import "./ContextMenu.css";

// Create a context to manage the menu state globally
const ContextMenuContext = createContext(null);

// Provider component that will wrap your application or a section of it
export const ContextMenuProvider = ({ children }) => {
  const [menuState, setMenuState] = useState({
    visible: false,
    position: { x: 0, y: 0 },
    context: null,
    items: [],
  });

  const showMenu = (x, y, context, items) => {
    setMenuState({
      visible: true,
      position: { x, y },
      context,
      items,
    });
  };

  const hideMenu = () => {
    setMenuState((prev) => ({ ...prev, visible: false }));
  };

  // Close menu when clicking anywhere
  useEffect(() => {
    const handleClick = () => hideMenu();
    document.addEventListener("click", handleClick);
    return () => document.removeEventListener("click", handleClick);
  }, []);

  return (
    <ContextMenuContext.Provider value={{ menuState, showMenu, hideMenu }}>
      {children}
      <ContextMenuComponent />
    </ContextMenuContext.Provider>
  );
};

// Custom hook to use the context menu
export const useContextMenu = () => {
  const context = useContext(ContextMenuContext);

  if (!context) {
    throw new Error("useContextMenu must be used within a ContextMenuProvider");
  }

  const { showMenu, hideMenu } = context;

  // Helper function to create context menu handlers
  const createContextMenuHandler = (context, getItems) => (event) => {
    event.preventDefault();
    const items = typeof getItems === "function" ? getItems(context) : getItems;
    showMenu(event.clientX, event.clientY, context, items);
  };

  return {
    // Attach to an element: useContextMenuOn(element, menuItems)
    useContextMenuOn: (context, getItems) => ({
      onContextMenu: createContextMenuHandler(context, getItems),
    }),
    // Manually show or hide the menu
    showContextMenu: showMenu,
    hideContextMenu: hideMenu,
  };
};

// The actual menu component (internal to the system)
const ContextMenuComponent = () => {
  const { menuState, hideMenu } = useContext(ContextMenuContext);
  const menuRef = useRef(null);

  // Adjust menu position if it would render outside viewport
  useEffect(() => {
    if (!menuRef.current || !menuState.visible) return;

    const menu = menuRef.current;
    const menuRect = menu.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    let adjustedX = menuState.position.x;
    let adjustedY = menuState.position.y;

    // Adjust X position if menu goes beyond right edge
    if (menuState.position.x + menuRect.width > viewportWidth) {
      adjustedX = viewportWidth - menuRect.width;
    }

    // Adjust Y position if menu goes beyond bottom edge
    if (menuState.position.y + menuRect.height > viewportHeight) {
      adjustedY = viewportHeight - menuRect.height;
    }

    // Directly set style on the element
    menu.style.left = `${adjustedX}px`;
    menu.style.top = `${adjustedY}px`;
  }, [menuState.visible, menuState.position]);

  const handleMenuClick = (event, menuItem) => {
    event.stopPropagation();
    menuItem.action();
    if (menuItem.close_after !== false) {
      hideMenu();
    }
  };

  if (!menuState.visible) {
    return null;
  }

  return ReactDOM.createPortal(
    <div
      ref={menuRef}
      className="context-menu"
      style={{
        left: `${menuState.position.x}px`,
        top: `${menuState.position.y}px`,
      }}
    >
      {menuState.items.map((menuItem, index) => (
        <div
          key={index}
          onClick={(e) => handleMenuClick(e, menuItem)}
          className="context-menu-item"
        >
          {menuItem.icon && <span className="w-4 h-4">{menuItem.icon}</span>}
          {menuItem.label}
        </div>
      ))}
    </div>,
    document.body,
  );
};

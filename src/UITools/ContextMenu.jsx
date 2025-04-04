import React, { useEffect, useRef } from 'react';
import ReactDOM from 'react-dom';

import "./ContextMenu.css"

export const ContextMenu = ({ menu_items, position }) => {
  const menu_ref = useRef(null);

  // Adjust menu position if it would render outside viewport
  useEffect(() => {
    if (!menu_ref.current) return;
    
    const menu = menu_ref.current;
    const menu_rect = menu.getBoundingClientRect();
    const viewport_width = window.innerWidth;
    const viewport_height = window.innerHeight;

    let adjusted_x = position.x;
    let adjusted_y = position.y;

    // Adjust X position if menu goes beyond right edge
    if (position.x + menu_rect.width > viewport_width) {
      adjusted_x = viewport_width - menu_rect.width;
    }

    // Adjust Y position if menu goes beyond bottom edge
    if (position.y + menu_rect.height > viewport_height) {
      adjusted_y = viewport_height - menu_rect.height;
    }

    // Directly set style on the element
    menu.style.left = `${adjusted_x}px`;
    menu.style.top = `${adjusted_y}px`;
  }, [position]);

  const handle_menu_click = (event, item_action) => {
    event.stopPropagation();
    item_action();
  };

  return ReactDOM.createPortal(
    <div
      ref={menu_ref}
      className="context-menu"
      style={{
        left: `${position.x}px`,
        top: `${position.y}px`,
      }}
    >
      {menu_items.map((menu_item, index) => (
        <div
          key={index}
          onClick={(e) => {handle_menu_click(e, menu_item.action);}}
          className="context-menu-item"
        >
          {menu_item.icon && <span className="w-4 h-4">{menu_item.icon}</span>}
          {menu_item.label}
        </div>
      ))}
    </div>,
    document.body
  );
};
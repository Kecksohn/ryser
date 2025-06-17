import React from "react";
import { useState, useEffect } from "react";

import { listen } from "@tauri-apps/api/event";

import { ReactNotifications } from "react-notifications-component";
import { Store } from "react-notifications-component";
import "react-notifications-component/dist/theme.css";
import "./NotificationManager.css";

export const NotificationManager = () => {
  // notifications need at least field 'js_notifcation_id', and optionally 'rust_notification_id'
  const [notifications, set_notifications] = useState([]);

  useEffect(() => {
    // Set-Up Rust Listener
    const unlistenPromise = listen("display-message", (event) => {
      const { header, message, id, duration_ms } = event.payload;
      create_notification(header, message, id, duration_ms);
    });

    // Runs when component unmounts
    return () => {
      unlistenPromise.then((unlistenFn) => unlistenFn());
    };
  }, []);

  function create_notification(
    header,
    message,
    rust_notification_id,
    duration_ms = 3000
  ) {
    const js_notification_id = Store.addNotification({
      title: header,
      message: message,
      type: "default",
      insert: "bottom",
      container: "bottom-right",
      animationIn: ["animate__animated", "animate__fadeIn"],
      animationOut: ["animate__animated", "animate__fadeOut"],
      dismiss: {
        duration: duration_ms,
        pauseOnHover: true,
        onScreen: false,
      },
      slidingEnter: {
        duration: 350,
        timingFunction: "ease-in",
        delay: 0,
      },
      slidingExit: {
        duration: 350,
        timingFunction: "ease-out",
        delay: 0,
      },
      onRemoval: (id, removed_by) => {
        set_notifications((prev) =>
          prev.filter((notification) => notification.js_notification_id !== id)
        );
      },
    });

    set_notifications((prev) => [
      ...prev,
      {
        js_notification_id: js_notification_id,
        rust_notification_id: rust_notification_id,
      },
    ]);
  }

  return <ReactNotifications />;
};

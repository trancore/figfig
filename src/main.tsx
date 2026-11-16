import React, { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Popup } from "./Popup";
import { emptyPopup, type CompletionResponse } from "./types";
import "./styles.css";

document.body.classList.add("popup-body");
const browserPreview = import.meta.env.DEV
  && new URLSearchParams(location.search).get("preview") === "1";
const demoPopup: CompletionResponse = {
  ...emptyPopup,
  visible: true,
  candidates: [
    {
      label: "checkout",
      description: "Switch branches or restore files",
      kind: "subcommand",
      insertText: "checkout",
      appendSpace: true,
    },
    {
      label: "cherry-pick",
      description: "Apply changes from existing commits",
      kind: "subcommand",
      insertText: "cherry-pick",
      appendSpace: true,
    },
  ],
};

function App() {
  const [completion, setCompletion] = useState(browserPreview ? demoPopup : emptyPopup);
  useEffect(() => {
    if (browserPreview) return;
    let disposed = false;
    let receivedEvent = false;
    let unlisten: (() => void) | undefined;
    void (async () => {
      const stop = await listen<CompletionResponse>("completion", (event) => {
        receivedEvent = true;
        if (!disposed) setCompletion(event.payload);
      });
      if (disposed) {
        stop();
        return;
      }
      unlisten = stop;
      const initial = await invoke<CompletionResponse>("get_completion");
      if (!disposed && !receivedEvent) setCompletion(initial);
    })().catch(() => {
      if (!disposed) setCompletion(emptyPopup);
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  return <Popup completion={completion} />;
}

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";

// Prevent WebKit from handling dropped files (triggers GStreamer errors).
// Only intercept OS file drops (dataTransfer contains "Files").
// Must use capture phase to intercept before WebKit's native media handling.
for (const evt of ["dragenter", "dragover", "drop"] as const) {
  document.addEventListener(
    evt,
    (e) => {
      if ((e as DragEvent).dataTransfer?.types?.includes("Files")) {
        e.preventDefault();
        e.stopPropagation();
      }
    },
    { capture: true },
  );
}

// Keep global errors visible in logs without destroying the UI state.
window.onerror = (msg, src, line, col, err) => {
  console.error("[GlobalError]", {
    msg,
    src,
    line,
    col,
    stack: err?.stack,
  });
  return false;
};
window.onunhandledrejection = (e) => {
  console.error("[UnhandledRejection]", {
    reason: e.reason,
    stack: e.reason?.stack,
  });
};

try {
  const app = mount(App, {
    target: document.getElementById("app")!,
  });
  // @ts-ignore
  window.__app = app;
} catch (e: any) {
  const message = e instanceof Error ? `${e.message}\n${e.stack || ""}` : String(e);
  document.body.innerHTML = `<pre style="color:red;padding:2em;white-space:pre-wrap">MOUNT ERROR: ${message}</pre>`;
}

export default {};

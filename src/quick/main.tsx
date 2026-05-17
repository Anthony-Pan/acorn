import React from "react";
import ReactDOM from "react-dom/client";

import { QuickApp } from "./App";
import "../index.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("Missing #root element in quick.html");
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <QuickApp />
  </React.StrictMode>,
);

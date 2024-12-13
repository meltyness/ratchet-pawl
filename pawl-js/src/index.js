import React, { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

import App from "./App";
import HomePanel from "./HomePanel"

const root = createRoot(document.getElementById("root"));
root.render(
  <StrictMode>
    <title>ratchet configuration and diagnostics</title>
    {/* <App />*/}
    <HomePanel />
  </StrictMode>
);
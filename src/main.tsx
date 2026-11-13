// SPDX-License-Identifier: RPL-1.5
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles/tokens.css";
import "./styles/app.css";
import "./styles/gdm.css";
import "./styles/tooltip.css";
import "./i18n";
import App from "./App";
import { TooltipLayer } from "./Tooltip";
import { boot } from "./store";

void boot().then(() =>
  createRoot(document.getElementById("root") as HTMLElement).render(
    <StrictMode>
      <App />
      <TooltipLayer />
    </StrictMode>,
  ),
);

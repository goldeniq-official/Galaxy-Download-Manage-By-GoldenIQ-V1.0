// SPDX-License-Identifier: RPL-1.5
// Drop basket (IDM "drop target"): a 76x76 always-on-top window. Links dropped on it go to add_url, one by one
// (at most 50 per drop, http/https only); the window may only do that and hide itself (capabilities/basket.json).
import { StrictMode, useState, type DragEvent } from "react";
import { createRoot } from "react-dom/client";
import { useTranslation } from "react-i18next";
import "./styles/tokens.css";
import "./styles/basket.css";
import "./i18n";
import { api, droppedLinks } from "./api";
import { Icon } from "./ui";

function Basket() {
  const { t } = useTranslation();
  const [over, setOver] = useState(false);
  const drop = async (e: DragEvent) => {
    e.preventDefault();
    setOver(false);
    for (const u of droppedLinks(e.dataTransfer.getData("text/uri-list"), e.dataTransfer.getData("text/plain"))) await api.addUrl(u).catch(() => {});
  };
  return (
    <div
      className={"bk" + (over ? " over" : "")}
      data-tauri-drag-region
      title={t("basket.label")}
      onDragOver={(e) => (e.preventDefault(), setOver(true))}
      onDragLeave={() => setOver(false)}
      onDrop={(e) => void drop(e)}
    >
      <img src="/gdm-icon-256.png" alt={t("basket.label")} width={40} height={40} draggable={false} data-tauri-drag-region />
      <button type="button" className="fx bk-x" aria-label={t("basket.hide")} title={t("basket.hide")} onClick={() => void api.basketShow(false)}>
        <Icon n="x" />
      </button>
    </div>
  );
}

createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <Basket />
  </StrictMode>,
);

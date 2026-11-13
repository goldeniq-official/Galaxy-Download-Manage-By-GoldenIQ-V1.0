// SPDX-License-Identifier: RPL-1.5
// Small primitives. State is shown by fill and colour only (owner rule 8, D6): no outline, ring or glow.
import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent as RKE, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { PH } from "./icons";

export function Icon({ n, lg, fill }: { n: string; lg?: boolean; fill?: boolean }) {
  const d = PH[fill && PH[n + "#fill"] ? n + "#fill" : n] ?? [];
  return (
    <svg className={"ico" + (lg ? " lg" : "")} viewBox="0 0 256 256" aria-hidden="true" focusable="false">
      {d.map((p, i) => (
        <path key={i} d={p} />
      ))}
    </svg>
  );
}

export function Toggle({ on, label, onChange, disabled }: { on: boolean; label: string; onChange: (v: boolean) => void; disabled?: boolean }) {
  return <button type="button" className="fx toggle" role="switch" aria-checked={on} aria-label={label} disabled={disabled} onClick={() => onChange(!on)} />;
}

export function Tabs<T extends string>({ value, items, label, onChange }: { value: T; items: [T, string][]; label: string; onChange: (v: T) => void }) {
  return (
    <div className="tabs" role="radiogroup" aria-label={label}>
      {items.map(([k, text]) => (
        <button key={k} type="button" className="fx tab" role="radio" aria-checked={value === k} onClick={() => onChange(k)}>
          {text}
        </button>
      ))}
    </div>
  );
}

/** modal with focus moved in on open and returned on close (P1 #22), Esc closes */
export function Modal({ label, onClose, children, wide }: { label: string; onClose: () => void; children: ReactNode; wide?: boolean }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const back = document.activeElement as HTMLElement | null;
    const app = document.getElementById("shell");
    app?.setAttribute("inert", "");
    const first = ref.current?.querySelector<HTMLElement>("input, select, textarea, button");
    first?.focus();
    return () => {
      app?.removeAttribute("inert");
      back?.focus?.();
    };
  }, []);
  return (
    <div className="scrim" onKeyDown={(e) => e.key === "Escape" && onClose()}>
      <div ref={ref} className={"modal" + (wide ? " wide" : "")} role="dialog" aria-modal="true" aria-label={label}>
        {children}
        <span tabIndex={0} onFocus={() => ref.current?.querySelector<HTMLElement>("input, select, textarea, button:not([disabled])")?.focus()} />
      </div>
    </div>
  );
}

export const ftClass = (cat: string) => ({ compressed: "zip", video: "vid", music: "aud", documents: "doc", programs: "app" })[cat] ?? "app";
export const ftIcon = (cat: string) =>
  ({ compressed: "file-zip", video: "file-video", music: "file-audio", documents: "file-doc", programs: "app-window" })[cat] ?? "file-doc";

export interface MenuItem { label: string; onClick: () => void; checked?: boolean; disabled?: boolean }
export interface MenuGroup { label?: string; items: MenuItem[] }

/** popup menu (toolbar, Tasks, row context menu): fixed at x,y inside the window, arrows move, Esc/Tab/outside click close */
export function Menu({ label, x, y, groups, onClose }: { label: string; x: number; y: number; groups: MenuGroup[]; onClose: () => void }) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ left: x, top: y });
  // the opener, kept once (StrictMode runs effects twice): Esc, Tab and a chosen item give focus back to it
  const back = useRef<HTMLElement | null>(null);
  useLayoutEffect(() => {
    const el = ref.current!;
    const r = el.getBoundingClientRect();
    setPos({ left: Math.max(8, Math.min(x, innerWidth - r.width - 8)), top: Math.max(8, Math.min(y, innerHeight - r.height - 8)) });
    if (!el.contains(document.activeElement)) back.current = document.activeElement as HTMLElement | null;
    el.querySelector<HTMLElement>("[role^=menuitem]:not([disabled])")?.focus();
  }, [x, y]);
  const close = (refocus: boolean) => {
    if (refocus) back.current?.focus?.();
    onClose();
  };
  const onKey = (e: RKE) => {
    const items = [...ref.current!.querySelectorAll<HTMLElement>("[role^=menuitem]:not([disabled])")];
    const i = items.indexOf(document.activeElement as HTMLElement);
    if (e.key === "ArrowDown") items[(i + 1) % items.length]?.focus();
    else if (e.key === "ArrowUp") items[(i - 1 + items.length) % items.length]?.focus();
    else if (e.key === "Home") items[0]?.focus();
    else if (e.key === "End") items[items.length - 1]?.focus();
    else if (e.key === "Escape" || e.key === "Tab") close(true);
    else return;
    e.preventDefault();
  };
  const item = (it: MenuItem, k: number) => (
    <button
      key={k}
      type="button"
      tabIndex={-1}
      className="fx mi"
      role={it.checked == null ? "menuitem" : "menuitemcheckbox"}
      aria-checked={it.checked}
      disabled={it.disabled}
      onClick={() => (close(true), it.onClick())}
    >
      <i aria-hidden="true">{it.checked && <Icon n="check" />}</i>
      <span>{it.label}</span>
    </button>
  );
  return createPortal(
    <>
      <div className="mscrim" onPointerDown={() => close(false)} onContextMenu={(e) => (e.preventDefault(), close(false))} />
      <div ref={ref} className="menu" role="menu" aria-label={label} style={pos} onKeyDown={onKey} onBlur={(e) => !ref.current?.contains(e.relatedTarget) && close(false)}>
        {groups.map((g, gi) =>
          g.label ? (
            <div key={gi} role="group" aria-label={g.label}>
              <div className="mhead" aria-hidden="true">
                {g.label}
              </div>
              {g.items.map(item)}
            </div>
          ) : (
            g.items.map((it, k) => item(it, gi * 100 + k))
          ),
        )}
      </div>
    </>,
    document.body,
  );
}

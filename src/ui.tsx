// SPDX-License-Identifier: RPL-1.5
// Small primitives. State is shown by fill and colour only (owner rule 8, D6): no outline, ring or glow.
import { useEffect, useRef, type ReactNode } from "react";
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
      </div>
    </div>
  );
}

export const ftClass = (cat: string) => ({ compressed: "zip", video: "vid", music: "aud", documents: "doc", programs: "app" })[cat] ?? "app";
export const ftIcon = (cat: string) =>
  ({ compressed: "file-zip", video: "file-video", music: "file-audio", documents: "file-doc", programs: "app-window" })[cat] ?? "file-doc";

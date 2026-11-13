// SPDX-License-Identifier: RPL-1.5
// Every tooltip in GDM, drawn by GDM (taken from GoldenIQ's Mazer Client TooltipLayer): one piece of frosted glass,
// card and arrow cut as a single outline, black shadow only. An element asks with `data-tip="text"`, never `title`
// (a title is Windows' own yellow box, on its own delay). `data-tip-side`: top (default) | bottom | left | right,
// used when it fits. `data-tip-overflow`: only when the text is cut off.
import { useCallback, useEffect, useId, useLayoutEffect, useRef, useState, type CSSProperties } from "react";
import { createPortal } from "react-dom";

const OPEN_DELAY_MS = 420;
const WARM_WINDOW_MS = 500; // from one tooltip straight onto the next: at once
const CLOSE_MS = 180;
const RADIUS = 8;
const ARROW_HALF = 6;
const ARROW_HEIGHT = 6;
const ARROW_FILLET = 1.5;
const ARROW_TIP = 1.4;
const GAP = ARROW_HEIGHT + 2;
const GUTTER = 8;
const ARROW_INSET = RADIUS + ARROW_HALF + ARROW_FILLET;
const BLEED = 24;
const SELECTOR = "[data-tip]";
const NAV_KEYS = new Set(["Tab", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"]);

type Side = "top" | "bottom" | "left" | "right";
type Shown = { anchor: HTMLElement; open: boolean };
type Place = { left: number; top: number; side: Side; arrow: number; width: number; height: number };

const isTruncated = (anchor: HTMLElement) => {
  const a = anchor.querySelector<HTMLElement>("[data-tip-measure]") ?? anchor; // the cut-off part, when it is a child
  return a.scrollWidth > a.clientWidth + 1 || a.scrollHeight > a.clientHeight + 1;
};
const hasContent = (a: HTMLElement) => !!(a.getAttribute("data-tip") ?? "").trim() && (!a.hasAttribute("data-tip-overflow") || isTruncated(a));
const sideOf = (a: HTMLElement): Side => {
  const s = a.getAttribute("data-tip-side");
  return s === "bottom" || s === "left" || s === "right" ? s : "top";
};

export function TooltipLayer() {
  const [shown, setShown] = useState<Shown | null>(null);
  const shownRef = useRef<Shown | null>(null);
  shownRef.current = shown;
  const pending = useRef<HTMLElement | null>(null);
  const quieted = useRef<HTMLElement | null>(null); // clicked: quiet until the pointer leaves it
  const openTimer = useRef<number | undefined>(undefined);
  const closeTimer = useRef<number | undefined>(undefined);
  const closedAt = useRef(0);

  const close = useCallback(() => {
    window.clearTimeout(openTimer.current);
    pending.current = null;
    const cur = shownRef.current;
    if (!cur?.open) return;
    closedAt.current = Date.now();
    setShown({ anchor: cur.anchor, open: false });
    window.clearTimeout(closeTimer.current);
    closeTimer.current = window.setTimeout(() => setShown(null), CLOSE_MS);
  }, []);

  const aim = useCallback(
    (anchor: HTMLElement | null, immediate = false) => {
      const cur = shownRef.current;
      if (anchor && cur?.open && cur.anchor === anchor) return;
      if (anchor && pending.current === anchor) return;
      const warm = cur?.open || Date.now() - closedAt.current < WARM_WINDOW_MS;
      close();
      if (!anchor || anchor === quieted.current || !hasContent(anchor)) return;
      pending.current = anchor;
      openTimer.current = window.setTimeout(() => {
        if (pending.current !== anchor || !anchor.isConnected || !hasContent(anchor)) return;
        window.clearTimeout(closeTimer.current);
        pending.current = null;
        setShown({ anchor, open: true });
      }, immediate || warm ? 0 : OPEN_DELAY_MS);
    },
    [close],
  );

  useEffect(() => {
    const anchorFor = (t: EventTarget | null) => (t instanceof Element ? t.closest<HTMLElement>(SELECTOR) : null);
    let keyboard = false;
    const onOver = (e: PointerEvent) => {
      if (e.pointerType === "touch") return;
      const a = anchorFor(e.target);
      if (quieted.current && a !== quieted.current) quieted.current = null;
      aim(a);
    };
    const onLeave = () => ((quieted.current = null), close());
    const onDown = (e: PointerEvent) => {
      keyboard = false;
      const a = anchorFor(e.target);
      if (a) quieted.current = a;
      close();
    };
    const onKey = (e: KeyboardEvent) => {
      if (NAV_KEYS.has(e.key)) keyboard = true;
      if (e.key === "Escape") {
        const cur = shownRef.current?.anchor ?? pending.current;
        if (cur) quieted.current = cur;
        close();
      }
    };
    const onFocusIn = (e: FocusEvent) => {
      if (!keyboard) return; // a click focuses its button too, and that one was just quieted
      const a = anchorFor(e.target);
      if (!a) return;
      quieted.current = null;
      aim(a, true);
    };
    const onFocusOut = (e: FocusEvent) => {
      const a = anchorFor(e.target);
      if (a && shownRef.current?.anchor === a) close();
    };
    const root = document.documentElement;
    document.addEventListener("pointerover", onOver);
    root.addEventListener("pointerleave", onLeave);
    document.addEventListener("pointerdown", onDown, true);
    document.addEventListener("keydown", onKey, true);
    document.addEventListener("focusin", onFocusIn);
    document.addEventListener("focusout", onFocusOut);
    window.addEventListener("blur", close);
    return () => {
      document.removeEventListener("pointerover", onOver);
      root.removeEventListener("pointerleave", onLeave);
      document.removeEventListener("pointerdown", onDown, true);
      document.removeEventListener("keydown", onKey, true);
      document.removeEventListener("focusin", onFocusIn);
      document.removeEventListener("focusout", onFocusOut);
      window.removeEventListener("blur", close);
      window.clearTimeout(openTimer.current);
      window.clearTimeout(closeTimer.current);
    };
  }, [aim, close]);

  return shown ? <TooltipCard anchor={shown.anchor} open={shown.open} onLost={close} /> : null;
}

/** the preferred side if it fits the window, then the opposite, then the other two; slid along the edge to stay inside */
export function placeTooltip(target: DOMRect, size: { width: number; height: number }, preferred: Side, vp: { width: number; height: number }) {
  const opposite: Record<Side, Side> = { top: "bottom", bottom: "top", left: "right", right: "left" };
  const across: Record<Side, Side[]> = { top: ["right", "left"], bottom: ["right", "left"], left: ["top", "bottom"], right: ["top", "bottom"] };
  const order = [preferred, opposite[preferred], ...across[preferred]];
  const raw = (side: Side) => {
    const cx = target.left + target.width / 2;
    const cy = target.top + target.height / 2;
    if (side === "top") return { left: cx - size.width / 2, top: target.top - GAP - size.height };
    if (side === "bottom") return { left: cx - size.width / 2, top: target.bottom + GAP };
    if (side === "left") return { left: target.left - GAP - size.width, top: cy - size.height / 2 };
    return { left: target.right + GAP, top: cy - size.height / 2 };
  };
  const fits = (side: Side) => {
    const { left, top } = raw(side);
    if (side === "top") return top >= GUTTER;
    if (side === "bottom") return top + size.height <= vp.height - GUTTER;
    if (side === "left") return left >= GUTTER;
    return left + size.width <= vp.width - GUTTER;
  };
  const room = (side: Side) => (side === "top" ? target.top : side === "bottom" ? vp.height - target.bottom : side === "left" ? target.left : vp.width - target.right);
  const side = order.find(fits) ?? [...order].sort((a, b) => room(b) - room(a))[0];
  const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), Math.max(lo, hi));
  let { left, top } = raw(side);
  left = clamp(left, GUTTER, vp.width - size.width - GUTTER);
  top = clamp(top, GUTTER, vp.height - size.height - GUTTER);
  const vertical = side === "top" || side === "bottom";
  const along = vertical ? target.left + target.width / 2 - left : target.top + target.height / 2 - top;
  const arrow = clamp(along, ARROW_INSET, (vertical ? size.width : size.height) - ARROW_INSET);
  return { left: Math.round(left), top: Math.round(top), side, arrow: Math.round(arrow) };
}

/** card + arrow as one outline (one fill, one blur, one shadow: no seam at 125/150 % scaling) */
function tooltipShape(width: number, height: number, side: Side, arrow: number, offset = 0): string {
  const f = (n: number) => Number(n.toFixed(2));
  const pt = (x: number, y: number) => `${f(x + offset)} ${f(y + offset)}`;
  const on = ({ top: "bottom", bottom: "top", left: "right", right: "left" } as const)[side];
  const base = 2 * (ARROW_HALF + ARROW_FILLET);
  const edgeLength = on === "left" || on === "right" ? height : width;
  const tight = Math.max(0, Math.min(RADIUS, (edgeLength - base) / 2));
  const corner = {
    topLeft: on === "top" || on === "left" ? tight : RADIUS,
    topRight: on === "top" || on === "right" ? tight : RADIUS,
    bottomRight: on === "bottom" || on === "right" ? tight : RADIUS,
    bottomLeft: on === "bottom" || on === "left" ? tight : RADIUS,
  };
  const low = tight + base / 2;
  const high = edgeLength - tight - base / 2;
  const at = low <= high ? Math.min(Math.max(arrow, low), high) : edgeLength / 2;
  const arc = (r: number, x: number, y: number) => (r > 0 ? `A ${f(r)} ${f(r)} 0 0 1 ${pt(x, y)}` : `L ${pt(x, y)}`);
  const spike = (cx: number, cy: number, dx: number, dy: number, nx: number, ny: number) => {
    const p = (along: number, out: number) => pt(cx + dx * along + nx * out, cy + dy * along + ny * out);
    const len = Math.hypot(ARROW_HALF, ARROW_HEIGHT);
    const ua = ARROW_HALF / len;
    const uo = ARROW_HEIGHT / len;
    const h = ARROW_HALF, k = ARROW_HEIGHT, fil = ARROW_FILLET, tip = ARROW_TIP;
    return [
      `L ${p(-h - fil, 0)}`,
      `Q ${p(-h, 0)} ${p(-h + ua * fil, uo * fil)}`,
      `L ${p(-ua * tip, k - uo * tip)}`,
      `Q ${p(0, k)} ${p(ua * tip, k - uo * tip)}`,
      `L ${p(h - ua * fil, uo * fil)}`,
      `Q ${p(h, 0)} ${p(h + fil, 0)}`,
    ].join(" ");
  };
  const { topLeft, topRight, bottomRight, bottomLeft } = corner;
  return [
    `M ${pt(topLeft, 0)}`,
    on === "top" ? spike(at, 0, 1, 0, 0, -1) : "",
    `L ${pt(width - topRight, 0)}`,
    arc(topRight, width, topRight),
    on === "right" ? spike(width, at, 0, 1, 1, 0) : "",
    `L ${pt(width, height - bottomRight)}`,
    arc(bottomRight, width - bottomRight, height),
    on === "bottom" ? spike(at, height, -1, 0, 0, 1) : "",
    `L ${pt(bottomLeft, height)}`,
    arc(bottomLeft, 0, height - bottomLeft),
    on === "left" ? spike(0, at, 0, -1, -1, 0) : "",
    `L ${pt(0, topLeft)}`,
    arc(topLeft, topLeft, 0),
    "Z",
  ]
    .filter(Boolean)
    .join(" ");
}

/** measured and placed before it paints, kept on its anchor every frame, closed if the anchor leaves the page */
function TooltipCard({ anchor, open, onLost }: { anchor: HTMLElement; open: boolean; onLost: () => void }) {
  const card = useRef<HTMLDivElement>(null);
  const [place, setPlace] = useState<Place | null>(null);
  const shadowId = useId();
  useLayoutEffect(() => {
    let frame = 0;
    let last = "";
    const measure = () => {
      const el = card.current;
      if (!el) return;
      if (!anchor.isConnected) return onLost();
      const target = anchor.getBoundingClientRect();
      const size = { width: el.offsetWidth, height: el.offsetHeight };
      const key = `${target.left},${target.top},${target.width},${target.height},${size.width},${size.height},${innerWidth},${innerHeight}`;
      if (key !== last) {
        last = key;
        setPlace({ ...placeTooltip(target, size, sideOf(anchor), { width: innerWidth, height: innerHeight }), ...size });
      }
      if (open) frame = requestAnimationFrame(measure);
    };
    measure();
    return () => cancelAnimationFrame(frame);
  }, [anchor, open, onLost]);
  const outline = place ? tooltipShape(place.width, place.height, place.side, place.arrow, BLEED) : null;
  const filter = `tip-shadow-${shadowId.replace(/[^\w-]/g, "")}`;
  return createPortal(
    <div
      ref={card}
      role="tooltip"
      className="tooltip"
      data-state={place && open ? "open" : "closed"}
      data-side={place?.side ?? sideOf(anchor)}
      style={place ? ({ left: place.left, top: place.top, "--tooltip-bleed": `${BLEED}px` } as CSSProperties) : undefined}
    >
      {outline && (
        <>
          {/* the shadow is cast by the outline and cut away inside it: it darkens the page around the card, never the glass */}
          <svg className="tooltip-shadow" aria-hidden="true">
            <defs>
              <filter id={filter} x="-50%" y="-50%" width="200%" height="200%">
                <feGaussianBlur in="SourceAlpha" stdDeviation="6" />
                <feOffset dy="3" result="far" />
                <feFlood floodColor="#000" floodOpacity="0.45" />
                <feComposite in2="far" operator="in" result="farShadow" />
                <feGaussianBlur in="SourceAlpha" stdDeviation="0.7" result="near" />
                <feFlood floodColor="#000" floodOpacity="0.55" />
                <feComposite in2="near" operator="in" result="nearShadow" />
                <feMerge result="both">
                  <feMergeNode in="farShadow" />
                  <feMergeNode in="nearShadow" />
                </feMerge>
                <feComposite in="both" in2="SourceAlpha" operator="out" />
              </filter>
            </defs>
            <path d={outline} filter={`url(#${filter})`} />
          </svg>
          <span className="tooltip-glass" style={{ clipPath: `path("${outline}")` }} />
        </>
      )}
      <div className="tooltip-body">
        <span className="tooltip-text">{anchor.getAttribute("data-tip")}</span>
      </div>
    </div>,
    document.body,
  );
}

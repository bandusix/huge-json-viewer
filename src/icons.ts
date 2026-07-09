// Inline SVG icon set (stroke uses currentColor). Injected into [data-ico] hosts.

const S = (inner: string, opts: { fill?: boolean; w?: number } = {}) =>
  `<svg viewBox="0 0 24 24" fill="${opts.fill ? "currentColor" : "none"}" ` +
  `stroke="${opts.fill ? "none" : "currentColor"}" stroke-width="1.8" ` +
  `stroke-linecap="round" stroke-linejoin="round">${inner}</svg>`;

export const ICONS: Record<string, string> = {
  open: S(
    `<path d="M3 7.5A1.5 1.5 0 0 1 4.5 6h4l2 2.2H19.5A1.5 1.5 0 0 1 21 9.7v8.8A1.5 1.5 0 0 1 19.5 20h-15A1.5 1.5 0 0 1 3 18.5z"/>`
  ),
  search: S(`<circle cx="11" cy="11" r="6.5"/><path d="m20 20-3.6-3.6"/>`),
  up: S(`<path d="m6 14 6-6 6 6"/>`),
  down: S(`<path d="m6 10 6 6 6-6"/>`),
  chevron: S(`<path d="m9 6 6 6-6 6"/>`),
  theme: S(
    `<circle cx="12" cy="12" r="4.2"/><path d="M12 2.5v2.4M12 19.1v2.4M2.5 12h2.4M19.1 12h2.4M5.2 5.2l1.7 1.7M17.1 17.1l1.7 1.7M18.8 5.2l-1.7 1.7M6.9 17.1l-1.7 1.7"/>`
  ),
  collapse: S(
    `<path d="M4 6h16M8 12h12M8 12l-3-2v4z" /><path d="M4 18h16"/>`
  ),
  expand: S(
    `<path d="M4 6h16M8 12h12M4 12l3-2v4z"/><path d="M4 18h16"/>`
  ),
  "file-big": S(
    `<path d="M7 3.5h7l5 5V20a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4.5a1 1 0 0 1 1-1z"/><path d="M14 3.5V9h5"/><path d="M9.5 13h5M9.5 16h3"/>`
  ),
  download: S(`<path d="M12 4v11M7 11l5 5 5-5M5 20h14"/>`),
  globe: S(
    `<circle cx="12" cy="12" r="9"/><path d="M3 12h18"/><path d="M12 3c2.6 2.7 2.6 15.3 0 18M12 3c-2.6 2.7-2.6 15.3 0 18"/>`
  ),
  check: S(`<path d="m5 12.5 4.5 4.5L19 6.5"/>`),
};

/** Replace every [data-ico] host in the DOM with its inline SVG. */
export function mountIcons(root: ParentNode = document): void {
  root.querySelectorAll<HTMLElement>("[data-ico]").forEach((el) => {
    const name = el.dataset.ico;
    if (name && ICONS[name] && !el.dataset.icoMounted) {
      el.innerHTML = ICONS[name];
      el.dataset.icoMounted = "1";
    }
  });
}

/** The twisty chevron used per-row (kept tiny/separate for hot-path rendering). */
export const TWISTY_SVG =
  `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" ` +
  `stroke-linecap="round" stroke-linejoin="round" width="9" height="9"><path d="m9 6 6 6-6 6"/></svg>`;

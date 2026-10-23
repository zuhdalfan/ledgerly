// Typed access to the captured layout table (page-space CSS px, 1920-wide).
import raw from './live-layout.json';

export type Box = { x: number; y: number; width: number; height: number };
export type PageLayout = {
  pageH: number;
  boxes: Record<string, Box[] | Box | null>;
  cutouts: ({ file: string } & Box)[];
};

export const L = raw as unknown as Record<string, PageLayout> & { pageW: number };

export const page = (name: string): PageLayout => L[name];

export const box = (name: string, key: string, index = 0): Box => {
  const b = page(name).boxes[key];
  if (!b) return { x: 0, y: 0, width: 0, height: 0 };
  return Array.isArray(b) ? b[Math.min(index, b.length - 1)] : b;
};

export const boxes = (name: string, key: string): Box[] => {
  const b = page(name).boxes[key];
  return Array.isArray(b) ? b : b ? [b] : [];
};

/** Relative path for a page texture. PageCam runs staticFile() on it itself. */
export const tex = (name: string) => `textures/live/${name}-full.png`;
/** Relative path for an element cutout (apply staticFile at the <Img>). */
export const cut = (file: string) => `textures/live/${file}`;

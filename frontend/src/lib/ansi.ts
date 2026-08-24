// Container logs arrive exactly as the process wrote them, which for most
// modern loggers means SGR ("select graphic rendition") escape sequences for
// colour and emphasis. A browser renders the ESC byte as nothing, so dumping
// the raw text into a <pre> leaves the parameter tail visible as literal
// `[2m` / `[32m` noise. This module turns those sequences into styled segments
// the log view can render, and drops the escape sequences it cannot interpret
// instead of showing them.

export interface AnsiStyle {
  /** CSS colour for the text, unset when the default foreground applies. */
  color?: string;
  /** CSS colour behind the text, unset when the default background applies. */
  background?: string;
  bold?: boolean;
  dim?: boolean;
  italic?: boolean;
  underline?: boolean;
  strike?: boolean;
  /** SGR 7: foreground and background are swapped when rendered. */
  inverse?: boolean;
}

export interface AnsiSegment {
  text: string;
  style: AnsiStyle;
}

// The 16 basic colours, picked to stay legible on the dark log background.
// Indices 0-7 are the normal set, 8-15 the bright one.
const BASE_COLORS = [
  '#3b4048', // black
  '#e06c75', // red
  '#98c379', // green
  '#e5c07b', // yellow
  '#61afef', // blue
  '#c678dd', // magenta
  '#56b6c2', // cyan
  '#abb2bf', // white
  '#5c6370', // bright black
  '#ff7b86', // bright red
  '#b5e08c', // bright green
  '#ffd68a', // bright yellow
  '#7cc5ff', // bright blue
  '#e39ef7', // bright magenta
  '#70d4df', // bright cyan
  '#ffffff' // bright white
];

// Every escape sequence a log line can carry: CSI (`ESC [ … final byte`, which
// includes SGR), OSC strings (`ESC ] … BEL|ST`) and the short two-byte escapes.
// Only SGR is interpreted; the rest is stripped.
// eslint-disable-next-line no-control-regex
const ESCAPE_SEQUENCE = /\x1b(?:\[[0-9;:?]*[ -/]*[@-~]|\][\s\S]*?(?:\x07|\x1b\\)|[@-Z\\-_])/g;

function rgb(r: number, g: number, b: number): string {
  return `#${[r, g, b].map((v) => v.toString(16).padStart(2, '0')).join('')}`;
}

/** xterm 256-colour palette: 16 base colours, a 6×6×6 cube, then 24 greys. */
function color256(index: number): string | undefined {
  if (!Number.isInteger(index) || index < 0 || index > 255) return undefined;
  if (index < 16) return BASE_COLORS[index];
  if (index < 232) {
    const level = (v: number) => (v === 0 ? 0 : 55 + v * 40);
    const n = index - 16;
    return rgb(level(Math.floor(n / 36)), level(Math.floor(n / 6) % 6), level(n % 6));
  }
  const grey = 8 + (index - 232) * 10;
  return rgb(grey, grey, grey);
}

/**
 * Apply one SGR sequence's parameters to `style`, returning the updated style.
 * Extended colours (`38`/`48`) consume the parameters that follow them, hence
 * the index-based loop. Unknown parameters are ignored, the way a terminal
 * treats them.
 */
function applySgr(style: AnsiStyle, params: number[]): AnsiStyle {
  let next: AnsiStyle = { ...style };
  for (let i = 0; i < params.length; i++) {
    const code = params[i];
    if (code === 0) next = {};
    else if (code === 1) next.bold = true;
    else if (code === 2) next.dim = true;
    else if (code === 3) next.italic = true;
    else if (code === 4) next.underline = true;
    else if (code === 7) next.inverse = true;
    else if (code === 9) next.strike = true;
    else if (code === 22) {
      delete next.bold;
      delete next.dim;
    } else if (code === 23) delete next.italic;
    else if (code === 24) delete next.underline;
    else if (code === 27) delete next.inverse;
    else if (code === 29) delete next.strike;
    else if (code >= 30 && code <= 37) next.color = BASE_COLORS[code - 30];
    else if (code === 39) delete next.color;
    else if (code >= 40 && code <= 47) next.background = BASE_COLORS[code - 40];
    else if (code === 49) delete next.background;
    else if (code >= 90 && code <= 97) next.color = BASE_COLORS[code - 90 + 8];
    else if (code >= 100 && code <= 107) next.background = BASE_COLORS[code - 100 + 8];
    else if (code === 38 || code === 48) {
      const target = code === 38 ? 'color' : 'background';
      const mode = params[i + 1];
      if (mode === 5) {
        const resolved = color256(params[i + 2]);
        if (resolved) next[target] = resolved;
        else delete next[target];
        i += 2;
      } else if (mode === 2) {
        const [r, g, b] = params.slice(i + 2, i + 5);
        if ([r, g, b].every((v) => Number.isInteger(v) && v >= 0 && v <= 255)) {
          next[target] = rgb(r, g, b);
        }
        i += 4;
      }
    }
  }
  return next;
}

/** Parameters of an SGR sequence; empty ones default to 0, like a terminal. */
function parseParams(sequence: string): number[] {
  // Drop the leading `ESC[` and the trailing `m`.
  const body = sequence.slice(2, -1);
  if (body === '') return [0];
  // Sub-parameters (`38:5:9`) are the colon-separated form of the same thing.
  return body.split(/[;:]/).map((part) => {
    const value = Number.parseInt(part, 10);
    return Number.isNaN(value) ? 0 : value;
  });
}

/**
 * Split log text into styled runs. Text without escape sequences comes back as
 * a single unstyled segment, so the common case costs one array entry.
 */
export function parseAnsi(text: string): AnsiSegment[] {
  const segments: AnsiSegment[] = [];
  let style: AnsiStyle = {};
  let cursor = 0;

  ESCAPE_SEQUENCE.lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = ESCAPE_SEQUENCE.exec(text)) !== null) {
    if (match.index > cursor) {
      segments.push({ text: text.slice(cursor, match.index), style });
    }
    // `ESC[…m` is SGR; every other sequence only moved a cursor or set a
    // window title and means nothing in a log view.
    if (match[0][1] === '[' && match[0].endsWith('m')) {
      style = applySgr(style, parseParams(match[0]));
    }
    cursor = match.index + match[0].length;
  }
  if (cursor < text.length) {
    segments.push({ text: text.slice(cursor), style });
  }
  return segments;
}

/** Inline CSS for one segment; empty when the segment needs no styling. */
export function ansiStyleToCss(style: AnsiStyle): string {
  const declarations: string[] = [];
  // Inverse swaps the two colours. With only one of them set the other side
  // falls back to the log view's own colours, so a bare `ESC[7m` still reads
  // as highlighted.
  const color = style.inverse ? (style.background ?? '#000000') : style.color;
  const background = style.inverse ? (style.color ?? 'currentColor') : style.background;
  if (color) declarations.push(`color:${color}`);
  if (background) declarations.push(`background-color:${background}`);
  if (style.bold) declarations.push('font-weight:600');
  if (style.dim) declarations.push('opacity:0.65');
  if (style.italic) declarations.push('font-style:italic');
  const lines = [style.underline && 'underline', style.strike && 'line-through'].filter(Boolean);
  if (lines.length > 0) declarations.push(`text-decoration:${lines.join(' ')}`);
  return declarations.join(';');
}

import { describe, it, expect } from 'vitest';
import { ansiStyleToCss, parseAnsi } from './ansi';

const ESC = '\u001b';
const plainText = (text: string) =>
  parseAnsi(text)
    .map((segment) => segment.text)
    .join('');

describe('parseAnsi', () => {
  it('returns untouched text as a single unstyled segment', () => {
    expect(parseAnsi('starting up\nlistening on :3033')).toEqual([
      { text: 'starting up\nlistening on :3033', style: {} }
    ]);
  });

  it('strips the escape sequences from a coloured tracing line', () => {
    const line = `${ESC}[2m2026-08-20T11:36:19.051818Z${ESC}[0m ${ESC}[32m INFO${ESC}[0m ${ESC}[2mfoobar::routes${ESC}[0m${ESC}[2m:${ESC}[0m request`;

    expect(plainText(line)).toBe('2026-08-20T11:36:19.051818Z  INFO foobar::routes: request');
  });

  it('applies and resets styles across segments', () => {
    const segments = parseAnsi(`${ESC}[1;31mboom${ESC}[0m done`);

    expect(segments).toEqual([
      { text: 'boom', style: { bold: true, color: '#e06c75' } },
      { text: ' done', style: {} }
    ]);
  });

  it('treats a bare ESC[m as a reset', () => {
    expect(parseAnsi(`${ESC}[33mwarn${ESC}[mplain`)[1].style).toEqual({});
  });

  it('drops escape sequences that are not SGR', () => {
    const noisy = `${ESC}[2K${ESC}[1Aprogress${ESC}]0;window title${ESC}\\ done`;

    expect(parseAnsi(noisy)).toEqual([
      { text: 'progress', style: {} },
      { text: ' done', style: {} }
    ]);
  });

  it('resolves 256-colour and truecolour parameters', () => {
    expect(parseAnsi(`${ESC}[38;5;196mred`)[0].style.color).toBe('#ff0000');
    expect(parseAnsi(`${ESC}[38;5;250mgrey`)[0].style.color).toBe('#bcbcbc');
    expect(parseAnsi(`${ESC}[48;2;17;34;51mrgb`)[0].style.background).toBe('#112233');
  });

  it('does not read extended-colour parameters as separate codes', () => {
    // 38;5;1 must not also switch on bold via the trailing `1`.
    expect(parseAnsi(`${ESC}[38;5;1mred`)[0].style).toEqual({ color: '#e06c75' });
  });
});

describe('ansiStyleToCss', () => {
  it('is empty for unstyled text', () => {
    expect(ansiStyleToCss({})).toBe('');
  });

  it('renders colour and emphasis', () => {
    expect(ansiStyleToCss({ color: '#98c379', bold: true, underline: true })).toBe(
      'color:#98c379;font-weight:600;text-decoration:underline'
    );
  });

  it('swaps the colours for inverse text', () => {
    expect(ansiStyleToCss({ color: '#98c379', inverse: true })).toBe(
      'color:#000000;background-color:#98c379'
    );
  });
});

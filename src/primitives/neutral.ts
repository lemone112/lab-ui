/**
 * Neutral primitive scale generator.
 *
 * Generates a 13-step neutral scale from three perceptual anchors
 * using CAM16-UCS color space. CAM16-UCS models human vision more
 * accurately than OKLab for near-neutral colors because it accounts
 * for the Helmholtz–Kohlrausch effect (saturation increases perceived
 * brightness) and hue non-linearity near the achromatic axis.
 *
 * Algorithm:
 * - J' (lightness): power ease-in (light half) / ease-out (dark half)
 * - M' (chroma): sinusoidal envelope (peak near step 8),
 *   compensating for chroma discrimination thresholds in mid-tones
 * - h (hue): power ease-in from white's intrinsic hue toward base hue,
 *   avoiding the purple shift that occurs with fixed-hue models
 *
 * References:
 * - Li, C. et al. (2017). "Comprehensive color solutions: CAM16, CAT16, and CAM16-UCS"
 * - Hunt, R.W.G. (2004). "The Reproduction of Colour"
 */

import { formatHex, parse, converter } from 'culori';
import type { Oklab } from 'culori';

import Color from 'colorjs.io';

const toOklab = converter('oklab');

/** 13-step neutral scale (0 = lightest, 12 = darkest). */
export interface NeutralScale {
  readonly steps: ReadonlyArray<string>;
  readonly base: string;
}

/** Input anchors. */
export interface NeutralLightOptions {
  readonly light: string;
  readonly base: string;
  readonly dark: string;
}

/* ------------------------------------------------------------------ */
/*  CAM16-JMH → CAM16-UCS helpers                                      */
/* ------------------------------------------------------------------ */

interface UcsCoords {
  Jp: number;
  ap: number;
  bp: number;
  h: number;
}

function toUcs(hex: string): UcsCoords {
  const [J, M, h] = new Color(hex).to('cam16-jmh').coords as [number, number, number];
  const Jp = (1.7 * J) / (1 + 0.007 * J);
  const Mp = Math.log(1 + 0.0228 * M) / 0.0228;
  const hr = (h * Math.PI) / 180;
  return { Jp, ap: Mp * Math.cos(hr), bp: Mp * Math.sin(hr), h };
}

function fromUcs({ Jp, ap, bp }: UcsCoords): string {
  const J = Jp / (1.7 - 0.007 * Jp);
  const Mp = Math.sqrt(ap * ap + bp * bp);
  const M = (Math.exp(0.0228 * Mp) - 1) / 0.0228;
  const h = ((Math.atan2(bp, ap) * 180) / Math.PI + 360) % 360;
  return new Color('cam16-jmh', [J, M, h]).to('srgb').toString({ format: 'hex' });
}

/* ------------------------------------------------------------------ */
/*  Curve helpers                                                      */
/* ------------------------------------------------------------------ */

/** Ease-in power (0 → 1). */
function easeIn(t: number, p: number): number {
  return Math.pow(t, p);
}

/** Ease-in-out power (0 → 1 → 0). */
function easeInOut(t: number, p: number): number {
  return t < 0.5
    ? Math.pow(2 * t, p) / 2
    : 1 - Math.pow(2 - 2 * t, p) / 2;
}

/** Sinusoidal envelope, peak at tPeak. */
function sineEnv(t: number, tPeak: number): number {
  if (t <= tPeak) {
    return Math.sin((Math.PI * t) / (2 * tPeak));
  }
  return Math.sin((Math.PI * (1 - t)) / (2 * (1 - tPeak)));
}

/* ------------------------------------------------------------------ */
/*  Constants tuned to the Figma Variables neutral scale.              */
/* ------------------------------------------------------------------ */

/** Lightness ease exponent. */
const P_J = 1.7;

/** Hue ease exponent (fast jump from white hue to base hue). */
const P_H = 0.6;

/** Sinusoid peak position (normalized 0..1). */
const T_PEAK = 0.35;

/** Chroma overshoot above base (20 %). */
const CHROMA_BOOST = 1.2;

/* ------------------------------------------------------------------ */
/*  Generator                                                          */
/* ------------------------------------------------------------------ */

export function createNeutralLightScale(options: NeutralLightOptions): NeutralScale {
  const a0 = toUcs(options.light);
  const a6 = toUcs(options.base);
  const a12 = toUcs(options.dark);

  const MpBase = Math.sqrt(a6.ap * a6.ap + a6.bp * a6.bp);
  const MpDark = Math.sqrt(a12.ap * a12.ap + a12.bp * a12.bp);
  const MpPeak = MpBase * CHROMA_BOOST;

  const baseHueRad = (a6.h * Math.PI) / 180;

  const steps: string[] = [];

  for (let i = 0; i <= 12; i++) {
    const t = i / 12;

    /* ----- J' (lightness) ----- */
    let Jp: number;
    if (i < 6) {
      Jp = a0.Jp + (a6.Jp - a0.Jp) * easeIn(t / 0.5, P_J);
    } else if (i === 6) {
      Jp = a6.Jp;
    } else {
      Jp = a6.Jp + (a12.Jp - a6.Jp) * (1 - easeIn(1 - (t - 0.5) / 0.5, P_J));
    }

    /* ----- M' (chroma) ----- */
    const env = sineEnv(t, T_PEAK);
    const Mp = MpDark + (MpPeak - MpDark) * env;

    /* ----- h (hue) ----- */
    let hDeg: number;
    if (i <= 6) {
      hDeg = a0.h + (a6.h - a0.h) * easeIn(t / 0.5, P_H);
    } else {
      hDeg = a6.h + (a12.h - a6.h) * ((t - 0.5) / 0.5);
    }
    if (i === 6) hDeg = a6.h;

    const hr = (hDeg * Math.PI) / 180;
    const ap = Mp * Math.cos(hr);
    const bp = Mp * Math.sin(hr);

    steps.push(fromUcs({ Jp, ap, bp, h: hDeg }));
  }

  const baseStep = steps[6];
  if (!baseStep) {
    throw new Error('neutral: base step missing');
  }
  return { steps: Object.freeze(steps), base: baseStep };
}

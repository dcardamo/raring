// Raring's own body-map figures. Each figure is drawn as its left half in a
// 100×220 box (centre line x=50) and mirrored; muscle groups are separate
// shapes so each can be shaded by its set count.
import {colors} from './charts';
import type {Figure} from './generated/Figure';
import type {Muscle} from './generated/Muscle';

type Shape = {muscle: Muscle; d: string};
export type FigureArt = {outline: string; head: {rx: number; ry: number}; front: Shape[]; back: Shape[]; frontLines: string; backLines: string};

const f = (n: number) => +n.toFixed(2);

// Widths are distances from the centre line.
const BUILD = {
 female: {neck: 4.6, trap: 12, shoulder: 20.5, arm: 4.6, fore: 3.6, armpit: 16.2, chest: 16, waist: 11.6, hip: 18.6, thigh: 17.4, knee: 11.2, calf: 12.8, ankle: 8.6},
 male: {neck: 5.6, trap: 14.5, shoulder: 24.5, arm: 5.6, fore: 4.2, armpit: 18.6, chest: 18.4, waist: 14.6, hip: 16.6, thigh: 16.6, knee: 11.6, calf: 13.4, ankle: 9},
} as const;

export function figureArt(figure: Figure): FigureArt {
 const b = BUILD[figure], x = (d: number) => f(50 - d);
 // Arm runs from the shoulder down and slightly out.
 const armOut = b.shoulder + 0.6, elbowOut = b.shoulder + 2.2, wristOut = b.shoulder + 3.4;
 const outline = [
  `M50,27 L${x(b.neck)},27`,
  `C${x(b.neck)},31 ${x(b.trap - 3)},33.5 ${x(b.trap)},35`,
  `C${x(b.shoulder - 3)},36 ${x(armOut)},38 ${x(armOut)},45`,
  `C${x(elbowOut - 0.4)},58 ${x(elbowOut)},70 ${x(elbowOut)},82`,
  `C${x(wristOut)},94 ${x(wristOut + 0.4)},106 ${x(wristOut - 0.4)},114`,
  `C${x(wristOut + 0.6)},120 ${x(wristOut)},128 ${x(wristOut - 2.2)},129.5`,
  `C${x(wristOut - 4.6)},130 ${x(wristOut - 5)},124 ${x(wristOut - 4)},116`,
  `C${x(wristOut - 4.4)},106 ${x(elbowOut - b.fore * 1.7)},94 ${x(elbowOut - b.arm * 1.5)},82`,
  `C${x(b.armpit + 1.6)},72 ${x(b.armpit + 0.6)},60 ${x(b.armpit)},52`,
  `C${x(b.chest - 0.6)},62 ${x(b.waist + 0.8)},72 ${x(b.waist)},82`,
  `C${x(b.waist + 0.2)},90 ${x(b.hip)},96 ${x(b.hip)},106`,
  `C${x(b.thigh + 0.4)},124 ${x(b.knee + 2.4)},144 ${x(b.knee)},160`,
  `C${x(b.calf - 0.4)},168 ${x(b.calf)},180 ${x(b.ankle + 1.4)},198`,
  `C${x(b.ankle + 0.6)},204 ${x(b.ankle + 1.4)},209 ${x(b.ankle + 2.2)},212`,
  `C${x(b.ankle + 1.4)},215.5 ${x(3)},215.5 ${x(3.2)},212`,
  `C${x(3.6)},208 ${x(3.6)},204 ${x(3.8)},198`,
  `C${x(4.6)},184 ${x(4.6)},172 ${x(3.8)},162`,
  `C${x(3.6)},146 ${x(1.6)},126 ${x(0.6)},112`,
  'L50,112 Z',
 ].join(' ');
 const shoulders = `M${x(b.trap - 1)},36.2 C${x(b.shoulder - 2)},36 ${x(armOut - 0.4)},39 ${x(armOut - 0.6)},46.5 C${x(armOut - 0.8)},52 ${x(b.shoulder - 1.2)},56 ${x(b.shoulder - 3.4)},57 C${x(b.shoulder - 4.2)},50 ${x(b.trap + 1)},43 ${x(b.trap - 1)},36.2 Z`;
 const upper = (inner: number) => `M${x(b.shoulder - 3.2)},${59 + inner} C${x(elbowOut - 0.3)},${63 + inner} ${x(elbowOut - 0.2)},72 ${x(elbowOut - 0.8)},79 C${x(elbowOut - b.arm)},81.5 ${x(b.armpit + 1.2)},74 ${x(b.armpit + 0.6)},${61 + inner} Z`;
 const forearm = `M${x(elbowOut - 0.3)},86 C${x(wristOut)},96 ${x(wristOut + 0.2)},104 ${x(wristOut - 0.6)},112 L${x(wristOut - 3.4)},112 C${x(wristOut - 3.8)},104 ${x(elbowOut - b.fore * 1.5)},95 ${x(elbowOut - b.arm * 1.3)},86 Z`;
 const chest = `M49.4,40.5 C${x(5)},38.6 ${x(b.trap - 1)},38.8 ${x(b.chest - 2.4)},42 C${x(b.chest - 0.4)},46 ${x(b.chest - 0.8)},53 ${x(b.chest - 2.2)},57.5 C${x(b.chest / 2)},62 ${x(3)},61 49.4,58.6 Z`;
 const absBlocks = [62, 69.5, 77].map(y => `M${x(5.6)},${y + 1} Q${x(5.8)},${y} ${x(5)},${y} L49.3,${y} L49.3,${y + 6.2} L${x(5.6)},${y + 6.2} Z`).join(' ') + ` M${x(5.2)},85 L49.3,85 L49.3,98 Q${x(2.4)},98 ${x(3.6)},94 Z`;
 const obliques = `M${x(7.2)},62 C${x(b.waist - 0.6)},66 ${x(b.waist - 0.8)},76 ${x(b.waist - 0.2)},84 C${x(b.waist)},90 ${x(b.hip - 3.4)},94 ${x(6.6)},97 C${x(7.6)},86 ${x(7.8)},74 ${x(7.2)},62 Z`;
 const hipFront = `M${x(b.hip - 2.8)},101 C${x(b.hip - 6)},102 ${x(6)},106 ${x(1.2)},113 C${x(2.4)},118 ${x(3.2)},124 ${x(3.8)},128 C${x(4.8)},119 ${x(b.hip - 6.4)},109 ${x(b.hip - 2.8)},101 Z`;
 const quads = `M${x(b.hip - 0.8)},106 C${x(b.thigh)},120 ${x(b.knee + 2.6)},140 ${x(b.knee + 0.6)},154 C${x(b.knee - 1)},158 ${x(5.4)},158 ${x(4.4)},154 C${x(4.6)},142 ${x(4.2)},134 ${x(4.8)},128 C${x(6.4)},118 ${x(b.hip - 5.6)},110 ${x(b.hip - 0.8)},106 Z`;
 const calvesFront = `M${x(b.calf - 0.8)},168 C${x(b.calf - 0.4)},176 ${x(b.calf - 1.4)},186 ${x(b.ankle + 2)},194 C${x(b.ankle + 3.2)},186 ${x(b.knee - 1.6)},174 ${x(b.knee - 1.8)},166 Z M${x(4.4)},168 C${x(4.6)},176 ${x(4.4)},184 ${x(4.6)},190 C${x(5.6)},182 ${x(6.2)},174 ${x(6)},167 Z`;
 const traps = `M50,28 L${x(b.neck - 0.4)},28.5 C${x(b.neck)},32 ${x(b.trap - 3)},34 ${x(b.trap)},35.6 C${x(b.trap - 3)},38 ${x(5)},44 50,58 Z`;
 const lats = `M${x(b.armpit - 0.6)},52 C${x(b.chest - 1)},62 ${x(b.waist + 0.4)},72 ${x(b.waist - 0.2)},80 C${x(8)},82 ${x(4)},80 ${x(1.4)},76 L${x(1.4)},60 C${x(6)},55 ${x(b.armpit - 4)},50 ${x(b.armpit - 0.6)},52 Z`;
 const lowerBack = `M${x(1.4)},79 C${x(4)},82 ${x(6.4)},83 ${x(7.2)},84 C${x(7.4)},90 ${x(6)},95 ${x(1.4)},98 Z`;
 const glutes = `M49.6,100 C${x(5)},98.4 ${x(b.hip - 3)},99 ${x(b.hip - 0.8)},105 C${x(b.hip)},112 ${x(b.hip - 2)},119 ${x(b.hip - 6)},121.5 C${x(6)},123 ${x(2)},121 49.6,117 Z`;
 const hipBack = `M${x(b.waist + 0.4)},88 C${x(b.hip - 1)},92 ${x(b.hip)},97 ${x(b.hip)},102 C${x(b.hip - 3)},98.6 ${x(b.hip - 7)},97.2 ${x(7)},97.4 C${x(9)},93.4 ${x(b.waist - 1)},89.6 ${x(b.waist + 0.4)},88 Z`;
 const hamstrings = `M${x(b.thigh - 1)},124 C${x(b.thigh - 0.6)},136 ${x(b.knee + 1.6)},148 ${x(b.knee + 0.2)},156 L${x(4.6)},156 C${x(4)},146 ${x(2.6)},134 ${x(2.2)},124 C${x(6)},122.6 ${x(b.thigh - 6)},122.6 ${x(b.thigh - 1)},124 Z`;
 const calvesBack = `M${x(b.calf - 0.6)},166 C${x(b.calf + 0.4)},174 ${x(b.calf - 0.6)},184 ${x(b.ankle + 2.6)},190 C${x(b.ankle + 1.6)},186 ${x(8.4)},178 ${x(8.2)},170 C${x(8.6)},167 ${x(b.knee - 2)},165 ${x(b.calf - 0.6)},166 Z M${x(4.2)},166 C${x(3.6)},174 ${x(4)},184 ${x(5.8)},189 C${x(7.4)},184 ${x(8)},176 ${x(7.8)},170 C${x(7)},167 ${x(5.6)},165.6 ${x(4.2)},166 Z`;
 const frontLines = `M50,40 L50,98 M${x(b.chest - 2.6)},57 C${x(b.chest / 2)},61.6 ${x(3)},60.6 50,58.6 M${x(b.knee + 1)},158 C${x(7.6)},160 ${x(5.6)},160 ${x(4.4)},158`;
 const backLines = `M50,30 L50,98 M50,108 L50,121 M${x(b.armpit - 0.6)},52 C${x(b.armpit - 4)},50 ${x(6)},55 ${x(1.4)},60`;
 return {
  outline,
  head: figure === 'male' ? {rx: 8.6, ry: 11} : {rx: 8, ry: 10.6},
  front: [
   {muscle: 'shoulders', d: shoulders},
   {muscle: 'chest', d: chest},
   {muscle: 'arms', d: upper(0)},
   {muscle: 'arms', d: forearm},
   {muscle: 'core', d: absBlocks},
   {muscle: 'core', d: obliques},
   {muscle: 'hips', d: hipFront},
   {muscle: 'quads', d: quads},
   {muscle: 'calves', d: calvesFront},
  ],
  back: [
   {muscle: 'back', d: traps},
   {muscle: 'back', d: lats},
   {muscle: 'back', d: lowerBack},
   {muscle: 'shoulders', d: shoulders},
   {muscle: 'arms', d: upper(-4)},
   {muscle: 'arms', d: forearm},
   {muscle: 'hips', d: hipBack},
   {muscle: 'glutes', d: glutes},
   {muscle: 'hamstrings', d: hamstrings},
   {muscle: 'calves', d: calvesBack},
  ],
  frontLines,
  backLines,
 };
}

const AMOUNT = colors.amount, FAINT = '#263C59', NONE = '#343A43';
const mix = (a: string, b: string, t: number) => '#' + [1, 3, 5].map(i => Math.round(parseInt(a.slice(i, i + 2), 16) * (1 - t) + parseInt(b.slice(i, i + 2), 16) * t).toString(16).padStart(2, '0')).join('');
/** One blue, faint to strong; no sets is a neutral gray, not a faint blue. */
export const amountColor = (sets: number, max: number) => sets > 0 ? mix(FAINT, AMOUNT, Math.min(sets / Math.max(max, 1), 1)) : NONE;
const luminance = (h: string) => [1, 3, 5].map(i => parseInt(h.slice(i, i + 2), 16) / 255).map(c => c <= .03928 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4).reduce((a, c, i) => a + c * [.2126, .7152, .0722][i], 0);
/** Black or white text, whichever reads better on the fill. */
export const textOn = (fill: string) => 1.05 / (luminance(fill) + .05) >= (luminance(fill) + .05) / .05 ? '#FFFFFF' : '#000000';

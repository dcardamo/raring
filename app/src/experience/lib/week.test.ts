import {describe, expect, it} from 'vitest';
import {derive, type State} from './rules';
import {backlog, dayRings, planPrompt, proposeGoals, takeItem, weekOutcome, weekRings, RING_ORDER} from './week';
import {fresh} from './store.svelte';
const goals = (t: object, from = '2025-09-01') => [{from, targets: {strength: 0, zone2: 0, hi: 0, im: 0, sleep: 0, sleepScore: 0, sleepTime: '22:15', sleepKind: 'winddown' as const, ...t}}];
const at = (extra: Partial<State>) => derive(fresh('mixed', {day: '2026-11-18', clock: 'day', ...extra}));
describe('day rings', () => {
 it('draws no ring where nothing was planned or recorded, an outline where planned', () => {
  const c = at({goalVersions: goals({strength: 2, zone2: 60})});
  const fri = dayRings(c, '2026-11-20');
  expect(fri.rings.map(r => r.key)).toEqual(['strength', 'zone2']);
  expect(fri.rings.every(r => r.outline === 0)).toBe(true);
  const thu = dayRings(at({goalVersions: goals({strength: 2}), intentions: [{id: 'x', sport: 'strength', date: '2026-11-19', time: null, link: null, history: []}]}), '2026-11-19');
  expect(thu.rings[0]).toMatchObject({planned: 1, filled: 0, outline: 1});
 });
 it('fills recorded work, even when nothing was planned that day', () => {
  const mon = dayRings(at({goalVersions: goals({strength: 2})}), '2026-11-16');
  expect(mon.rings[0]).toMatchObject({recorded: 1, filled: 1, outline: 1});
 });
 it('keeps one order everywhere', () => {
  const c = at({goalVersions: goals({sleep: 3, im: 100, strength: 1, hi: 20, zone2: 50})});
  expect(dayRings(c, '2026-11-17').rings.map(r => r.key)).toEqual(RING_ORDER);
  expect(weekRings(c).rings.map(r => r.key)).toEqual(RING_ORDER);
 });
 it('marks rest, illness, pause and awaiting data distinctly', () => {
  expect(dayRings(at({rest: {day: '2026-11-18'}}), '2026-11-18').mark).toBe('rest');
  expect(dayRings(at({pause: {since: '2026-11-17', review: null, reason: 'ill'}}), '2026-11-18').mark).toBe('ill');
  expect(dayRings(at({pause: {since: '2026-11-17', review: null}}), '2026-11-18').mark).toBe('paused');
  expect(dayRings(derive(fresh('sparse', {day: '2026-11-18', clock: 'day'})), '2026-11-17').mark).toBe('unknown');
 });
 it('counts recorded Zone 4–5 time only, never planned minutes', () => {
  const c = at({goalVersions: goals({hi: 30})});
  const w = weekRings(c).rings[0];
  const acts = c.world.activities.filter(a => a.date >= c.weekFrom && a.date <= c.calendarDate);
  expect(w.recorded).toBe(Math.floor(acts.reduce((n, a) => n + (a.zones[3] ?? 0) + (a.zones[4] ?? 0), 0) / 60));
 });
});
describe('backlog', () => {
 it('lists what is left after recordings and placed sessions, and takes one for today', () => {
  const c = at({goalVersions: goals({strength: 3}), intentions: []});
  const b = backlog(c);
  expect(b.items.filter(i => i.key === 'strength')).toHaveLength(2);
  expect(takeItem(c, b.items[0], c.calendarDate)).toMatchObject({sport: 'strength', date: '2026-11-18', allocation: {strength: 1}});
 });
 it('pauses while ill and adds no catch-up afterwards', () => {
  expect(backlog(at({goalVersions: goals({strength: 3}), pause: {since: '2026-11-17', review: null, reason: 'ill'}})).paused).toBe(true);
  const after = at({day: '2026-11-20', goalVersions: goals({strength: 3}), pauses: [{since: '2026-11-17', until: '2026-11-19', review: null, reason: 'ill'}], intentions: []});
  expect(backlog(after).items.filter(i => i.key === 'strength').length).toBeLessThanOrEqual(2);
 });
});
describe('week review and next goals', () => {
 it('does not count a week with a coverage gap or a pause', () => {
  const c = at({});
  expect(weekOutcome(c, '2026-10-26').counted).toBe(false);
  expect(weekOutcome(c, '2026-10-19').reasons.join()).toContain('paused');
 });
 it('never escalates without comfortable success and eases back after illness', () => {
  const c = at({day: '2026-11-23', goalVersions: goals({strength: 9, zone2: 30})});
  const p = proposeGoals(c, '2026-11-23');
  expect(p.find(x => x.key === 'strength')!.proposed).toBeLessThanOrEqual(9);
  const ill = at({day: '2026-11-23', goalVersions: goals({strength: 2}), pauses: [{since: '2026-11-18', until: '2026-11-21', review: null, reason: 'ill'}]});
  expect(proposeGoals(ill, '2026-11-23')[0]).toMatchObject({current: 2, proposed: 1});
 });
 it('nudges a minute goal up only after two met weeks', () => {
  const c = at({day: '2026-11-23', goalVersions: goals({zone2: 10})});
  const p = proposeGoals(c, '2026-11-23')[0];
  expect(p.proposed).toBeGreaterThanOrEqual(p.current);
  expect(p.proposed - p.current).toBeLessThanOrEqual(20);
 });
 it('never nudges the Sleep Score goal', () => {
  const c = at({goalVersions: goals({sleepScore: 80})});
  expect(proposeGoals(c, '2026-11-23').some(p => (p.key as string) === 'sleepScore')).toBe(false);
 });
});
describe('Plan your week prompt', () => {
 it('offers last week’s review at the start of a planned week, not mid-week', () => {
  expect(planPrompt(at({day: '2026-11-16'}))).toEqual({review: '2026-11-09'});
  expect(planPrompt(at({}))).toBeNull();
 });
 it('stays away during wind-down and when turned off or dismissed', () => {
  expect(planPrompt(at({day: '2026-11-16', clock: 'winddown'}))).toBeNull();
  expect(planPrompt(at({day: '2026-11-16', planReminder: false}))).toBeNull();
  expect(planPrompt(at({day: '2026-11-16', planWeeks: {'2026-W47': 'dismissed'}}))).toBeNull();
 });
});

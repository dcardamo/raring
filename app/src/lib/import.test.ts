import { describe, expect, it } from 'vitest';
import { importChanges, importDone, importRowText, importSummary } from './import';
import type { ImportPreview } from './generated/ImportPreview';

const preview: ImportPreview = {
  exportedAt: '2025-10-19T08:12:00+02:00',
  appVersion: '0.1.30',
  from: '2024-11-04',
  to: '2025-11-04',
  rows: [
    { kind: 'Goals', added: 2, replaced: 1, kept: 0 },
    { kind: 'Your sessions', added: 0, replaced: 0, kept: 3 },
    { kind: 'Events', added: 1, replaced: 0, kept: 2 },
  ],
  skipped: [{ what: 'Garmin data and journal logs', reason: 'Not imported.' }],
};

describe('import preview text', () => {
  it('counts only what changes', () => {
    expect(importChanges(preview)).toBe(4);
    expect(importRowText(preview.rows[0])).toBe('2 new · 1 replaced');
    expect(importRowText(preview.rows[1])).toBe('3 already here');
  });

  it('names the export, its date range and the change count', () => {
    // A year of plans never reads as one day.
    expect(importSummary(preview)).toBe('Exported 19 Oct 2025 · Raring 0.1.30 · 4 Nov 2024 – 4 Nov 2025 · 4 changes');
  });

  it('says when nothing would change', () => {
    const unchanged: ImportPreview = { ...preview, rows: [{ kind: 'Goals', added: 0, replaced: 0, kept: 2 }] };
    expect(importSummary(unchanged)).toContain('nothing here changes this phone');
    expect(importDone(preview)).toBe('Settings imported: 3 added, 1 replaced, 5 already here.');
  });

  it('handles an export without dates', () => {
    const bare: ImportPreview = { exportedAt: null, appVersion: null, from: null, to: null, rows: [], skipped: [] };
    expect(importSummary(bare)).toBe('This export · nothing here changes this phone');
  });
});

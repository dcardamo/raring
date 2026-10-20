import type { ImportPreview } from './generated/ImportPreview';
import type { ImportRow } from './generated/ImportRow';
import { yearDate } from './format';

// The preview's counts, in the words Settings uses. Added and replaced are
// changes; kept rows are already here or newer on this phone.
export const importChanges = (preview: ImportPreview): number =>
  preview.rows.reduce((total, row) => total + row.added + row.replaced, 0);

export const importRowText = (row: ImportRow): string => {
  const parts: string[] = [];
  if (row.added) parts.push(`${row.added} new`);
  if (row.replaced) parts.push(`${row.replaced} replaced`);
  if (row.kept) parts.push(`${row.kept} already here`);
  return parts.length ? parts.join(' · ') : 'nothing changes';
};

// Years throughout: an export can be a year old and span more than a year.
export const importSummary = (preview: ImportPreview): string => {
  const when = preview.exportedAt
    ? `Exported ${yearDate(preview.exportedAt.slice(0, 10))}${preview.appVersion ? ` · Raring ${preview.appVersion}` : ''}`
    : 'This export';
  const range = preview.from && preview.to
    ? preview.from === preview.to ? yearDate(preview.from) : `${yearDate(preview.from)} – ${yearDate(preview.to)}`
    : null;
  const changes = importChanges(preview);
  const head = changes === 0
    ? 'nothing here changes this phone'
    : `${changes} ${changes === 1 ? 'change' : 'changes'}`;
  return [when, range, head].filter((part): part is string => part !== null).join(' · ');
};

export const importDone = (preview: ImportPreview): string => {
  const sum = (pick: (row: ImportRow) => number) => preview.rows.reduce((total, row) => total + pick(row), 0);
  return `Settings imported: ${sum(row => row.added)} added, ${sum(row => row.replaced)} replaced, ${sum(row => row.kept)} already here.`;
};

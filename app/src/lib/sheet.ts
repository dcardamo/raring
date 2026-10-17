// Android's back button closes an open sheet before it navigates.
let closeOpen: (() => void) | null = null;

export function sheetOpened(close: () => void): () => void {
  closeOpen = close;
  return () => { if (closeOpen === close) closeOpen = null; };
}

export function closeSheet(): boolean {
  if (!closeOpen) return false;
  const close = closeOpen;
  closeOpen = null;
  close();
  return true;
}

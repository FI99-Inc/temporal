// Presentation geometry only. Classification, opportunity, fit, and pressure
// arrive from Rust. Packing moves labels into lanes, never their time stems.
const SOFTENING_MS = 6 * 3_600_000;

export function position(at: number, now: number, end: number): number {
  if (![at, now, end].every(Number.isFinite) || end <= now) throw new RangeError('Invalid Horizon extent');
  const elapsed = Math.max(0, Math.min(end - now, at - now));
  return Math.log1p(elapsed / SOFTENING_MS) / Math.log1p((end - now) / SOFTENING_MS);
}

export function pack<T extends { id: string; start: number; end: number | null }>(items: T[], now: number, end: number, width: number) {
  const lanes: number[] = [];
  return [...items].sort((a, b) => a.start - b.start || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0)).map(item => {
    const x = position(item.start, now, end) * width;
    const barEnd = position(item.end ?? item.start, now, end) * width;
    const labelWidth = Math.min(width, Math.max(142, barEnd - x));
    const labelX = Math.min(x, width - labelWidth);
    let lane = lanes.findIndex(last => last + 8 <= labelX);
    if (lane < 0) lane = lanes.length;
    lanes[lane] = labelX + labelWidth;
    return { item, x, barEnd, labelX, width: labelWidth, lane };
  });
}

export function visibleTicks<T extends { at: number }>(ticks: T[], now: number, end: number, width: number): T[] {
  const ordered = [...ticks].filter(t => t.at >= now && t.at <= end).sort((a, b) => a.at - b.at);
  const last = ordered.at(-1);
  const shown: T[] = [];
  for (const tick of ordered) {
    const x = position(tick.at, now, end) * width;
    const previous = shown.at(-1);
    if ((!previous || x - position(previous.at, now, end) * width >= 64)
      && (tick === last || !last || (position(last.at, now, end) * width - x >= 64))) shown.push(tick);
  }
  return shown;
}

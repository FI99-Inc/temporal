import test from 'node:test';
import assert from 'node:assert/strict';
import { position, pack, visibleTicks } from '../src/lib/horizon.ts';

const hour = 3_600_000;

test('Horizon preserves endpoints and temporal order while near time has more space', () => {
  assert.equal(position(0, 0, 336 * hour), 0);
  assert.equal(position(336 * hour, 0, 336 * hour), 1);
  let previous = -1;
  for (let i = 0; i <= 336; i++) {
    const next = position(i * hour, 0, 336 * hour);
    assert.ok(next > previous);
    previous = next;
  }
  assert.ok(position(hour, 0, 336 * hour) > position(169 * hour, 0, 336 * hour) - position(168 * hour, 0, 336 * hour));
  assert.ok(Math.abs(position(hour + 1, 0, 336 * hour) - position(hour, 0, 336 * hour)) < 0.00001);
  assert.equal(position(-hour, 0, 336 * hour), 0);
  assert.equal(position(400 * hour, 0, 336 * hour), 1);
  assert.throws(() => position(0, 1, 1));
});

test('clock translation is deterministic and an approaching object moves toward NOW', () => {
  assert.equal(position(7 * hour, 5 * hour, 341 * hour), position(2 * hour, 0, 336 * hour));
  assert.ok(position(24 * hour, hour, 336 * hour) < position(24 * hour, 0, 336 * hour));
});

test('packing prevents label collisions without moving temporal coordinates or changing tie order', () => {
  const items = [
    { id: 'b', start: 2 * hour, end: 3 * hour },
    { id: 'a', start: 2 * hour, end: 4 * hour },
    { id: 'c', start: 24 * hour, end: 25 * hour },
  ];
  const before = structuredClone(items);
  const rows = pack(items, 0, 336 * hour, 800);
  assert.deepEqual(rows, pack([...items].reverse(), 0, 336 * hour, 800));
  assert.deepEqual(items, before);
  for (const row of rows) {
    assert.equal(row.x, position(row.item.start, 0, 336 * hour) * 800);
    assert.ok(row.labelX >= 0 && row.labelX + row.width <= 800);
    for (const other of rows.filter(r => r.lane === row.lane && r !== row)) {
      assert.ok(row.labelX + row.width + 8 <= other.labelX || other.labelX + other.width + 8 <= row.labelX);
    }
  }
});

test('axis labels remain ordered and do not overlap at narrow or wide widths', () => {
  const ticks = [0, 1, 3, 6, 24, 72, 168, 336].map(h => ({ at: h * hour, label: String(h) }));
  for (const width of [360, 800, 1200]) {
    const shown = visibleTicks(ticks, 0, 336 * hour, width);
    assert.equal(shown[0].at, 0);
    assert.equal(shown.at(-1)!.at, 336 * hour);
    assert.ok(shown.every((t, i) => i === 0 || (position(t.at, 0, 336 * hour) - position(shown[i - 1].at, 0, 336 * hour)) * width >= 64));
  }
});

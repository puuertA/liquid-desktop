import test from 'node:test';
import assert from 'node:assert/strict';
import {monthCells} from '../src/widgets/calendar.mjs';
test('leap-year February has 29 days and Monday-first offset',()=>{
  const cells=monthCells(2024,1);
  assert.equal(cells.length,42);
  assert.equal(cells.indexOf(1),3);
  assert.deepEqual(cells.filter(v=>v!==null),Array.from({length:29},(_,i)=>i+1));
});
test('December navigation into next year uses actual dates',()=>{
  assert.deepEqual(monthCells(2024,12),monthCells(2025,0));
});
test('month starting on Sunday reserves six leading cells',()=>{
  const cells=monthCells(2026,10);
  assert.equal(cells.indexOf(1),6);
  assert.equal(cells.filter(v=>v!==null).length,30);
});

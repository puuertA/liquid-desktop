import {test} from 'node:test';
import assert from 'node:assert/strict';
import {rate,percent,memoryPercent,tintColor} from '../src/topbar/format.mjs';
test('unavailable or warming counters remain unknown rather than a fake zero',()=>{
  for(const value of [null,undefined,NaN]){assert.equal(rate(value),'—');assert.equal(percent(value),'—')}
  assert.equal(memoryPercent({memory_total:0,memory_used:0}),'—');
  assert.equal(rate(0),'0.0 KiB/s');assert.equal(percent(0),'0%');
});
test('real metrics retain units and memory uses the reported total',()=>{
  assert.equal(rate(1536),'1.5 KiB/s');assert.equal(rate(1572864),'1.5 MiB/s');
  assert.equal(memoryPercent({memory_used:8*1024**3,memory_total:32*1024**3}),'25%');
  assert.equal(percent(37.8),'38%');
});
test('glass colors accept only hex and bounded opacity',()=>{
  assert.equal(tintColor('#aAbBcC',.5),'rgba(170,187,204,0.5)');
  assert.equal(tintColor('url(unsafe)',0),'rgba(24,35,45,0.35)');
  assert.equal(tintColor('#000000',NaN),'rgba(0,0,0,0.82)');
});

import {test} from 'node:test';
import assert from 'node:assert/strict';
import {springSample} from '../src/glass/spring.mjs';
test('retargeting a moving spring has no position or velocity discontinuity',()=>{
  const moving=springSample(1,0,.99,.05);
  const reversed=springSample(moving.value,moving.velocity,1,0);
  assert.equal(reversed.value,moving.value);assert.ok(Math.abs(reversed.velocity-moving.velocity)<1e-10);
});
test('spring converges without a perpetual animation',()=>{
  let state={value:1,velocity:0};for(let i=0;i<90;i++)state=springSample(state.value,state.velocity,.99,1/60);
  assert.ok(Math.abs(state.value-.99)<.0001);assert.ok(Math.abs(state.velocity)<.001);
});

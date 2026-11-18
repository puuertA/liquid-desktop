import {test} from 'node:test';
import assert from 'node:assert/strict';
import {stepIsland,contentGeometry} from '../src/island/motion.mjs';
import {geometries} from '../src/island/state.mjs';
const initial=()=>({width:140,height:36,radius:18,vw:0,vh:0,vr:0});

test('island follows the same trajectory at 30, 60 and 144 Hz',()=>{
  const samples=[30,60,144].map(fps=>{let state=initial();for(let i=0;i<fps/2;i++)state=stepIsland(state,geometries.expanded,1/fps);return state});
  for(const state of samples)for(const key of Object.keys(state))assert.ok(Math.abs(state[key]-samples[0][key])<1e-8,key);
});
test('rapid reversals preserve momentum, fit the HWND and eventually settle',()=>{
  let state=initial();
  for(let frame=0;frame<150;frame++){
    const target=geometries[frame%18<9?'expanded':'idle'];
    const retargeted=stepIsland(state,target,0);
    for(const key of Object.keys(state))assert.ok(Math.abs(state[key]-retargeted[key])<1e-8,key);
    state=stepIsland(state,target,1/60);
    assert.ok(state.width>=100&&state.width<=460&&state.height>=28&&state.height<=330);
  }
  for(let frame=0;frame<120;frame++)state=stepIsland(state,geometries.idle,1/60);
  assert.ok(Math.abs(state.width-140)+Math.abs(state.height-36)+Math.abs(state.radius-18)<.001);
  assert.ok(Math.abs(state.vw)+Math.abs(state.vh)+Math.abs(state.vr)<.001);
});
test('header and cover geometry are continuous at compact and expanded boundaries',()=>{
  assert.deepEqual(contentGeometry(36),{expansion:0,topline:36});
  assert.deepEqual(contentGeometry(306),{expansion:1,topline:76});
  for(const boundary of [52,306]){
    const before=contentGeometry(boundary-.0001),after=contentGeometry(boundary+.0001);
    assert.ok(Math.abs(after.topline-before.topline)<.001);
    assert.ok(Math.abs(after.expansion-before.expansion)<.001);
  }
});

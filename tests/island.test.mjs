import {test} from 'node:test';
import assert from 'node:assert/strict';
import {islandState,geometries,geometryQueue,timeLabel,nextNotice} from '../src/island/state.mjs';
const base={expanded:false,notificationUntil:0,now:100,media:false,timer:false};
test('timer completion takes priority over volume until its notice expires',()=>{
  const completed=nextNotice({until:0,priority:0},'Timer',100,3);
  assert.equal(nextNotice(completed,'Volume',200,1),completed);
  assert.equal(nextNotice(completed,'Volume',5000,1).text,'Volume');
});
test('island activity arbitration restores previous concurrent activities after notification',()=>{
  assert.equal(islandState(base),'idle');
  assert.equal(islandState({...base,media:true}),'compact');
  assert.equal(islandState({...base,media:true,timer:true}),'multi');
  assert.equal(islandState({...base,media:true,timer:true,notificationUntil:200}),'notification');
  assert.equal(islandState({...base,media:true,timer:true,notificationUntil:200,now:201}),'multi');
  assert.equal(islandState({...base,expanded:true,notificationUntil:200}),'expanded');
});
test('native geometry queue coalesces intermediate frames and applies the final contour',async()=>{
  const calls=[];let release;const gate=new Promise(resolve=>release=resolve);
  const queue=geometryQueue(async value=>{calls.push(value);if(calls.length===1)await gate});
  const done=queue.set(1);await queue.set(2);await queue.set(3);release();await done;
  assert.deepEqual(calls,[1,3]);queue.dispose();await queue.set(4);assert.deepEqual(calls,[1,3]);
});
test('all target geometries fit the native validation bounds',()=>{
  for(const [w,h,r] of Object.values(geometries)){assert.ok(w>=100&&w<=460);assert.ok(h>=28&&h<=330);assert.ok(r>=0&&r<=80)}
  assert.equal(timeLabel(61),'1:01');assert.equal(timeLabel(-1),'0:00');
});

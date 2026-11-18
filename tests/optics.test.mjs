import {test} from 'node:test';
import assert from 'node:assert/strict';
import {rasterizeLens,lensProfile,roundedDistance,adaptiveShade,relativeLuminance} from '../src/glass/optics.mjs';
test('convex lens has a clear center, continuous rim and a positive interior peak',()=>{
  assert.equal(lensProfile(0,18,26),0);assert.equal(lensProfile(1,18,26),0);
  assert.ok(lensProfile(.3,18,26)>lensProfile(.99,18,26));
  assert.ok(lensProfile(.3,30,26)>lensProfile(.3,8,26));
});
test('map samples inward and is mirror symmetric within 8-bit rounding',()=>{
  const r=rasterizeLens({width:360,height:220,cap:640});
  const pixel=(x,y)=>Array.from(r.data.slice((y*r.width+x)*4,(y*r.width+x)*4+4));
  const mid=Math.floor(r.height/2),left=Math.round(12*r.width/360),right=r.width-1-left;
  assert.deepEqual(pixel(Math.floor(r.width/2),mid),[128,128,128,255]);
  assert.ok(pixel(left,mid)[0]>128);assert.ok(pixel(right,mid)[0]<128);
  assert.equal(pixel(left,mid)[0]+pixel(right,mid)[0],255);
  assert.ok(r.computed<r.width*r.height/2);
});
test('high DPI maps retain physical detail without exceeding the cap',()=>{
  const r=rasterizeLens({width:360,height:220,cap:720});
  assert.equal(r.width,720);assert.equal(r.height,440);
});
test('tiny capsule and large resized surfaces remain finite and bounded',()=>{
  for(const [width,height] of [[120,32],[640,600],[200,180]]){
    const r=rasterizeLens({width,height,radius:32,bezel:48,thickness:40,cap:256});
    assert.ok(r.width<=256&&r.height<=256);assert.ok(Number.isFinite(r.peak));
    assert.equal(r.data.length,r.width*r.height*4);
    assert.ok(roundedDistance(width/2,height/2,width,height,Math.min(32,height/2)).distance<0);
  }
});
test('adaptive contrast does not weaken manual contrast and recognizes light backgrounds',()=>{
  assert.equal(relativeLuminance(0,0,0),0);assert.equal(relativeLuminance(255,255,255),1);
  assert.ok(adaptiveShade(1,.3)>adaptiveShade(0,.3));assert.equal(adaptiveShade(0,.65),.65);
  assert.ok(adaptiveShade(2,.3)<=.75);
});

import {test} from 'node:test';import assert from 'node:assert/strict';import {decodeLiveFrame} from '../src/glass/liveFrame.mjs';
test('binary frame preserves negative origins, RGBA pixels and application masks',()=>{
 const bytes=new Uint8Array(28),h=new DataView(bytes.buffer);[-1920,20,2,1,42].forEach((v,i)=>h.setInt32(i*4,v,true));bytes.set([200,30,10,255,0,0,0,0],20);
 for(const input of [bytes.buffer,Array.from(bytes)]){const r=decodeLiveFrame(input);assert.equal(r.x,-1920);assert.equal(r.sequence,42);assert.equal(r.width,2);assert.deepEqual([...r.pixels],[200,30,10,255,0,0,0,0]);}
});
test('truncated and oversized frames are rejected before GPU upload',()=>{
 assert.throws(()=>decodeLiveFrame(new ArrayBuffer(19)),/incompleto/);
 const b=new ArrayBuffer(20),h=new DataView(b);h.setInt32(8,1920,true);h.setInt32(12,1080,true);assert.throws(()=>decodeLiveFrame(b),/Dimensões/);
 h.setInt32(8,0x7fffffff,true);assert.throws(()=>decodeLiveFrame(b),/Dimensões/);
});

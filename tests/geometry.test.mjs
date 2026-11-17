import test from 'node:test';import assert from 'node:assert/strict';import {imageRect,union} from '../src/glass/geometry.mjs';
test('fill crops centrally on negative-origin monitor',()=>assert.deepEqual(imageRect(4,1000,1000,[-1920,0,1920,1080]),[-1920,-420,1920,1920]));
test('fit preserves letterboxing',()=>imageRect(3,1920,1080,[0,0,1000,1000]).forEach((v,i)=>assert.ok(Math.abs(v-[0,218.75,1000,562.5][i])<1e-9)));
test('stretch matches screen bounds',()=>assert.deepEqual(imageRect(2,100,50,[500,-100,800,600]),[500,-100,800,600]));
test('center keeps physical image size',()=>assert.deepEqual(imageRect(0,400,300,[0,0,800,600]),[200,150,400,300]));
test('span uses virtual desktop including upper monitors',()=>assert.deepEqual(union([[-1920,0,1920,1080],[0,-200,2560,1440]]),[-1920,-200,4480,1440]));

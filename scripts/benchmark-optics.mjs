import {performance} from 'node:perf_hooks';
import {rasterizeLens} from '../src/glass/optics.mjs';
// Previous 0.3.0 pixel loop, without DOM/PNG encoding; both timings exclude encoding.
function oldMap(width,height){
  const data=new Uint8ClampedArray(width*height*4);
  for(let y=0;y<height;y++)for(let x=0;x<width;x++){
    const qx=Math.abs(x-width/2)-(width/2-32),qy=Math.abs(y-height/2)-(height/2-32);
    const ax=Math.max(qx,0),ay=Math.max(qy,0),len=Math.hypot(ax,ay);
    const distance=len+Math.min(Math.max(qx,qy),0)-32;
    const strength=Math.pow(Math.max(0,1-Math.abs(distance)/26),2);
    const nx=len?ax/len:(qx>qy?1:0),ny=len?ay/len:(qy>=qx?1:0),i=(y*width+x)*4;
    data[i]=128+Math.sign(x-width/2)*nx*strength*110;
    data[i+1]=128+Math.sign(y-height/2)*ny*strength*110;data[i+2]=128;data[i+3]=255;
  }return data;
}
function bench(fn){for(let i=0;i<8;i++)fn();const times=[];for(let i=0;i<40;i++){const at=performance.now();fn();times.push(performance.now()-at)}times.sort((a,b)=>a-b);return {median_ms:+times[20].toFixed(3),p95_ms:+times[38].toFixed(3)}}
const results=[[360,220],[380,340],[640,600]].map(([width,height])=>({width,height,old:bench(()=>oldMap(width,height)),balanced:bench(()=>rasterizeLens({width,height,cap:384})),high:bench(()=>rasterizeLens({width,height,cap:640}))}));
console.log(JSON.stringify({runtime:process.version,platform:process.platform,samples:40,scope:'CPU raster only, not WebView/GPU/frame rate',results},null,2));

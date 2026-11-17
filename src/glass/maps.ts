import {rasterizeLens} from './optics.mjs';
type Geometry = {width:number;height:number;radius:number;bezel:number;thickness:number;cap:number};
type Entry = {url:string; bytes:number; generatedMs:number};
const maps = new Map<string,Entry>();
let hits=0, generations=0;
// Bounded per-WebView cache. Independent native windows do not share a JS heap.
export function lensMap(geometry:Geometry) {
  const key=JSON.stringify(geometry), cached=maps.get(key);
  if(cached){maps.delete(key);maps.set(key,cached);hits++;return cached.url}
  const start=performance.now(), raster=rasterizeLens(geometry);
  const canvas=document.createElement('canvas');canvas.width=raster.width;canvas.height=raster.height;
  const context=canvas.getContext('2d');if(!context)throw Error('Canvas 2D indisponível');
  context.putImageData(new ImageData(raster.data,raster.width,raster.height),0,0);
  const url=canvas.toDataURL('image/png');
  maps.set(key,{url,bytes:url.length*2,generatedMs:performance.now()-start});generations++;
  while(maps.size>8)maps.delete(maps.keys().next().value!);
  return url;
}
export function mapMetrics(){return {entries:maps.size,hits,generations,bytes:[...maps.values()].reduce((n,e)=>n+e.bytes,0),generatedMs:[...maps.values()].reduce((n,e)=>n+e.generatedMs,0)}}

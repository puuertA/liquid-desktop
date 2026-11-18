import {springSample} from '../glass/spring.mjs';

// Position and velocity survive retargeting; no delayed height start.
export function stepIsland(value,target,dt) {
  const width=springSample(value.width,value.vw,target[0],dt,18,.9);
  const height=springSample(value.height,value.vh,target[1],dt,15.5,.92);
  const radius=springSample(value.radius,value.vr,target[2],dt,18,.9);
  return {width:Math.max(100,Math.min(460,width.value)),height:Math.max(28,Math.min(330,height.value)),radius:Math.max(0,Math.min(80,radius.value)),vw:width.velocity,vh:height.velocity,vr:radius.velocity};
}
export function contentGeometry(height) {
  const expansion=Math.max(0,Math.min(1,(height-52)/254));
  return {expansion,topline:Math.min(52,height)+24*expansion};
}

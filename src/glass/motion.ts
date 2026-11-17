import gsap from 'gsap';
import {springSample} from './spring.mjs';
type Motion={x:number;y:number;vx:number;vy:number;tx:number;ty:number;at:number;running:boolean;tick:()=>void};
const motions=new WeakMap<HTMLElement,Motion>();
// GSAP schedules only moving surfaces; retargeting preserves current velocity.
export function pressSurface(element:HTMLElement, pressed:boolean, enabled:boolean) {
  if(!enabled||matchMedia('(prefers-reduced-motion: reduce)').matches){stopSurface(element);return}
  let motion=motions.get(element);
  if(!motion){
    const m:Motion={x:1,y:1,vx:0,vy:0,tx:1,ty:1,at:performance.now(),running:false,tick:()=>{}};
    m.tick=()=>{
      const now=performance.now(),dt=(now-m.at)/1000;m.at=now;
      const x=springSample(m.x,m.vx,m.tx,dt),y=springSample(m.y,m.vy,m.ty,dt);
      m.x=x.value;m.vx=x.velocity;m.y=y.value;m.vy=y.velocity;
      gsap.set(element,{scaleX:m.x,scaleY:m.y});
      if(Math.abs(m.x-m.tx)+Math.abs(m.y-m.ty)<.0001&&Math.abs(m.vx)+Math.abs(m.vy)<.001){
        gsap.ticker.remove(m.tick);m.running=false;m.x=m.tx;m.y=m.ty;m.vx=m.vy=0;gsap.set(element,{scaleX:m.tx,scaleY:m.ty});
      }
    };
    motion=m;motions.set(element,m);
  }
  motion.tx=pressed?1.004:1;motion.ty=pressed?.99:1;
  if(!motion.running){motion.running=true;motion.at=performance.now();gsap.ticker.add(motion.tick)}
}
export function stopSurface(element:HTMLElement){const motion=motions.get(element);if(motion)gsap.ticker.remove(motion.tick);motions.delete(element);gsap.killTweensOf(element);gsap.set(element,{scaleX:1,scaleY:1})}

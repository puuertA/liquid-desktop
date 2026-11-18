import {useEffect,type RefObject} from 'react';
import gsap from 'gsap';

// Specular light follows the pointer without React renders or a permanent loop.
export function useGlassLight(root:RefObject<HTMLDivElement|null>,enabled:boolean){
  useEffect(()=>{
    const element=root.current;if(!element)return;
    const query=matchMedia('(prefers-reduced-motion: reduce)');
    let x=28,y=12,tx=x,ty=y,at=0,running=false;
    const paint=()=>{element.style.setProperty('--light-x',`${x}%`);element.style.setProperty('--light-y',`${y}%`)};
    const stop=()=>{gsap.ticker.remove(tick);running=false};
    const tick=()=>{
      const now=performance.now(),amount=1-Math.exp(-14*(now-at)/1000);at=now;
      x+=(tx-x)*amount;y+=(ty-y)*amount;paint();
      if(Math.abs(tx-x)+Math.abs(ty-y)<.05){x=tx;y=ty;paint();stop()}
    };
    const target=(nextX:number,nextY:number)=>{
      if(!enabled||query.matches)return;
      tx=nextX;ty=nextY;if(!running){at=performance.now();running=true;gsap.ticker.add(tick)}
    };
    const move=(e:PointerEvent)=>{const rect=element.getBoundingClientRect();target(Math.max(0,Math.min(100,(e.clientX-rect.left)/rect.width*100)),Math.max(0,Math.min(100,(e.clientY-rect.top)/rect.height*100)))};
    const leave=()=>target(28,12);
    const reset=()=>{stop();x=tx=28;y=ty=12;paint()};
    paint();element.addEventListener('pointermove',move);element.addEventListener('pointerleave',leave);window.addEventListener('blur',reset);query.addEventListener('change',reset);
    return()=>{stop();element.removeEventListener('pointermove',move);element.removeEventListener('pointerleave',leave);window.removeEventListener('blur',reset);query.removeEventListener('change',reset)};
  },[root,enabled]);
}

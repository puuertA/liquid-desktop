import {useEffect,useRef} from 'react';
import {invoke} from '@tauri-apps/api/core';
import type {Wallpaper} from '../types';
import type {FusionSnapshot,Shape} from './types';
import {vertex,fragment} from '../glass/fusionShader';
import {imageRect,union} from '../glass/geometry.mjs';
import {lensProfile} from '../glass/optics.mjs';
import {decodeLiveFrame} from '../glass/liveFrame.mjs';
export function FusionGlass({snapshot:s,shapes,onError,previewWallpaper}:{snapshot:FusionSnapshot;shapes:Shape[];onError:(e:string)=>void;previewWallpaper?:Wallpaper}){
  const canvas=useRef<HTMLCanvasElement>(null),render=useRef<(()=>void)|null>(null),latest=useRef({s,shapes});latest.current={s,shapes};
  useEffect(()=>{
    const context=canvas.current?.getContext('webgl2',{alpha:true,premultipliedAlpha:false,antialias:true,preserveDrawingBuffer:false});
    if(!context){onError('WebGL2 indisponível. Desative Widgets unidos para voltar às janelas individuais.');return}
    const gl:WebGL2RenderingContext=context;
    let disposed=false,textured=false,loading=false,lastWallpaper=0,wallpaperKey="",textureSource='';
    let textureRect=[0,0,s.width,s.height],liveTimer:ReturnType<typeof setTimeout>|undefined,failures=0,reported=false;
    const compile=(kind:number,source:string)=>{const shader=gl.createShader(kind)!;gl.shaderSource(shader,source);gl.compileShader(shader);if(!gl.getShaderParameter(shader,gl.COMPILE_STATUS)){const error=gl.getShaderInfoLog(shader);gl.deleteShader(shader);throw Error(error||'Shader inválido')}return shader};
    let program:WebGLProgram,buffer:WebGLBuffer,texture:WebGLTexture;
    try{const vs=compile(gl.VERTEX_SHADER,vertex),fs=compile(gl.FRAGMENT_SHADER,fragment);program=gl.createProgram()!;gl.attachShader(program,vs);gl.attachShader(program,fs);gl.linkProgram(program);gl.deleteShader(vs);gl.deleteShader(fs);if(!gl.getProgramParameter(program,gl.LINK_STATUS))throw Error(gl.getProgramInfoLog(program)||'Link WebGL falhou');
      gl.useProgram(program);buffer=gl.createBuffer()!;gl.bindBuffer(gl.ARRAY_BUFFER,buffer);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,1,-1,-1,1,-1,1,1,-1,1,1]),gl.STATIC_DRAW);const position=gl.getAttribLocation(program,'position');gl.enableVertexAttribArray(position);gl.vertexAttribPointer(position,2,gl.FLOAT,false,0,0);
      texture=gl.createTexture()!;gl.bindTexture(gl.TEXTURE_2D,texture);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,1,1,0,gl.RGBA,gl.UNSIGNED_BYTE,new Uint8Array([0,0,0,255]));
    }catch(e){onError(String(e));return}
    const location=(name:string)=>gl.getUniformLocation(program,name);
    function draw(){if(disposed||!canvas.current||gl.isContextLost())return;const {s,shapes}=latest.current,material=s.widgets[s.config.source];
      const ratio=Math.min(s.scale*(material.quality==='high'?1.5:1),gl.getParameter(gl.MAX_TEXTURE_SIZE)/Math.max(s.width,s.height)),w=Math.round(s.width*ratio),h=Math.round(s.height*ratio);
      if(canvas.current.width!==w||canvas.current.height!==h){canvas.current.width=w;canvas.current.height=h}gl.viewport(0,0,w,h);gl.useProgram(program);
      gl.uniform2f(location('size'),s.width,s.height);gl.uniform1i(location('count'),shapes.length);const rects=new Float32Array(20);shapes.forEach((r,i)=>rects.set([r.x,r.y,r.width,r.height],i*4));gl.uniform4fv(location('shapes[0]'),rects);gl.uniform1fv(location('radii[0]'),new Float32Array(Array.from({length:5},(_,i)=>shapes[i]?.radius??32)));gl.uniform1fv(location('topRadii[0]'),new Float32Array(Array.from({length:5},(_,i)=>shapes[i]?.topRadius??shapes[i]?.radius??32)));
      gl.uniform1i(location('textured'),textured&&textureSource===(material.static_wallpaper?'static':'live')?1:0);
      gl.uniform4f(location('textureRect'),textureRect[0],textureRect[1],textureRect[2],textureRect[3]);
      const hex=material.tint;gl.uniform3f(location('tint'),parseInt(hex.slice(1,3),16)/255,parseInt(hex.slice(3,5),16)/255,parseInt(hex.slice(5,7),16)/255);
      for(const [key,value]of Object.entries({opacity:material.opacity,shade:material.shade,shine:material.shine,strength:material.refraction,dispersion:material.quality==='economy'?0:material.dispersion,bezel:material.bezel,thickness:material.thickness,frost:material.frost,peak:Math.max(...Array.from({length:513},(_,i)=>lensProfile(i/512,material.thickness,material.bezel)))}))gl.uniform1f(location(key),value);
      gl.drawArrays(gl.TRIANGLES,0,6);
    }
    async function wallpaper(){if(disposed||loading||!latest.current.s.widgets[latest.current.s.config.source].static_wallpaper)return;const key=JSON.stringify([latest.current.s.screen,latest.current.s.config.source]);if(key===wallpaperKey&&Date.now()-lastWallpaper<15000)return;wallpaperKey=key;lastWallpaper=Date.now();loading=true;
      try{const wall=previewWallpaper??await invoke<Wallpaper>('get_wallpaper');const images=await Promise.all(wall.monitors.map(m=>new Promise<HTMLImageElement|null>(resolve=>{if(!m.image)return resolve(null);const img=new Image();img.onload=()=>resolve(img);img.onerror=()=>resolve(null);img.src=m.image;})));if(disposed)return;
        const {s}=latest.current,source=document.createElement('canvas');const density=Math.min(1,gl.getParameter(gl.MAX_TEXTURE_SIZE)/Math.max(s.screen[2],s.screen[3]));source.width=Math.round(s.screen[2]*density);source.height=Math.round(s.screen[3]*density);const ctx=source.getContext('2d')!;ctx.scale(density,density);ctx.fillStyle=`rgb(${wall.color&255},${wall.color>>8&255},${wall.color>>16&255})`;ctx.fillRect(0,0,s.screen[2],s.screen[3]);const bounds=union(wall.monitors.map(m=>m.rect));
        wall.monitors.forEach((m,i)=>{const img=images[i];if(!img)return;ctx.save();ctx.beginPath();ctx.rect(m.rect[0]-s.screen[0],m.rect[1]-s.screen[1],m.rect[2],m.rect[3]);ctx.clip();if(wall.mode===1){const pattern=ctx.createPattern(img,'repeat');if(pattern){ctx.translate(bounds[0]-s.screen[0],bounds[1]-s.screen[1]);ctx.fillStyle=pattern;ctx.fillRect(s.screen[0]-bounds[0],s.screen[1]-bounds[1],s.screen[2],s.screen[3])}}else{const [x,y,w,h]=imageRect(wall.mode,img.naturalWidth,img.naturalHeight,wall.mode===5?bounds:m.rect);ctx.drawImage(img,x-s.screen[0],y-s.screen[1],w,h)}ctx.restore()});
        gl.bindTexture(gl.TEXTURE_2D,texture);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,gl.RGBA,gl.UNSIGNED_BYTE,source);textureRect=[0,0,s.width,s.height];textureSource='static';textured=true;draw();
      }catch(e){onError(String(e))}finally{loading=false}
    }
    async function live(){
      if(disposed)return;const start=performance.now();let delay=33;
      try{
        const {s,shapes}=latest.current;
        if(!previewWallpaper&&!s.widgets[s.config.source].static_wallpaper&&!document.hidden&&shapes.length){
          delay=s.widgets[s.config.source].quality==='economy'?100:33;
          const x=Math.max(0,Math.floor(Math.min(...shapes.map(r=>r.x))-48)),y=Math.max(0,Math.floor(Math.min(...shapes.map(r=>r.y))-48));
          const right=Math.min(s.width,Math.ceil(Math.max(...shapes.map(r=>r.x+r.width))+48)),bottom=Math.min(s.height,Math.ceil(Math.max(...shapes.map(r=>r.y+r.height))+48));
          const data=await invoke<ArrayBuffer|number[]>('live_wallpaper_frame',{bounds:[x,y,right-x,bottom-y]});if(disposed)return;
          const frame=decodeLiveFrame(data);const {width,height}=frame;
          gl.bindTexture(gl.TEXTURE_2D,texture);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,width,height,0,gl.RGBA,gl.UNSIGNED_BYTE,frame.pixels);
          textureRect=[(frame.x-s.screen[0])/s.scale,(frame.y-s.screen[1])/s.scale,width/s.scale,height/s.scale];
          textureSource='live';textured=true;failures=0;if(reported){reported=false;onError('')}draw();
        }else{delay=250}
      }catch(e){textured=false;draw();delay=1000;if(++failures>=3&&!reported){reported=true;onError(String(e))}}
      finally{if(!disposed)liveTimer=setTimeout(()=>void live(),Math.max(0,delay-(performance.now()-start)))}
    }
    render.current=()=>{draw();void wallpaper()};draw();void wallpaper();void live();const timer=setInterval(()=>void wallpaper(),15000);
    const lost=(e:Event)=>{e.preventDefault();onError('O renderizador perdeu o contexto gráfico. Desative e reative Widgets unidos.')};canvas.current?.addEventListener('webglcontextlost',lost);
    return()=>{disposed=true;render.current=null;clearInterval(timer);clearTimeout(liveTimer);canvas.current?.removeEventListener('webglcontextlost',lost);gl.deleteTexture(texture);gl.deleteBuffer(buffer);gl.deleteProgram(program)};
  },[onError,previewWallpaper]);
  useEffect(()=>{render.current?.()},[s,shapes]);
  return <canvas className="fusion-glass" ref={canvas} aria-hidden="true"/>;
}

import {useEffect,useRef} from 'react';
import {invoke} from '@tauri-apps/api/core';
import type {Wallpaper} from '../types';
import type {FusionSnapshot,Shape} from './types';
import {vertex,fragment} from '../glass/fusionShader';
import {imageRect,union} from '../glass/geometry.mjs';
import {lensProfile} from '../glass/optics.mjs';
export function FusionGlass({snapshot:s,shapes,onError,previewWallpaper}:{snapshot:FusionSnapshot;shapes:Shape[];onError:(e:string)=>void;previewWallpaper?:Wallpaper}){
  const canvas=useRef<HTMLCanvasElement>(null),render=useRef<(()=>void)|null>(null),latest=useRef({s,shapes});latest.current={s,shapes};
  useEffect(()=>{
    const context=canvas.current?.getContext('webgl2',{alpha:true,premultipliedAlpha:false,antialias:true,preserveDrawingBuffer:false});
    if(!context){onError('WebGL2 indisponível. Desative Widgets unidos para voltar às janelas individuais.');return}
    const gl:WebGL2RenderingContext=context;
    let disposed=false,textured=false,loading=false,lastWallpaper=0,wallpaperKey="";
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
      gl.uniform1i(location('textured'),textured&&material.static_wallpaper?1:0);
      const hex=material.tint;gl.uniform3f(location('tint'),parseInt(hex.slice(1,3),16)/255,parseInt(hex.slice(3,5),16)/255,parseInt(hex.slice(5,7),16)/255);
      for(const [key,value]of Object.entries({opacity:material.opacity,shade:material.shade,shine:material.shine,strength:material.refraction,dispersion:material.quality==='economy'?0:material.dispersion,bezel:material.bezel,thickness:material.thickness,frost:material.frost,peak:Math.max(...Array.from({length:513},(_,i)=>lensProfile(i/512,material.thickness,material.bezel)))}))gl.uniform1f(location(key),value);
      gl.drawArrays(gl.TRIANGLES,0,6);
    }
    async function wallpaper(){if(disposed||loading||!latest.current.s.widgets[latest.current.s.config.source].static_wallpaper)return;const key=JSON.stringify([latest.current.s.screen,latest.current.s.config.source]);if(key===wallpaperKey&&Date.now()-lastWallpaper<15000)return;wallpaperKey=key;lastWallpaper=Date.now();loading=true;
      try{const wall=previewWallpaper??await invoke<Wallpaper>('get_wallpaper');const images=await Promise.all(wall.monitors.map(m=>new Promise<HTMLImageElement|null>(resolve=>{if(!m.image)return resolve(null);const img=new Image();img.onload=()=>resolve(img);img.onerror=()=>resolve(null);img.src=m.image;})));if(disposed)return;
        const {s}=latest.current,source=document.createElement('canvas');const density=Math.min(1,gl.getParameter(gl.MAX_TEXTURE_SIZE)/Math.max(s.screen[2],s.screen[3]));source.width=Math.round(s.screen[2]*density);source.height=Math.round(s.screen[3]*density);const ctx=source.getContext('2d')!;ctx.scale(density,density);ctx.fillStyle=`rgb(${wall.color&255},${wall.color>>8&255},${wall.color>>16&255})`;ctx.fillRect(0,0,s.screen[2],s.screen[3]);const bounds=union(wall.monitors.map(m=>m.rect));
        wall.monitors.forEach((m,i)=>{const img=images[i];if(!img)return;ctx.save();ctx.beginPath();ctx.rect(m.rect[0]-s.screen[0],m.rect[1]-s.screen[1],m.rect[2],m.rect[3]);ctx.clip();if(wall.mode===1){const pattern=ctx.createPattern(img,'repeat');if(pattern){ctx.translate(bounds[0]-s.screen[0],bounds[1]-s.screen[1]);ctx.fillStyle=pattern;ctx.fillRect(s.screen[0]-bounds[0],s.screen[1]-bounds[1],s.screen[2],s.screen[3])}}else{const [x,y,w,h]=imageRect(wall.mode,img.naturalWidth,img.naturalHeight,wall.mode===5?bounds:m.rect);ctx.drawImage(img,x-s.screen[0],y-s.screen[1],w,h)}ctx.restore()});
        gl.bindTexture(gl.TEXTURE_2D,texture);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,gl.RGBA,gl.UNSIGNED_BYTE,source);textured=true;draw();
      }catch(e){onError(String(e))}finally{loading=false}
    }
    render.current=()=>{draw();void wallpaper()};draw();void wallpaper();const timer=setInterval(()=>void wallpaper(),15000);
    const lost=(e:Event)=>{e.preventDefault();onError('O renderizador perdeu o contexto gráfico. Desative e reative Widgets unidos.')};canvas.current?.addEventListener('webglcontextlost',lost);
    return()=>{disposed=true;render.current=null;clearInterval(timer);canvas.current?.removeEventListener('webglcontextlost',lost);gl.deleteTexture(texture);gl.deleteBuffer(buffer);gl.deleteProgram(program)};
  },[onError,previewWallpaper]);
  useEffect(()=>{render.current?.()},[s,shapes]);
  return <canvas className="fusion-glass" ref={canvas} aria-hidden="true"/>;
}

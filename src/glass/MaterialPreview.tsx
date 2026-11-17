import {useCallback,useMemo,useState} from 'react';
import type {Settings,Wallpaper} from '../types';
import {LiquidGlass} from './LiquidGlass';
// Original optical test pattern. Explicitly a studio preview, never a substitute desktop wallpaper.
const pattern=`<svg xmlns="http://www.w3.org/2000/svg" width="480" height="280" viewBox="0 0 480 280"><rect width="480" height="280" fill="#8d9b8e"/><path d="M-50 280Q50-150 245 70T540 50V300Z" fill="#d7d2bd"/><path d="M-50 330Q130-140 290 115T550 120" fill="none" stroke="#b0bba7" stroke-width="70"/><g fill="none" stroke="#394e46" stroke-width="2" opacity=".6"><path d="M0 65H480M0 105H480M0 145H480M0 185H480M0 225H480M80 0V280M120 0V280M160 0V280M200 0V280M240 0V280M280 0V280M320 0V280M360 0V280M400 0V280"/></g></svg>`;
const uri=`data:image/svg+xml,${encodeURIComponent(pattern)}`;
const wallpaper:Wallpaper={mode:2,color:0,monitors:[{id:'studio',rect:[0,0,480,280],image:uri,error:null}]};
const rect=():[number,number,number,number]=>[80,50,320,180];
export function MaterialPreview({settings}:{settings:Settings}) {
  const [error,setError]=useState('');const report=useCallback((message:string)=>setError(message),[]);
  const preview=useMemo(()=>({...settings,width:320,height:180,x:80,y:50,static_wallpaper:true,edit:false}),[settings]);
  return <figure className="material-preview"><div className="material-stage" style={{backgroundImage:`url("${uri}")`}}><div className="material-specimen"><LiquidGlass settings={preview} wallpaper={wallpaper} previewRect={rect} onError={report}><div className="material-specimen-label"><span>Liquid Glass</span><strong>{settings.material==='custom'?'Seu material':settings.material}</strong><small>Mova o cursor sobre a lente</small></div></LiquidGlass></div></div><figcaption>{error||'Estúdio de material · textura ilustrativa refratada pelo motor real'}</figcaption></figure>;
}

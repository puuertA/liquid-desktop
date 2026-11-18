import {useEffect,useRef,useState} from 'react';
export function CommitSlider({label,value,min,max,disabled,onCommit}:{label:string;value:number;min:number;max:number;disabled:boolean;onCommit:(value:number)=>void}){
  const [draft,setDraft]=useState(value),editing=useRef(false);
  useEffect(()=>{if(!editing.current)setDraft(value)},[value]);
  function commit(){if(!editing.current)return;editing.current=false;onCommit(draft)}
  return <input aria-label={label} type="range" min={min} max={max} step="1" value={Math.max(min,Math.min(max,draft))} disabled={disabled}
    onChange={e=>setDraft(+e.target.value)} onPointerDown={()=>editing.current=true} onPointerUp={commit}
    onPointerCancel={()=>{editing.current=false;setDraft(value)}} onBlur={commit}
    onKeyDown={e=>{if(['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','Home','End','PageUp','PageDown'].includes(e.key))editing.current=true}}
    onKeyUp={commit}/>;
}

import {useEffect,useRef,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import type {Settings} from '../types';
import {MarkdownPreview} from './MarkdownPreview';
export function NotesWidget({initial,settings}:{initial:string;settings:Settings}) {
  const [text,setText]=useState(initial),[status,setStatus]=useState('Salvo neste computador');
  const [preview,setPreview]=useState(false);
  const queue=useRef(Promise.resolve()),revision=useRef(0),dirty=useRef(false);
  useEffect(()=>{if(!dirty.current)setText(initial)},[initial]);
  function change(value:string) {
    setText(value);dirty.current=true;setStatus('Salvando…');const version=++revision.current;
    queue.current=queue.current.catch(()=>{}).then(()=>invoke('save_note',{text:value})).then(()=>{
      if(version===revision.current){dirty.current=false;setStatus('Salvo neste computador')}
    }).catch(error=>{if(version===revision.current)setStatus(`Não salvo: ${String(error)}`)});
  }
  return <div className={`notes-widget notes-${settings.notes_style}`}><div className="widget-heading"><h2>Notas rápidas</h2><div className="notes-actions" data-no-drag><span className="note-count">{text.length.toLocaleString('pt-BR')}</span><button onClick={()=>setPreview(v=>!v)}>{preview?'Editar':'Prévia'}</button></div></div>{preview?<MarkdownPreview text={text}/>:<textarea aria-label="Anotações" placeholder="Escreva uma ideia, um lembrete…" maxLength={8000} value={text} onChange={e=>change(e.target.value)} spellCheck/>}<small role="status">{status}</small></div>;
}

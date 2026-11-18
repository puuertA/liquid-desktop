export function rate(value){
  if(value==null||!Number.isFinite(value)||value<0)return '—';
  if(value>=1048576)return `${(value/1048576).toFixed(1)} MiB/s`;
  return `${(value/1024).toFixed(1)} KiB/s`;
}
export function percent(value){return value==null||!Number.isFinite(value)?'—':`${Math.round(Math.max(0,Math.min(100,value)))}%`}
export function memoryPercent(sample){return sample&&sample.memory_total>0?percent(sample.memory_used/sample.memory_total*100):'—'}
export function tintColor(hex,opacity){
  const color=/^#[\da-f]{6}$/i.test(hex)?hex:'#18232d';
  return `rgba(${parseInt(color.slice(1,3),16)},${parseInt(color.slice(3,5),16)},${parseInt(color.slice(5,7),16)},${Number.isFinite(opacity)?Math.max(.35,Math.min(1,opacity)):.82})`;
}

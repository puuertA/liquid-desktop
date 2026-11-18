export function islandState({expanded,notificationUntil,now,media,timer}) {
  if(expanded)return 'expanded';
  if(notificationUntil>now)return 'notification';
  if(media&&timer)return 'multi';
  if(media||timer)return 'compact';
  return 'idle';
}
export const geometries={idle:[140,36,18],compact:[320,52,26],multi:[390,56,28],notification:[340,62,28],expanded:[440,306,32]};
export function nextNotice(previous,text,now,priority=1){
  return previous.until>now&&previous.priority>priority?previous:{text,until:now+4500,priority};
}
export function timeLabel(seconds){const n=Math.max(0,Math.ceil(seconds));return `${Math.floor(n/60)}:${String(n%60).padStart(2,'0')}`}

// At most one native update in flight. The latest geometry always wins.
export function geometryQueue(send) {
  let pending=null,busy=false,disposed=false;
  async function drain(){if(busy||disposed)return;busy=true;try{while(pending&&!disposed){const value=pending;pending=null;await send(value)}}finally{busy=false}}
  return {set(value){pending=value;return drain()},dispose(){disposed=true;pending=null}};
}

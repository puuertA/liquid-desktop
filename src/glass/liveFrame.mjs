// Native frame protocol: physical x/y/width/height, sequence, then RGBA8.
export function decodeLiveFrame(payload) {
  const data=payload instanceof ArrayBuffer?payload:Uint8Array.from(payload).buffer;
  if(data.byteLength<20)throw Error('Frame incompleto');
  const header=new DataView(data),width=header.getInt32(8,true),height=header.getInt32(12,true);
  if(width<1||height<1||width*height>16000000||data.byteLength!==20+width*height*4)throw Error('Dimensões do frame inválidas');
  return {x:header.getInt32(0,true),y:header.getInt32(4,true),width,height,sequence:header.getUint32(16,true),pixels:new Uint8Array(data,20)};
}

// All geometry is in physical screen pixels, including negative monitor origins.
export function imageRect(mode, imageWidth, imageHeight, rect) {
  const [x,y,w,h]=rect;
  if(mode===2) return [x,y,w,h];
  const scale=mode===3?Math.min(w/imageWidth,h/imageHeight):mode===4||mode===5?Math.max(w/imageWidth,h/imageHeight):1;
  const iw=imageWidth*scale, ih=imageHeight*scale;
  return [x+(w-iw)/2,y+(h-ih)/2,iw,ih];
}
export function union(rects) {
  const x=Math.min(...rects.map(r=>r[0])),y=Math.min(...rects.map(r=>r[1]));
  return [x,y,Math.max(...rects.map(r=>r[0]+r[2]))-x,Math.max(...rects.map(r=>r[1]+r[3]))-y];
}

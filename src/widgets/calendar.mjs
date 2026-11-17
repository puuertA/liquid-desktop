export function monthCells(year, month, mondayFirst = true) {
  const first = new Date(year, month, 1);
  const offset = (first.getDay() + (mondayFirst ? 6 : 0)) % 7;
  const days = new Date(year, month + 1, 0).getDate();
  return Array.from({length:42},(_,index)=>index>=offset && index<offset+days ? index-offset+1 : null);
}

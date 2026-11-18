export const widgetIds = ['clock','analog','calendar','notes','system'] as const;
export type WidgetId = typeof widgetIds[number];
export type Settings = {
  x:number; y:number; width:number; height:number; enabled:boolean;
  refraction:number; opacity:number; edit:boolean; static_wallpaper:boolean;
  tint:string; shine:number; shade:number; show_seconds:boolean; hour24:boolean; note:string;
  material:'custom'|'clear'|'thin'|'regular'|'frosted'|'dark'|'adaptive'|'liquid';
  thickness:number; bezel:number; dispersion:number; frost:number;
  quality:'economy'|'balanced'|'high'; motion:boolean;
};
export type Layout = {version:number; sync_theme:boolean; island:import('./island/types').IslandConfig; topbar:import('./topbar/types').TopBarConfig; fusion:import('./fusion/types').FusionConfig; widgets:Record<WidgetId,Settings>};
export type Monitor = {id:string;rect:[number,number,number,number];image:string|null;error:string|null};
export type Wallpaper = {mode:number;color:number;monitors:Monitor[]};
export type SystemSnapshot = {cpu:number|null;memory_used:number;memory_total:number;download:number|null;upload:number|null;interfaces:number;sequence:number};

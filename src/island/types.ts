export type IslandConfig={static_wallpaper:boolean;enabled:boolean;form:'floating'|'notch';appearance:'liquid'|'black'|'graphite'|'adaptive';above_apps:boolean;monitor:number;motion:boolean};
export type Track={id:string;app:string;title:string;artist:string;artwork:string|null;playing:boolean;position:number;start:number;end:number;toggle:boolean;previous:boolean;next:boolean;seek:boolean};
export type IslandSnapshot={media:{sessions:{id:string;app:string}[];track:Track|null}|null;media_error:string|null;volume:{level:number;muted:boolean}|null;volume_error:string|null;remaining:number|null};
export const defaultIsland:IslandConfig={static_wallpaper:false,enabled:false,form:'floating',appearance:'black',above_apps:true,monitor:0,motion:true};

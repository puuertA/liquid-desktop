import type {Settings} from '../types';
export type TaskbarMode='coexist'|'overlay'|'experimental';
export type TaskbarWidth='compact'|'adaptive'|'full';
export type TaskbarSegment='start'|'apps'|'system';
export type TaskbarConfig={enabled:boolean;mode:TaskbarMode;monitor:number;width:TaskbarWidth;height:number;radius:number;material:'shared'|'clear'|'regular'|'frosted'|'dark'|'adaptive';opacity:number;refraction:number;icon_size:number;magnification:number;auto_hide:boolean;hide_fullscreen:boolean;show_labels:boolean;show_search:boolean;show_task_view:boolean;show_weather:boolean;show_system:boolean;show_clock:boolean;show_bluetooth:boolean;show_clipboard:boolean;show_media:boolean;windows_auto_hide:boolean;segment_order:TaskbarSegment[];pinned:string[];hidden_pins:string[]};
export type TaskbarApp={id:string;label:string;path:string;icon:string|null;pinned:boolean;running:boolean;active:boolean;minimized:boolean;windows:number[];window_titles:string[]};
export type TaskbarSnapshot={config:TaskbarConfig;apps:TaskbarApp[];volume:{level:number;muted:boolean}|null;battery:number|null;charging:boolean;network:boolean;media:string|null};
export type MaterialSource=Settings;

import type {SystemSnapshot} from '../types';
export type TopBarConfig={enabled:boolean;monitor:number;tint:string;opacity:number;above_apps:boolean;hide_fullscreen:boolean;show_seconds:boolean;show_date:boolean;show_cpu:boolean;show_memory:boolean;show_network:boolean;show_volume:boolean};
export type TopBarSnapshot={config:TopBarConfig;shared_island:boolean;system:SystemSnapshot|null;volume:{level:number;muted:boolean}|null;volume_error:string|null};

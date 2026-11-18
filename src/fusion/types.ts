import type {Settings,WidgetId} from '../types';
export type FusionConfig={enabled:boolean;monitor:number;source:WidgetId;positions:Partial<Record<WidgetId,[number,number]>>};
export type Shape={id:WidgetId;x:number;y:number;width:number;height:number};
export type FusionSnapshot={config:FusionConfig;widgets:Record<WidgetId,Settings>;shapes:Shape[];screen:[number,number,number,number];scale:number;width:number;height:number};

import type {Settings,WidgetId} from '../types';
import {DigitalClock,AnalogClock} from './ClockWidgets';
import {CalendarWidget} from './CalendarWidget';
import {NotesWidget} from './NotesWidget';
import {SystemWidget} from './SystemWidget';
import {MediaWidget} from './MediaWidget';
import {WeatherWidget} from './WeatherWidget';
import {GalleryWidget} from './GalleryWidget';
import {StorageWidget} from './StorageWidget';
export function WidgetContent({id,settings}:{id:WidgetId;settings:Settings}){
  switch(id){
    case 'clock':return <DigitalClock settings={settings}/>;
    case 'analog':return <AnalogClock settings={settings}/>;
    case 'calendar':return <CalendarWidget settings={settings}/>;
    case 'notes':return <NotesWidget initial={settings.note} settings={settings}/>;
    case 'system':return <SystemWidget settings={settings}/>;
    case 'media':return <MediaWidget settings={settings}/>;
    case 'weather':return <WeatherWidget settings={settings}/>;
    case 'gallery':return <GalleryWidget settings={settings}/>;
    case 'storage':return <StorageWidget settings={settings}/>;
  }
}

import type {WidgetId} from '../types';
export const catalog: Record<WidgetId,{name:string;description:string;tag:string;size:[number,number]}> = {
  clock:{name:'Relógio digital',description:'A hora, sem distrações.',tag:'Hora local',size:[360,220]},
  analog:{name:'Relógio analógico',description:'Ponteiros e um ritmo mais calmo.',tag:'Hora local',size:[240,260]},
  calendar:{name:'Calendário',description:'O mês inteiro à vista.',tag:'Navegação mensal',size:[380,320]},
  notes:{name:'Notas rápidas',description:'Ideias que ficam no desktop.',tag:'Salvo neste computador',size:[360,280]},
  system:{name:'Monitor do sistema',description:'CPU, memória e tráfego de rede.',tag:'Dados reais · 2 s',size:[380,340]},
  media:{name:'Media Player',description:'Controle a mídia em reprodução.',tag:'Sessões do Windows',size:[380,240]},
  weather:{name:'Clima',description:'Condições e previsão da sua cidade.',tag:'Open-Meteo',size:[400,360]},
  gallery:{name:'Galeria',description:'Suas fotos no desktop.',tag:'Imagens locais',size:[400,300]},
  storage:{name:'Armazenamento',description:'Espaço das unidades locais.',tag:'Discos reais',size:[380,300]},
};
export function WidgetIcon({id}:{id:WidgetId}) {
  const paths:Record<WidgetId,string>={
    clock:'M4 5h16v14H4z M8 9v6 M12 9v6 M16 9v6',
    analog:'M12 8v4l3 2 M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18',
    calendar:'M4 5h16v16H4z M4 10h16 M8 3v4 M16 3v4 M8 14h2 M14 14h2 M8 17h2',
    notes:'M5 3h14v18H5z M8 8h8 M8 12h8 M8 16h5',
    system:'M3 4h18v14H3z M8 22h8 M12 18v4 M5 12h3l2-4 4 7 2-4h3',
    media:'M9 5v14l11-7z M4 5v14',
    weather:'M7 17h11a4 4 0 0 0 0-8 6 6 0 0 0-11-1 4.5 4.5 0 0 0 0 9z',
    gallery:'M3 4h18v16H3z M5 17l5-5 3 3 3-4 4 5 M8 8h1',
    storage:'M4 6h16v12H4z M4 13h16 M8 16h1 M12 16h1',
  };
  return <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d={paths[id]}/></svg>;
}

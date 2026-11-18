import type {Settings} from '../types';
export const materials = {
  liquid: {name:'Liquid Prism',description:'Lente espessa e separação RGB nas bordas.',opacity:.05,shade:.22,shine:.85,refraction:32,thickness:28,bezel:32,dispersion:.4,frost:.25},
  clear: {name:'Clear',description:'Centro cristalino, borda convexa.',opacity:.06,shade:.3,shine:.7,refraction:24,thickness:22,bezel:24,dispersion:.12,frost:0},
  thin: {name:'Thin',description:'Uma lente leve e discreta.',opacity:.08,shade:.32,shine:.4,refraction:10,thickness:8,bezel:14,dispersion:.06,frost:.3},
  regular: {name:'Regular',description:'Equilíbrio entre profundidade e leitura.',opacity:.12,shade:.42,shine:.65,refraction:18,thickness:18,bezel:26,dispersion:.1,frost:.5},
  frosted: {name:'Frosted',description:'Difusão para superfícies com mais conteúdo.',opacity:.18,shade:.48,shine:.5,refraction:18,thickness:22,bezel:30,dispersion:.08,frost:5},
  dark: {name:'Dark',description:'Vidro escurecido, reflexos contidos.',opacity:.1,shade:.7,shine:.4,refraction:16,thickness:18,bezel:24,dispersion:.08,frost:1.5},
  adaptive: {name:'Adaptive',description:'Contraste conforme o wallpaper estático.',opacity:.08,shade:.3,shine:.6,refraction:20,thickness:20,bezel:26,dispersion:.1,frost:.5},
} as const;
export type Material = keyof typeof materials | 'custom';
export function materialPatch(material:keyof typeof materials):Partial<Settings> {
  const {name:_,description:__,...values}=materials[material];return {...values,material};
}

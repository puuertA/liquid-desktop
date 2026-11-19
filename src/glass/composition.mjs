// One optical film for captured pixels and compositor-backed transparent pixels.
export function filmParameters(opacity,shade,tint){
 const transmission=(1-opacity)*(1-shade*.7);
 return {alpha:1-transmission,premultiplied:tint.map(c=>c*opacity*(1-shade*.7))};
}
export function fitLens(material,shapes){
 const extent=Math.min(...shapes.map(s=>Math.min(s.width,s.height)));
 const bezel=Math.min(material.bezel,Math.max(1,extent*.45));
 const ratio=bezel/material.bezel;
 return {bezel,thickness:material.thickness*ratio,strength:material.refraction*ratio};
}

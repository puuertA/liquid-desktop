import {test} from 'node:test';import assert from 'node:assert/strict';import {filmParameters,fitLens} from '../src/glass/composition.mjs';
test('masked glass and captured glass match over the same background, including saturated red',()=>{
 for(const opacity of [0,.05,.6])for(const shade of [.2,.42,.75])for(const tint of [[1,0,0],[1,1,1],[.2,.5,.9]])for(const background of [[0,0,0],[1,1,1],[.7,.1,.3]]){
  const film=filmParameters(opacity,shade,tint);
  for(let i=0;i<3;i++){const composited=film.premultiplied[i]/film.alpha*film.alpha+background[i]*(1-film.alpha);const captured=(background[i]*(1-opacity)+tint[i]*opacity)*(1-shade*.7);assert.ok(Math.abs(composited-captured)<1e-12);}
 }
});
test('thin bars and compact islands retain a clear center without changing widget optics',()=>{
 const m={bezel:32,thickness:28,refraction:32};
 const bar=fitLens(m,[{width:800,height:36}]);assert.equal(bar.bezel,16.2);assert.ok(bar.strength<18);assert.equal(bar.thickness/m.thickness,bar.bezel/m.bezel);
 assert.deepEqual(fitLens(m,[{width:360,height:220}]),{bezel:32,thickness:28,strength:32});
});

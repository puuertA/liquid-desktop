import fs from 'node:fs';
// Generate a local WebGL regression page; open it in a WebGL2 browser and check PASS.
fs.mkdirSync('.tools',{recursive:true});
const source=fs.readFileSync('src/glass/fusionShader.ts','utf8');
const vertex=source.match(/vertex=`([\s\S]*?)`;/)[1],fragment=source.match(/fragment=`([\s\S]*?)`;/)[1];
fs.writeFileSync('.tools/check-glass.html',`<body><pre id="result">pending</pre><canvas width="64" height="64"></canvas><script>
try {
const gl=document.querySelector('canvas').getContext('webgl2',{premultipliedAlpha:false});
const p=gl.createProgram();for(const [kind,source] of [[gl.VERTEX_SHADER,${JSON.stringify(vertex)}],[gl.FRAGMENT_SHADER,${JSON.stringify(fragment)}]]){const s=gl.createShader(kind);gl.shaderSource(s,source);gl.compileShader(s);if(!gl.getShaderParameter(s,gl.COMPILE_STATUS))throw gl.getShaderInfoLog(s);gl.attachShader(p,s)}gl.linkProgram(p);if(!gl.getProgramParameter(p,gl.LINK_STATUS))throw gl.getProgramInfoLog(p);gl.useProgram(p);
const buf=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,buf);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([-1,-1,1,-1,-1,1,-1,1,1,-1,1,1]),gl.STATIC_DRAW);const pos=gl.getAttribLocation(p,'position');gl.enableVertexAttribArray(pos);gl.vertexAttribPointer(pos,2,gl.FLOAT,false,0,0);
const u=n=>gl.getUniformLocation(p,n);gl.uniform2f(u('size'),64,64);gl.uniform1i(u('count'),1);gl.uniform4fv(u('shapes[0]'),new Float32Array([0,0,64,64]));gl.uniform1fv(u('radii[0]'),new Float32Array([8]));gl.uniform1fv(u('topRadii[0]'),new Float32Array([8]));gl.uniform4f(u('textureRect'),0,0,64,64);gl.uniform1i(u('textured'),1);
gl.uniform3f(u('filmColor'),.043,.0086,.0043);gl.uniform1f(u('filmAlpha'),.183);for(const [n,v] of Object.entries({bezel:10,thickness:10,peak:1,strength:0,shine:0,frost:0,dispersion:0}))gl.uniform1f(u(n),v);
const t=gl.createTexture();gl.bindTexture(gl.TEXTURE_2D,t);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.NEAREST);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.NEAREST);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);
const pixels=new Uint8Array(64*64*4);for(let y=0;y<64;y++)for(let x=0;x<32;x++)pixels.set([100,20,10,255],(y*64+x)*4);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,64,64,0,gl.RGBA,gl.UNSIGNED_BYTE,pixels);gl.drawArrays(gl.TRIANGLES,0,6);
const a=new Uint8Array(4),b=new Uint8Array(4);gl.readPixels(20,32,1,1,gl.RGBA,gl.UNSIGNED_BYTE,a);gl.readPixels(44,32,1,1,gl.RGBA,gl.UNSIGNED_BYTE,b);const bg=[100,20,10];const delta=Math.max(...bg.map((c,i)=>Math.abs(a[i]*a[3]/255+c*(1-a[3]/255)-(b[i]*b[3]/255+c*(1-b[3]/255)))));if(delta>2||gl.getError())throw 'Mismatch '+delta;document.querySelector('#result').textContent='PASS: GLSL compiled; masked/captured seam delta='+delta.toFixed(3)+' /255';
}catch(e){document.querySelector('#result').textContent='FAIL '+e}
</script>`);

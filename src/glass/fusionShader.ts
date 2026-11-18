export const vertex=`#version 300 es
in vec2 position;out vec2 uv;void main(){uv=position*.5+.5;gl_Position=vec4(position,0,1);}`;
export const fragment=`#version 300 es
precision highp float;
in vec2 uv;out vec4 color;
uniform vec2 size;uniform vec4 shapes[5];uniform int count;uniform sampler2D backdrop;uniform bool textured;
uniform vec3 tint;uniform float opacity,shade,shine,strength,dispersion,bezel,thickness,peak,frost;
float field(vec2 p){float d=100000.;for(int i=0;i<5;i++){if(i>=count)break;vec4 r=shapes[i];vec2 q=abs(p-r.xy-r.zw*.5)-(r.zw*.5-32.);float n=length(max(q,0.))+min(max(q.x,q.y),0.)-32.;float h=max(1.-abs(d-n)/48.,0.);d=min(d,n)-h*h*12.;}return d;}
vec3 sampleBackground(vec2 p){vec2 v=clamp(p/size,0.,1.);vec2 b=vec2(frost)/size;return texture(backdrop,v).rgb*.5+(texture(backdrop,v+b).rgb+texture(backdrop,v-b).rgb)*.25;}
void main(){
 vec2 p=vec2(uv.x,1.-uv.y)*size;float d=field(p);float alpha=1.-smoothstep(-.7,.7,d);if(alpha<=0.){color=vec4(0);return;}
 vec2 grad=vec2(field(p+vec2(.7,0))-field(p-vec2(.7,0)),field(p+vec2(0,.7))-field(p-vec2(0,.7)));
 vec2 normal=grad/max(length(grad),.0001);float s=clamp(-d/bezel,.001,.999);
 float arc=sqrt(s*(2.-s)),slope=thickness/bezel*(1.-s)/arc,ci=inversesqrt(1.+slope*slope),si=slope*ci;
 float eta=1./1.5,ct=sqrt(1.-eta*eta*si*si),tangent=(eta*ci-ct)*si,vertical=-eta+(eta*ci-ct)*ci;
 float bend=thickness*arc*abs(tangent/vertical)/max(peak,.001)*strength*(1.-step(bezel,-d));
 vec3 base=tint;float a=min(.9,opacity+shade);
 if(textured){vec2 red=p-normal*bend*(1.-dispersion*.7),green=p-normal*bend,blue=p-normal*bend*(1.+dispersion*.7);
 base=vec3(sampleBackground(red).r,sampleBackground(green).g,sampleBackground(blue).b);base=mix(base,tint,opacity);base*=1.-shade*.7;a=1.;}
 else{base=mix(tint,vec3(.025,.04,.065),shade);}
 float rim=exp(-max(-d,0.)/1.3),inner=exp(-abs(d+5.)/4.);
 float spec=pow(max(dot(normal,normalize(vec2(-.55,-.8))),0.),5.)+pow(max(dot(normal,normalize(vec2(.4,1.))),0.),7.)*.45;
 vec3 spectral=vec3(.35+.3*normal.x,.55+.25*normal.y,.8-.25*normal.x);
 base+=shine*(vec3(spec)*(rim*.8+inner*.1)+spectral*rim*dispersion*.6);
 a=max(a,shine*rim*.7);color=vec4(base,alpha*a);
}`;

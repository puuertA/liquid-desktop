import type {ReactNode} from 'react';
const token=/(\*\*[^*]+\*\*|\*[^*]+\*|`[^`]+`|\[[^\]]+\]\(https?:\/\/[^\s)]+\))/g;
function inline(text:string):ReactNode[]{return text.split(token).filter(Boolean).map((part,i)=>{
  if(part.startsWith('**')&&part.endsWith('**'))return <strong key={i}>{part.slice(2,-2)}</strong>;
  if(part.startsWith('*')&&part.endsWith('*'))return <em key={i}>{part.slice(1,-1)}</em>;
  if(part.startsWith('`')&&part.endsWith('`'))return <code key={i}>{part.slice(1,-1)}</code>;
  const link=part.match(/^\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)$/);if(link)return <a key={i} href={link[2]} target="_blank" rel="noopener noreferrer">{link[1]}</a>;
  return part;
})}
export function MarkdownPreview({text}:{text:string}){return <div className="markdown-preview">{text.split('\n').map((line,i)=>{
  if(line.startsWith('### '))return <h4 key={i}>{inline(line.slice(4))}</h4>;
  if(line.startsWith('## '))return <h3 key={i}>{inline(line.slice(3))}</h3>;
  if(line.startsWith('# '))return <h2 key={i}>{inline(line.slice(2))}</h2>;
  if(line.startsWith('- '))return <p key={i} className="markdown-list">• {inline(line.slice(2))}</p>;
  return <p key={i}>{inline(line)||'\u00a0'}</p>;
})}</div>}

/* Builds the Whirlpool desktop (menu bar + dock) used by the website hero and the
   renders in site/renders. Magnification follows the cursor, as in the real dock. */
(function(){
const g=(id,a,b)=>`<linearGradient id="${id}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="${a}"/><stop offset="1" stop-color="${b}"/></linearGradient>`;
const ICONS=[
 {n:'Files',on:1,bg:'linear-gradient(145deg,#f7dc82,#e39a35)',svg:'<path d="M4 8.5a2 2 0 0 1 2-2h4l2 2.2h6a2 2 0 0 1 2 2V17a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z" fill="#fff" fill-opacity=".92"/>'},
 {n:'Browser',on:1,bg:'linear-gradient(145deg,#6be3e0,#0b6f86)',svg:'<circle cx="12" cy="12" r="8" fill="none" stroke="#fff" stroke-width="1.8"/><path d="M15.8 8.2l-2.1 5.5-5.5 2.1 2.1-5.5z" fill="#fff"/>'},
 {n:'Terminal',on:1,bg:'linear-gradient(145deg,#173a45,#061a21)',svg:'<path d="M6 8l4 4-4 4" fill="none" stroke="#9ff0e6" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M13 17h5" stroke="#9ff0e6" stroke-width="2" stroke-linecap="round"/>'},
 {n:'Mail',bg:'linear-gradient(145deg,#8fb7ff,#3c63d8)',svg:'<rect x="4" y="6.5" width="16" height="11" rx="2.2" fill="#fff" fill-opacity=".95"/><path d="M5 8l7 5 7-5" fill="none" stroke="#3c63d8" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>'},
 {n:'Music',bg:'linear-gradient(145deg,#ff9a6b,#e0483f)',svg:'<path d="M10 16.5V7.5l8-1.8V14" fill="none" stroke="#fff" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><circle cx="8.3" cy="16.6" r="2.2" fill="#fff"/><circle cx="16.3" cy="14.4" r="2.2" fill="#fff"/>'},
 {n:'Notes',bg:'linear-gradient(145deg,#fff1a6,#f2c94c)',svg:'<path d="M7 7h10M7 11h10M7 15h6" stroke="#8a6a12" stroke-width="1.8" stroke-linecap="round"/>'},
 {n:'Photos',bg:'linear-gradient(145deg,#f4f8f7,#cfe3e0)',svg:'<circle cx="12" cy="12" r="6.6" fill="none" stroke="#e5567a" stroke-width="2.6" stroke-dasharray="6.2 3.0"/><circle cx="12" cy="12" r="6.6" fill="none" stroke="#3bb7c7" stroke-width="2.6" stroke-dasharray="3 6.2" stroke-dashoffset="-9"/>'},
 {n:'Whirlpool Settings',img:(window.WP_BASE||'')+'assets/icon.png'},
 'sep',
 {n:'Downloads',bg:'linear-gradient(145deg,#4fd1c5,#0c7f8a)',svg:'<path d="M12 5v9m0 0l-3.5-3.5M12 14l3.5-3.5M6 18h12" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>'},
 {n:'Recycle Bin',bg:'linear-gradient(145deg,#c9dedc,#7f9fa3)',svg:'<path d="M7.5 8h9l-.8 9.2a1.6 1.6 0 0 1-1.6 1.4H9.9a1.6 1.6 0 0 1-1.6-1.4zM6.5 8h11M10 6h4" fill="none" stroke="#fff" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>'}
];
const clock=()=>{const d=new Date();return d.toLocaleDateString('en',{weekday:'short'})+'  '+d.toLocaleTimeString('en',{hour:'2-digit',minute:'2-digit',hour12:false})};
window.Whirlpool={
 menubar(opts={}){
  return `<div class="mbar"><div class="l"><img class="mark" src="${window.WP_BASE||''}assets/icon.png" alt=""><b>${opts.app||'Browser'}</b><span class="m">File</span><span class="m">Edit</span><span class="m">View</span><span class="m">Window</span><span class="m">Help</span></div>
  <div class="island"><span>5h</span><span class="bar"><u style="width:${opts.a||62}%"></u></span><i>${opts.a||62}%</i><span>7d</span><span class="bar"><u style="width:${opts.b||31}%"></u></span><i>${opts.b||31}%</i></div>
  <div class="r">
   <svg viewBox="0 0 24 24"><path d="M5 12.5a10 10 0 0 1 14 0M8 15.6a6 6 0 0 1 8 0"/><circle cx="12" cy="18.6" r=".6"/></svg>
   <svg viewBox="0 0 24 24"><path d="M4 10v4h3l4 3.5v-11L7 10zM15 9.5a4 4 0 0 1 0 5M17.5 7a7.5 7.5 0 0 1 0 10"/></svg>
   <svg viewBox="0 0 24 24"><rect x="3" y="8" width="16" height="9" rx="2.4"/><path d="M21.5 11v2"/><rect x="5" y="10" width="9" height="5" rx="1" fill="currentColor" stroke="none"/></svg>
   <span class="clock" data-clock>${opts.clock||clock()}</span></div></div>`;
 },
 dock(opts={}){
  const s=opts.size||58;
  return `<div class="dockwrap"><div class="dock" style="--s:${s}px">`+ICONS.map(i=>i==='sep'?'<div class="sep"></div>':
   `<div class="app${i.on?' on':''}" data-n="${i.n}"><span class="tip">${i.n}</span><div class="tile" style="${i.img?'background:none;box-shadow:none':'background:'+i.bg}">${i.img?`<img src="${i.img}" style="width:100%;height:100%" alt="">`:`<svg viewBox="0 0 24 24">${i.svg}</svg>`}</div></div>`).join('')+`</div></div>`;
 },
 /* Cursor magnification: gaussian falloff on distance, eased each frame. */
 magnify(root,{base=58,peak=1.62,reach=95}={}){
  const apps=[...root.querySelectorAll('.app')];const dock=root.querySelector('.dock');
  let mx=null,cur=apps.map(()=>base),raf=0;
  const frame=()=>{let moving=false;
   apps.forEach((a,i)=>{const r=a.getBoundingClientRect(),sc=root.getBoundingClientRect().width/root.offsetWidth||1;
     const c=r.left+r.width/2;const want=mx==null?base:base*(1+(peak-1)*Math.exp(-Math.pow((mx-c)/(reach*sc),2)));
     cur[i]+=(want-cur[i])*.28;if(Math.abs(want-cur[i])>.15)moving=true;else cur[i]=want;a.style.setProperty('--s',cur[i].toFixed(2)+'px');a.style.width=a.style.height=cur[i]+'px'});
   raf=moving?requestAnimationFrame(frame):0};
  const kick=()=>{if(!raf)raf=requestAnimationFrame(frame)};
  dock.addEventListener('pointermove',e=>{mx=e.clientX;kick()});
  dock.addEventListener('pointerleave',()=>{mx=null;kick()});
 },
 popover(){return `<div class="pop" style="left:calc(50% - 460px);bottom:124px"><div class="thumbs">
   ${[['Whirlpool — Release notes',1],['Specifications · Search',0]].map(([t,s])=>`<div class="thumb${s?' sel':''}"><div class="t">${t}</div><div class="w"><i></i><s style="top:26px;width:60%"></s><s style="top:42px"></s><s style="top:58px;width:78%"></s><s style="top:80px;right:40%;height:22px;border-radius:6px;background:rgba(14,147,152,.45)"></s></div></div>`).join('')}</div></div>`}
};
setInterval(()=>document.querySelectorAll('[data-clock]').forEach(e=>{if(!e.dataset.fixed)e.textContent=clock()}),30000);
})();

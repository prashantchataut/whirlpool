(()=>{
const REPO='prashantchataut/whirlpool';
/* --- hero: the live desktop --- */
const desk=document.getElementById('heroDesk'),wrap=document.getElementById('heroWrap');
desk.innerHTML=Whirlpool.menubar({app:'Browser'})+Whirlpool.dock({size:60});
Whirlpool.magnify(desk,{base:60,peak:1.7,reach:100});
const fit=()=>{const k=Math.min(1,wrap.clientWidth/1280);desk.style.transform=`scale(${k})`;wrap.style.height=(760*k+50)+'px'};
addEventListener('resize',fit);fit();

/* --- background vortex turns with the page --- */
const vx=document.getElementById('vortex');
let paths='';for(let arm=0;arm<3;arm++){let d='';for(let i=0;i<=160;i++){const t=i/160,a=arm*2.0944+t*9.5,r=24+t*460;d+=(i?'L':'M')+(r*Math.cos(a)).toFixed(1)+' '+(r*Math.sin(a)).toFixed(1)}
 paths+=`<path d="${d}" stroke-width="${1.2+arm*.4}" opacity="${.5-arm*.12}"/>`}
vx.innerHTML=paths;
let ticking=false;const spin=()=>{ticking=false;vx.style.transform=`rotate(${scrollY*.07}deg)`};
addEventListener('scroll',()=>{if(!ticking){ticking=true;requestAnimationFrame(spin)}},{passive:true});spin();

/* --- tour: chapters drive the screen --- */
const items=[...document.querySelectorAll('#chapters li')],imgs=[...document.querySelectorAll('#screen img')],cap=document.getElementById('cap');
const caps={dock:['Fig. 1 — Dock, bottom, magnification 1.62×','Fig. 1b — Window previews on hover'],menubar:['Fig. 4 — Usage island, now playing, quick settings'],'settings-dark':['Fig. 3 — Settings, Abyss (dark)']};
const io=new IntersectionObserver(es=>{es.forEach(e=>{if(!e.isIntersecting)return;items.forEach(i=>i.classList.toggle('on',i===e.target));
 const k=e.target.dataset.img;imgs.forEach(im=>im.classList.toggle('on',im.dataset.k===k));
 const idx=items.indexOf(e.target);cap.textContent=idx===1?caps.dock[1]:caps[k][0]})},{rootMargin:'-45% 0px -45% 0px'});
items.forEach(i=>io.observe(i));

/* --- releases, straight from GitHub --- */
const fmt=b=>b>1048576?(b/1048576).toFixed(1)+' MB':Math.round(b/1024)+' KB';
const esc=s=>String(s).replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));
fetch(`https://api.github.com/repos/${REPO}/releases?per_page=6`).then(r=>r.ok?r.json():Promise.reject(r.status)).then(rs=>{
 const list=document.getElementById('relList');
 if(!rs.length)throw 0;
 list.innerHTML=rs.map(r=>`<li><div class="rel-top"><h3>${esc(r.name||r.tag_name)}</h3><span class="mono tag">${esc(r.tag_name)}</span><span class="mono date">${new Date(r.published_at).toISOString().slice(0,10)}</span></div>
  <div class="rel-notes">${esc((r.body||'').replace(/^#+\s.*$/gm,'').trim().split('\n').slice(0,5).join('\n'))}</div>
  <div class="assets">${r.assets.map(a=>`<a class="asset" href="${a.browser_download_url}">${esc(a.name)}<span>${fmt(a.size)}</span></a>`).join('')}</div></li>`).join('');
 const exe=rs[0].assets.filter(a=>/\.exe$/.test(a.name));
 if(exe.length)document.getElementById('sizes').innerHTML=exe.map(a=>`<b>${esc(a.name)}</b> ${fmt(a.size)}`).join('<br>')+`<br><span class="dim">Measured from ${esc(rs[0].tag_name)}.</span>`;
}).catch(()=>{document.getElementById('relList').innerHTML=`<li><div class="rel-top"><h3>Latest release</h3></div><div class="assets" style="margin-top:14px"><a class="asset" href="https://github.com/${REPO}/releases/latest">Open releases on GitHub</a></div></li>`});
document.getElementById('copy').addEventListener('click',e=>{navigator.clipboard.writeText(document.getElementById('oneliner').textContent);e.target.textContent='Copied';setTimeout(()=>e.target.textContent='Copy',1600)});
})();

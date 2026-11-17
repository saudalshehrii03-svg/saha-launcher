const Q = new URLSearchParams(location.search);
const $ = (s, r = document) => r.querySelector(s);
const I = {
  home:'<path d="M3 10.5 12 3l9 7.5V20a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1z"/>',
  search:'<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',
  film:'<rect x="3" y="3" width="18" height="18" rx="3"/><path d="M7.5 3v18M16.5 3v18M3 8h4.5M3 12h18M3 16h4.5M16.5 8H21M16.5 16H21"/>',
  tv:'<rect x="2.5" y="6" width="19" height="14" rx="3"/><path d="m8 2 4 4 4-4"/>',
  bookmark:'<path d="M6 4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v17l-6-4-6 4z"/>',
  puzzle:'<path d="M10 3.5a2 2 0 0 1 4 0V5h4a1 1 0 0 1 1 1v4h-1.5a2 2 0 0 0 0 4H19v4a1 1 0 0 1-1 1h-4v-1.5a2 2 0 0 0-4 0V19H6a1 1 0 0 1-1-1v-4h1.5a2 2 0 0 0 0-4H5V6a1 1 0 0 1 1-1h4z"/>',
  gear:'<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/>',
  play:'<path d="M7 4.8v14.4a.8.8 0 0 0 1.2.7l11.5-7.2a.8.8 0 0 0 0-1.4L8.2 4.1A.8.8 0 0 0 7 4.8z" fill="currentColor" stroke="none"/>',
  pause:'<rect x="6" y="4" width="4.2" height="16" rx="1.2" fill="currentColor" stroke="none"/><rect x="13.8" y="4" width="4.2" height="16" rx="1.2" fill="currentColor" stroke="none"/>',
  plus:'<path d="M12 5v14M5 12h14"/>', check:'<path d="m5 12.5 4.5 4.5L19 7"/>', x:'<path d="M6 6l12 12M18 6 6 18"/>',
  info:'<circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 8h.01"/>',
  star:'<path d="m12 3 2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1L3.2 9.5l6.1-.9z" fill="currentColor" stroke="none"/>',
  heart:'<path d="M12 20s-7-4.4-9-9a5 5 0 0 1 9-3 5 5 0 0 1 9 3c-2 4.6-9 9-9 9z"/>',
  share:'<path d="M4 12v7a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-7M12 3v12M7 8l5-5 5 5"/>',
  cc:'<rect x="3" y="5" width="18" height="14" rx="3"/><path d="M10.5 10.2a2.2 2.2 0 1 0 0 3.6M17 10.2a2.2 2.2 0 1 0 0 3.6"/>',
  folder:'<path d="M3 6.5a1.5 1.5 0 0 1 1.5-1.5H9l2 2h8.5A1.5 1.5 0 0 1 21 8.5v9a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 17.5z"/>',
  sparkles:'<path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8zM19 15l.8 2.2L22 18l-2.2.8L19 21l-.8-2.2L16 18l2.2-.8z"/>',
  sync:'<path d="M20 11a8 8 0 0 0-14.3-4.9L4 8M4 4v4h4M4 13a8 8 0 0 0 14.3 4.9L20 16M20 20v-4h-4"/>',
  grid:'<rect x="3.5" y="3.5" width="7" height="7" rx="2"/><rect x="13.5" y="3.5" width="7" height="7" rx="2"/><rect x="3.5" y="13.5" width="7" height="7" rx="2"/><rect x="13.5" y="13.5" width="7" height="7" rx="2"/>',
  globe:'<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"/>',
  link:'<path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"/>',
  back:'<path d="M9 5l7 7-7 7"/>', chevL:'<path d="M15 5l-7 7 7 7"/>', chevD:'<path d="m6 9 6 6 6-6"/>',
  vol:'<path d="M4 9.5v5h3.5L13 19V5L7.5 9.5z"/><path d="M16.5 8.5a5 5 0 0 1 0 7M19 6a8.5 8.5 0 0 1 0 12"/>',
  max:'<path d="M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5"/>',
  rw:'<path d="M4 12a8 8 0 1 0 2.3-5.7L4 8.5M4 4v4.5h4.5"/>', fw:'<path d="M20 12a8 8 0 1 1-2.3-5.7L20 8.5M20 4v4.5h-4.5"/>',
  next:'<path d="M5 5.5v13l10-6.5z" fill="currentColor" stroke="none"/><path d="M19 5v14"/>',
  bell:'<path d="M6 16v-5a6 6 0 0 1 12 0v5l1.5 2h-15zM10 20.5a2 2 0 0 0 4 0"/>',
  user:'<circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/>',
  palette:'<path d="M12 3a9 9 0 1 0 0 18c1.2 0 1.8-.9 1.5-2-.3-1.2.4-2 1.6-2H17a4 4 0 0 0 4-4c0-5.5-4-10-9-10z"/><circle cx="7.5" cy="11" r="1.1"/><circle cx="10" cy="7" r="1.1"/><circle cx="15" cy="7.5" r="1.1"/>',
  shield:'<path d="M12 3 4.5 6v6c0 4.6 3.2 7.8 7.5 9 4.3-1.2 7.5-4.4 7.5-9V6z"/>',
  download:'<path d="M12 4v11M7 10l5 5 5-5M5 20h14"/>', clock:'<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  layers:'<path d="m12 3 9 5-9 5-9-5z"/><path d="m3 13 9 5 9-5"/>', trailer:'<rect x="3" y="5" width="18" height="14" rx="3"/><path d="m10 9.5 5 2.5-5 2.5z" fill="currentColor"/>',
  mic:'<rect x="9" y="3" width="6" height="12" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3"/>', bolt:'<path d="M13 2 4 14h7l-1 8 9-12h-7z"/>',
  monitor:'<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>', moon:'<path d="M20 14.5A8 8 0 0 1 9.5 4 8 8 0 1 0 20 14.5z"/>',
  type:'<path d="M5 6V4h14v2M12 4v16M9 20h6"/>', wave:'<path d="M2 12q2.5-4 5 0t5 0 5 0 5 0"/>',
};
const ic = (n, s, extra='') => `<svg class="i" viewBox="0 0 24 24"${s ? ` style="width:${s}px;height:${s}px"` : ''} ${extra}>${I[n]}</svg>`;
let LG = 0;
const mark = (c='#fff') => { const k='cut'+(LG++); return `<svg viewBox="0 0 100 100"><defs><mask id="${k}" maskUnits="userSpaceOnUse" x="0" y="0" width="100" height="100"><rect width="100" height="100" fill="#fff"/><path d="M18 58Q26 51 34 58Q42 65 50 58Q58 51 66 58Q74 65 82 58Q90 51 98 58" fill="none" stroke="#000" stroke-width="6" stroke-linecap="round"/></mask></defs><g transform="translate(50 50) scale(.78) translate(-52 -50)"><path mask="url(#${k})" d="M32 20L32 76L80 48Z" fill="${c}" stroke="${c}" stroke-width="4" stroke-linejoin="round"/><path d="M22 88Q29 82 36 88Q43 94 50 88Q57 82 64 88Q71 94 78 88" fill="none" stroke="${c}" stroke-width="5" stroke-linecap="round"/></g></svg>`; };

const ADDONS = [
  { n:'Cinemeta', d:'الكتالوج الرسمي: أفلام، مسلسلات وبيانات وصفية', v:'3.0.12', t:'كتالوج', c:['#7b5cff','#4fd1ff'], i:'film', on:true, off:true },
  { n:'OpenSubtitles', d:'ترجمات بأكثر من ٦٠ لغة مع مطابقة تلقائية', v:'1.4.0', t:'ترجمة', c:['#22c55e','#a3e635'], i:'cc', on:true },
  { n:'Local Files', d:'شغّل ملفاتك من الجهاز والشبكة المحلية', v:'2.1.3', t:'مصدر', c:['#f59e0b','#f97316'], i:'folder', on:true },
  { n:'Anime Kitsu', d:'كتالوج الأنمي مع المواسم والحلقات', v:'0.9.4', t:'أنمي', c:['#ff7a59','#ffb86b'], i:'sparkles', users:'٢٤٠ ألف' },
  { n:'Trakt Sync', d:'مزامنة سجل المشاهدة وقوائمك', v:'2.3.1', t:'مزامنة', c:['#ed1c24','#9b1c31'], i:'sync', users:'١٨٠ ألف' },
  { n:'Public Domain', d:'كلاسيكيات مجانية من الملكية العامة', v:'1.2.0', t:'أفلام', c:['#a78bfa','#f0abfc'], i:'globe', users:'٦٢ ألف' },
];
const ai = (a, s=56, r=16) => `<div class="aicon" style="width:${s}px;height:${s}px;border-radius:${r}px;background:linear-gradient(135deg,${a.c[0]},${a.c[1]});display:grid;place-items:center;color:#fff;flex:none;box-shadow:inset 0 1px 0 rgba(255,255,255,.45),0 8px 20px -10px rgba(0,0,0,.6)">${ic(a.i, s*.46)}</div>`;
const IMG = { poster:{}, back:{}, wide:{}, ep:[] };
async function buildImages(D){
  const o = D.art || {}; const pal = it => o.pal ? o.pal(it) : null;
  const jobs = [];
  for (const it of SHOWS){
    jobs.push(art(it, 400, 600, 0, { title:true, font:D.posterFont || D.font, pal:pal(it), filter:o.filter, titleWeight:o.titleWeight, titleColor:o.titleColor }).then(u => IMG.poster[it.id] = u));
    jobs.push(art(it, 1600, 900, 1, { pal:pal(it), filter:o.filter }).then(u => IMG.back[it.id] = u));
  }
  const F = S[D.feat];
  for (let e = 0; e < 6; e++) jobs.push(art(F, 560, 315, 20 + e, { pal:pal(F), filter:o.filter }).then(u => IMG.ep[e] = u));
  jobs.push(art(F, 1600, 1000, 99, { pal:pal(F), filter:o.filter }).then(u => IMG.player = u));
  await Promise.all(jobs);
}
const bg = u => `background-image:url(${u});background-size:cover;background-position:center`;

// Invented catalogue (original titles) + procedural artwork
window.SHOWS = [
  { id:'athar', t:'أثر', en:'TRACE', kind:'مسلسل', year:'٢٠٢٦', r:'8.9', g:['غموض','دراما'], age:'+16', seasons:3, m:'mountains', pal:['#0d1424','#33507a','#f2b48a','#ffe6cf'],
    d:'مصوّرة وثائقية تعود إلى قريتها في الجبال لتجد أن كل صورة التقطتها قبل عشر سنوات تخفي شخصاً لم يكن هناك.' },
  { id:'falak', t:'فلك', en:'ORBIT', kind:'مسلسل', year:'٢٠٢٦', r:'8.7', g:['خيال علمي'], age:'+13', seasons:2, m:'planet', pal:['#0a1030','#3b2d86','#e7a3ff','#8fe1ff'],
    d:'أول مستعمرة عربية على قمر بعيد، وإشارة غامضة تصل كل ٢٧ يوماً بالضبط.' },
  { id:'sahil', t:'الساحل', en:'THE COAST', kind:'فيلم', year:'٢٠٢٥', r:'7.8', g:['إثارة'], age:'+16', m:'lighthouse', pal:['#061520','#0e3c55','#9fe7ff','#ffd27a'],
    d:'ليلة عاصفة واحدة على ساحل مهجور تكشف سرّ عائلة كاملة.' },
  { id:'madina', t:'مدينة الضوء', en:'CITY OF LIGHT', kind:'مسلسل', year:'٢٠٢٤', r:'8.2', g:['جريمة'], age:'+18', seasons:4, m:'city', pal:['#0a0a1c','#2c1b4d','#ff5aa8','#5ef0ff'],
    d:'محققة شابة في مدينة لا تنطفئ أنوارها تلاحق لصاً لا يسرق إلا الذكريات.' },
  { id:'rimal', t:'رمال', en:'SANDS', kind:'مسلسل', year:'٢٠٢٥', r:'8.4', g:['تاريخي','مغامرة'], age:'+13', seasons:2, m:'dunes', pal:['#2a0f1f','#c2562f','#ffcf8a','#ffeacd'],
    d:'قافلة تائهة وخريطة على جلد قديم وسرّ دفنته الصحراء مئة عام.' },
  { id:'bahr', t:'عمق', en:'DEEP', kind:'فيلم', year:'٢٠٢٦', r:'7.9', g:['مغامرة'], age:'+13', m:'sea', pal:['#03202b','#06607a','#7ff0e0','#fff1c9'],
    d:'غواصة صغيرة، وثلاثة باحثين، وصوت يأتي من أعمق نقطة في البحر.' },
  { id:'bawaba', t:'البوابة', en:'THE GATE', kind:'مسلسل', year:'٢٠٢٥', r:'8.3', g:['فانتازيا'], age:'+13', seasons:2, m:'portal', pal:['#14081f','#4a1d5e','#ffb3e6','#b9a7ff'],
    d:'باب حجري في قلب المدينة القديمة يفتح مرة كل جيل.' },
  { id:'ghurub', t:'غروب', en:'DUSK', kind:'فيلم', year:'٢٠٢٤', r:'7.4', g:['رومانسي'], age:'عام', m:'stripes', pal:['#2b0d2e','#e2466d','#ffb05c','#fff0a8'],
    d:'رسائل تصل مع كل غروب، من شخص لم يعد موجوداً.' },
  { id:'ghaba', t:'الغابة', en:'THE WOODS', kind:'مسلسل', year:'٢٠٢٦', r:'8.0', g:['رعب'], age:'+18', seasons:1, m:'forest', pal:['#06130e','#1b4332','#b7f5c9','#e9ffe8'],
    d:'قرية على حافة غابة، وأطفال يختفون كلما اكتمل القمر.' },
  { id:'nabd', t:'نبض', en:'PULSE', kind:'فيلم', year:'٢٠٢٦', r:'7.8', g:['خيال علمي'], age:'+16', m:'portal', pal:['#020b1c','#0b3a73','#4ef2ff','#ff5fd2'],
    d:'جرّاحة تكتشف أن قلب مريضها يدقّ بإيقاع رسالة مشفّرة.' },
  { id:'qimam', t:'القمم', en:'SUMMITS', kind:'فيلم', year:'٢٠٢٥', r:'7.6', g:['وثائقي'], age:'عام', m:'mountains', pal:['#101a2b','#3a5a7a','#f5b38a','#ffe1c4'],
    d:'رحلة إلى أعلى قمم الجزيرة العربية.' },
  { id:'sirr', t:'السر', en:'THE SECRET', kind:'مسلسل', year:'٢٠٢٣', r:'8.8', g:['دراما'], age:'+16', seasons:3, m:'planet', pal:['#050505','#262626','#f2f2f2','#ff6a3d'],
    d:'عائلة واحدة، وسرّ واحد، وثلاثة أجيال.' },
  { id:'bayt', t:'بيت الجدة', en:'GRANDMA\'S HOUSE', kind:'مسلسل', year:'٢٠٢٤', r:'8.1', g:['عائلي','كوميديا'], age:'عام', seasons:2, m:'stripes', pal:['#06283d','#1b8bb4','#ffd166','#fff7e0'],
    d:'سبعة أحفاد يقضون صيفاً كاملاً في بيت واحد.' },
  { id:'shamal', t:'الشمال', en:'NORTH', kind:'مسلسل', year:'٢٠٢٤', r:'7.9', g:['جريمة'], age:'+16', seasons:2, m:'mountains', pal:['#0a1218','#24414f','#d9f2ff','#ffffff'],
    d:'جثة في الثلج، وغريب وصل للتو، وبلدة لا تحب الأسئلة.' },
];
window.S = Object.fromEntries(SHOWS.map(x => [x.id, x]));
window.EPS = [
  ['العودة','تصل «سلمى» إلى القرية بعد غياب عشر سنوات.','٥٢ د',100],
  ['الصورة الأولى','في الأرشيف القديم، وجه لا تعرفه يحدّق في العدسة.','٤٨ د',100],
  ['الظل','الجيران يرفضون الحديث عن ليلة الحريق.','٥٥ د',62],
  ['الممر','طريق جبلي لا يظهر في أي خريطة.','٥٠ د',0],
  ['الشاهد','الراعي العجوز يعرف أكثر مما يقول.','٥٨ د',0],
  ['الأثر','الصورة الأخيرة تكشف كل شيء.','١ س ٤ د',0],
];

function rng(seed){ let s = 0; for (const ch of String(seed)) s = (s*31 + ch.charCodeAt(0)) >>> 0; s ||= 7; return () => { s ^= s << 13; s >>>= 0; s ^= s >> 17; s ^= s << 5; s >>>= 0; return (s % 100000) / 100000; }; }

function artSVG(it, w, h, v, pal){
  const r = rng(it.id + ':' + v); const [c0,c1,c2,c3] = pal || it.pal; const id = 'a' + Math.floor(r()*1e9);
  const defs = `<linearGradient id="${id}s" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${c0}"/><stop offset=".65" stop-color="${c1}"/><stop offset="1" stop-color="${c2}" stop-opacity=".9"/></linearGradient>
    <radialGradient id="${id}g"><stop offset="0" stop-color="${c3}" stop-opacity=".95"/><stop offset=".35" stop-color="${c2}" stop-opacity=".5"/><stop offset="1" stop-color="${c2}" stop-opacity="0"/></radialGradient>
    <radialGradient id="${id}v" cx=".5" cy=".45" r=".75"><stop offset=".55" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity=".55"/></radialGradient>
    <filter id="${id}b"><feGaussianBlur stdDeviation="${w/60}"/></filter>
    <filter id="${id}n"><feTurbulence type="fractalNoise" baseFrequency=".9" numOctaves="2" stitchTiles="stitch"/><feColorMatrix values="0 0 0 0 1 0 0 0 0 1 0 0 0 0 1 0 0 0 .09 0"/></filter>`;
  let g = `<rect width="${w}" height="${h}" fill="url(#${id}s)"/>`;
  const stars = (n, my) => { let s=''; for (let i=0;i<n;i++) s += `<circle cx="${r()*w}" cy="${r()*h*my}" r="${(r()*1.4+.3)*w/800}" fill="#fff" opacity="${r()*.8+.2}"/>`; return s; };
  const sx = w*(.25+r()*.5), sy = h*(.28+r()*.18), sr = Math.min(w,h)*(.12+r()*.08);
  const sun = () => `<circle cx="${sx}" cy="${sy}" r="${sr*3}" fill="url(#${id}g)"/><circle cx="${sx}" cy="${sy}" r="${sr}" fill="${c3}"/>`;
  const layer = (y0, amp, col, op, seg=6, sm=true) => { let d = `M0 ${h} L0 ${y0}`; const st = w/seg;
    for (let i=1;i<=seg;i++){ const x=i*st, y=y0-(r()-.4)*amp; d += sm ? ` Q${x-st/2} ${y0-(r()-.5)*amp*1.6} ${x} ${y}` : ` L${x-st/2} ${y0-r()*amp} L${x} ${y}`; }
    return `<path d="${d} L${w} ${h} Z" fill="${col}" opacity="${op}"/>`; };
  const fig = (x, b, s) => `<g transform="translate(${x} ${b}) scale(${s})" fill="#05060a"><circle cx="0" cy="-172" r="15"/><path d="M-20 -150 Q0 -158 20 -150 L26 -70 L16 -70 L12 0 L3 0 L0 -60 L-3 0 L-12 0 L-16 -70 L-26 -70 Z"/></g>`;
  switch (it.m){
    case 'planet': g += stars(160,1) + `<circle cx="${sx}" cy="${sy+h*.08}" r="${sr*4}" fill="url(#${id}g)" opacity=".7"/><circle cx="${sx}" cy="${sy+h*.08}" r="${sr*1.7}" fill="${c1}"/><circle cx="${sx-sr*.4}" cy="${sy+h*.08-sr*.5}" r="${sr*1.5}" fill="${c2}" opacity=".35" filter="url(#${id}b)"/><ellipse cx="${sx}" cy="${sy+h*.08}" rx="${sr*3}" ry="${sr*.55}" fill="none" stroke="${c3}" stroke-width="${sr*.12}" opacity=".8" transform="rotate(-14 ${sx} ${sy+h*.08})"/>` + layer(h*.86, h*.05, '#05060c', 1, 5) + (r()>.3 ? fig(w*(.3+r()*.4), h*.88, h/900) : ''); break;
    case 'dunes': g += stars(40,.4) + sun() + layer(h*.62,h*.08,c2,.55,4) + layer(h*.72,h*.08,c1,.85,4) + layer(h*.84,h*.06,c0,1,3) + (r()>.4 ? fig(w*(.35+r()*.3), h*.82, h/1100) : ''); break;
    case 'mountains': g += stars(70,.5) + sun() + layer(h*.55,h*.3,c2,.35,7,false) + layer(h*.66,h*.25,c1,.7,6,false) + layer(h*.8,h*.2,c0,1,5,false) + (r()>.5 ? fig(w*(.3+r()*.4), h*.86, h/1300) : ''); break;
    case 'city': { g += stars(80,.45) + `<circle cx="${sx}" cy="${sy*.8}" r="${sr*2.5}" fill="url(#${id}g)" opacity=".8"/><circle cx="${sx}" cy="${sy*.8}" r="${sr*.7}" fill="${c3}"/>`;
      for (let L=0; L<2; L++){ let x=0; const col = L ? '#05050c' : c1; while (x<w){ const bw=w*(.04+r()*.07), bh=h*(.25+r()*.4)*(L?.8:1); g += `<rect x="${x}" y="${h-bh}" width="${bw}" height="${bh}" fill="${col}" opacity="${L?1:.6}"/>`;
        if (L) for (let k=0;k<12;k++) if (r()>.5) g += `<rect x="${x+bw*(.1+r()*.7)}" y="${h-bh*r()*.95}" width="${w/260}" height="${w/200}" fill="${r()>.5?c2:c3}" opacity=".9"/>`; x += bw + w*.004; } } break; }
    case 'sea': case 'lighthouse': { g += stars(90,.5) + sun(); for (let i=0;i<6;i++) g += layer(h*(.62+i*.065), h*.035, i%2?c1:c0, .55+i*.08, 8);
      g += `<rect x="${sx-sr*.5}" y="${h*.62}" width="${sr}" height="${h*.38}" fill="${c3}" opacity=".18" filter="url(#${id}b)"/>`;
      if (it.m==='lighthouse'){ const lx=w*.72, lb=h*.7; g += `<path d="M${lx-w*.11} ${h} L${lx-w*.06} ${lb} L${lx+w*.08} ${lb} L${lx+w*.13} ${h} Z" fill="#03070c"/><path d="M${lx-w*.018} ${lb} L${lx-w*.012} ${lb-h*.32} L${lx+w*.012} ${lb-h*.32} L${lx+w*.018} ${lb} Z" fill="#03070c"/><rect x="${lx-w*.016}" y="${lb-h*.36}" width="${w*.032}" height="${h*.04}" fill="${c3}"/><path d="M${lx} ${lb-h*.34} L${-w*.1} ${lb-h*.55} L${-w*.1} ${lb-h*.2} Z" fill="${c3}" opacity=".18"/>`; } break; }
    case 'portal': { g += stars(60,1); const cx=w/2, cy=h*.52, R=Math.min(w,h)*.32; g += `<circle cx="${cx}" cy="${cy}" r="${R*1.8}" fill="url(#${id}g)" opacity=".8"/>`;
      for (let i=6;i>0;i--) g += `<circle cx="${cx}" cy="${cy}" r="${R*i/5}" fill="none" stroke="${i%2?c3:c2}" stroke-width="${w/300}" opacity="${.15+i*.08}"/>`;
      g += `<path d="M${cx-R*.55} ${h} L${cx-R*.55} ${cy} A${R*.55} ${R*.55} 0 0 1 ${cx+R*.55} ${cy} L${cx+R*.55} ${h} Z" fill="${c3}" opacity=".9"/>` + fig(cx, h*.97, h/700) + layer(h*.93, h*.02, '#05050a', 1, 4); break; }
    case 'stripes': { g += sun(); for (let i=0;i<7;i++) g += `<rect x="0" y="${sy+sr*.15+i*sr*.18}" width="${w}" height="${sr*.05+i*sr*.012}" fill="${c1}"/>`;
      g += layer(h*.7,h*.05,c0,.9,3) + layer(h*.82,h*.04,'#05050a',1,4);
      for (let i=0;i<5;i++){ const px=w*(.1+r()*.8); g += `<path d="M${px} ${h*.82} q${w*.01} -${h*.12} ${w*.03} -${h*.18}" stroke="#05050a" stroke-width="${w/160}" fill="none"/><circle cx="${px+w*.03}" cy="${h*.64}" r="${w*.03}" fill="#05050a"/>`; } break; }
    case 'forest': { g += `<circle cx="${sx}" cy="${sy*.7}" r="${sr*.8}" fill="${c3}"/><circle cx="${sx}" cy="${sy*.7}" r="${sr*3}" fill="url(#${id}g)" opacity=".6"/>` + stars(60,.4);
      for (let L=0;L<3;L++){ const base=h*(.68+L*.12), col=[c2,c1,'#020604'][L], op=[.3,.7,1][L]; for (let i=0;i<14;i++){ const tx=(i/13)*w+(r()-.5)*w*.06, th=h*(.25+r()*.2)*(1-L*.15), tw=th*.35; g += `<path d="M${tx} ${base-th} L${tx+tw/2} ${base} L${tx-tw/2} ${base} Z" fill="${col}" opacity="${op}"/>`; } g += `<rect y="${base-2}" width="${w}" height="${h}" fill="${col}" opacity="${op}"/>`; } break; }
  }
  g += `<rect width="${w}" height="${h}" fill="url(#${id}v)"/><rect width="${w}" height="${h}" filter="url(#${id}n)"/>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}"><defs>${defs}</defs>${g}</svg>`;
}
async function art(it, w, h, v, opt = {}){
  const img = new Image(); img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(artSVG(it, w, h, v, opt.pal));
  await img.decode();
  const cv = document.createElement('canvas'); cv.width = w; cv.height = h; const cx = cv.getContext('2d');
  if (opt.filter) cx.filter = opt.filter;
  cx.drawImage(img, 0, 0); cx.filter = 'none';
  if (opt.title){
    const gr = cx.createLinearGradient(0, h*.5, 0, h); gr.addColorStop(0,'rgba(0,0,0,0)'); gr.addColorStop(1,'rgba(0,0,0,.78)'); cx.fillStyle = gr; cx.fillRect(0, h*.5, w, h*.5);
    cx.direction = 'rtl'; cx.textAlign = 'center'; cx.fillStyle = opt.titleColor || '#fff'; cx.shadowColor = 'rgba(0,0,0,.5)'; cx.shadowBlur = 16;
    let fs = w*.15; const f = s => `${opt.titleWeight || 800} ${s}px ${opt.font}`; cx.font = f(fs);
    while (cx.measureText(it.t).width > w*.84) { fs -= 2; cx.font = f(fs); }
    cx.fillText(it.t, w/2, h*.86); cx.shadowBlur = 0; cx.direction = 'ltr';
    cx.font = `600 ${w*.04}px 'Space Grotesk'`; cx.fillStyle = 'rgba(255,255,255,.72)'; cx.fillText(it.en.split('').join(' '), w/2, h*.935);
  }
  return cv.toDataURL('image/jpeg', .9);
}

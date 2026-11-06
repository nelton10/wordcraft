const constructions = [
  { n:'01', title:'Perpendicular por um ponto', family:'Simetria e equidistância', desc:'Construa uma reta perpendicular passando por um ponto exterior.', tags:['90°','Mediatriz'], color:'mint', slide:3 },
  { n:'02', title:'Paralela por um ponto exterior', family:'Perpendicularidade', desc:'Reduza o paralelismo a duas construções de ângulo reto.', tags:['Paralelas','Ângulos'], color:'coral', slide:4 },
  { n:'03', title:'Dividir um segmento em n partes', family:'Proporcionalidade', desc:'Use uma semirreta auxiliar para repartir um segmento em partes iguais.', tags:['Tales','Razão'], color:'gold', slide:5 },
  { n:'04', title:'Bissetriz de um ângulo', family:'Congruência', desc:'Encontre a semirreta que divide um ângulo em duas partes iguais.', tags:['Ângulos iguais','Arcos'], color:'blue', slide:6 },
  { n:'05', title:'Transportar e operar ângulos', family:'Transporte de medidas', desc:'Copie uma abertura e construa a soma ou a diferença de ângulos.', tags:['Cópia','Soma'], color:'pink', slide:7 },
  { n:'06', title:'Somar e subtrair segmentos', family:'Transporte de comprimentos', desc:'Leve comprimentos sobre uma mesma reta sem usar escala graduada.', tags:['Comprimento','Adição'], color:'mint', slide:8 },
  { n:'07', title:'Quadrado inscrito em um círculo', family:'Simetria e polígonos', desc:'Construa diâmetros perpendiculares e una os quatro pontos da circunferência.', tags:['Diâmetros','Quadrado'], color:'coral', slide:9 },
  { n:'08', title:'Polígonos regulares com 2ⁿ lados', family:'Iteração', desc:'Dobre o número de lados sucessivamente pela bissecção dos ângulos centrais.', tags:['Recorrência','Polígonos'], color:'gold', slide:10 },
];
const motifs = [
 '<path d="M18 74h128"/><path d="M83 24v100" class="accent-line"/><path d="M42 74a41 41 0 0 1 82 0M42 74a41 41 0 0 0 82 0" class="arc-line"/><circle cx="83" cy="74" r="4" class="dot"/>',
 '<path d="M18 82h128"/><path d="M82 28v100" class="arc-line"/><path d="M18 54h128" class="accent-line"/><circle cx="82" cy="54" r="4" class="dot"/>',
 '<path d="M23 107L137 38"/><path d="M37 97l9 15M60 83l9 15M83 69l9 15M106 55l9 15" class="arc-line"/><path d="M43 55l64 45" class="accent-line"/>',
 '<path d="M35 110L73 40M35 110l93-34"/><path d="M53 77a40 40 0 0 1 44 18" class="arc-line"/><path d="M35 110l63-18" class="accent-line"/><circle cx="98" cy="92" r="4" class="dot"/>',
 '<path d="M24 105h118M24 105L110 45"/><path d="M24 105a68 68 0 0 1 76-62" class="arc-line"/><path d="M24 105l53-37M24 105l83-7" class="accent-line"/>',
 '<path d="M24 94h118"/><path d="M45 94h42M87 94h42" class="accent-line"/><circle cx="87" cy="94" r="37" class="arc-line"/><circle cx="45" cy="94" r="3" class="dot"/><circle cx="129" cy="94" r="3" class="dot"/>',
 '<circle cx="83" cy="78" r="48" class="arc-line"/><path d="M83 30v96M35 78h96M49 44l68 68M117 44L49 112" class="accent-line"/><circle cx="83" cy="78" r="3" class="dot"/>',
 '<circle cx="83" cy="78" r="47" class="arc-line"/><path d="M36 78h94M83 31v94M50 45l66 66M116 45L50 111" class="accent-line"/><circle cx="83" cy="78" r="4" class="dot"/>'
];
const grid=document.querySelector('#constructionGrid');
grid.innerHTML=constructions.map((c,i)=>`<a class="construction-card ${c.color} reveal" href="./aula.html?slide=${c.slide}"><div class="card-top"><span class="card-number">${c.n}</span><span class="card-family">${c.family}</span><span class="card-arrow" aria-hidden="true">↗</span></div><div class="card-diagram"><svg viewBox="0 0 166 148" aria-hidden="true">${motifs[i]}</svg></div><h3>${c.title}</h3><p>${c.desc}</p><div class="card-tags">${c.tags.map(t=>`<span>${t}</span>`).join('')}</div><span class="card-link">VER A CONSTRUÇÃO <b>→</b></span></a>`).join('');
const observer=new IntersectionObserver(entries=>entries.forEach(entry=>{if(entry.isIntersecting){entry.target.classList.add('in-view');observer.unobserve(entry.target)}}),{threshold:.12});
document.querySelectorAll('.reveal').forEach(el=>observer.observe(el));

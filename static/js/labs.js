/* Technical guide: engine charts, browser labs, query dialects and local runs. Needs js/estate-graph.js first. */
"use strict";
const G = window.ESTATE_GRAPH;

/* ---------- Sticky in-page nav sits under the site header ---------- */
(function(){
  const hdr=document.querySelector(".site-header");if(!hdr)return;
  const set=()=>document.documentElement.style.setProperty("--toc-top",hdr.offsetHeight+"px");
  set();if(window.ResizeObserver)new ResizeObserver(set).observe(hdr);
})();

/* ---------- Engine data ---------- */
const GV={"A":4,"A-":3.7,"B+":3.3,"B":3,"B-":2.7,"C+":2.3,"C":2,"C-":1.7,"D":1,"F":0};
const E=[
 {n:"Neo4j",logo:"../assets/engine-logos/neo4j.svg",initials:"N4",cat:"native",shared:"A-",embed:null,d:5,o:5,c:2,v:5,cost:4670,note:"Business Critical, 32 GB"},
 {n:"FalkorDB",logo:"../assets/engine-logos/falkordb.png",initials:"FK",cat:"native",shared:"B+",embed:"B",d:4,o:3,c:4,v:3,cost:350,note:"Pro, 8 GB with HA"},
 {n:"Memgraph",logo:"../assets/engine-logos/memgraph.svg",initials:"MG",cat:"native",shared:"B",embed:null,d:4,o:3,c:3,v:3,cost:null,note:"Community free for internal use (BSL); Enterprise on request; Cloud priced by calculator"},
 {n:"Neptune",displayName:"Amazon Neptune",logo:null,initials:"AN",cat:"native",shared:"B",embed:null,d:3,o:5,c:3,v:5,cost:460,note:"Two db.r6g.large, before storage and I/O"},
 {n:"Spanner Graph",displayName:"Google Cloud Spanner Graph",logo:"../assets/engine-logos/google-cloud-spanner.svg",initials:"SG",cat:"over",shared:"B",embed:null,d:3,o:5,c:3,v:5,cost:90,hi:898,note:"Enterprise edition, us-central1, 100 processing units to 1 node; compute only"},
 {n:"PuppyGraph",logo:"../assets/engine-logos/puppygraph.svg",initials:"PG",cat:"over",shared:"B",prov:true,embed:null,d:4,o:3,c:4,v:3,cost:null,note:"Priced on server CPU and memory; quote only"},
 {n:"Fabric graph",displayName:"Microsoft Fabric Graph",logo:"../assets/engine-logos/microsoft-fabric.svg",initials:"FG",cat:"over",shared:"C+",embed:null,d:3,o:3,c:3,v:4,cost:1300,note:"Always on, pay-as-you-go estimate"},
 {n:"MongoDB Atlas",logo:"../assets/engine-logos/mongodb.svg",initials:"MA",cat:"doc",shared:"C",embed:null,d:2,o:5,c:4,v:5,cost:394,note:"M30 cluster"},
 {n:"Cosmos DB Gremlin",displayName:"Azure Cosmos DB for Apache Gremlin",logo:"../assets/engine-logos/azure-cosmos-db.svg",initials:"CD",cat:"doc",shared:"D",embed:null,d:2,o:4,c:2,v:2,cost:null,note:"Throughput-based; depends on load"},
 {n:"Kuzu forks",logo:null,initials:"KF",cat:"embed",shared:null,embed:"B-",d:4,o:1,c:5,v:2,cost:0,note:"Open source; your own compute"},
 {n:"Kuzu 0.11.3",logo:null,initials:"KZ",cat:"embed",shared:null,embed:"C",d:4,o:1,c:5,v:0,cost:0,note:"Archived; pin only for existing use"}
];
const CV=c=>`var(--${c})`;
/* Engine label: a verified local mark when available, otherwise a blank alignment slot, plus the visible product name. Only trusted static data from E reaches this markup. */
document.addEventListener("error",ev=>{if(ev.target.matches?.(".emark img"))ev.target.remove()},true);
function engineLabel(e,pill){
  const name=e.displayName||e.n;
  const mark=`<span class="emark" data-initials="${e.initials}" aria-hidden="true">${e.logo?`<img src="${e.logo}" alt="" width="60" height="22" decoding="async">`:""}</span>`;
  const label=pill?`<span class="pill" style="--c:${CV(e.cat)}">${name}</span>`:`<span class="ename">${name}</span>`;
  return `<span class="elabel${e.logo?'':' elabel--text'}">${mark}<span class="etext">${label}</span></span>`;
}

/* ---------- Grade chart ---------- */
let job="shared";
function drawGrades(){
  const rows=E.filter(e=>e[job]);
  let h='<div class="bars">';
  for(const e of rows){const w=GV[e[job]]/4*72;
    h+=`<div class="blab">${engineLabel(e)}</div><div class="track"><div class="bar" style="--c:${CV(e.cat)};width:${Math.max(w,1)}%"></div><span class="val">${e[job]}${e.prov?" (prov.)":""}</span></div>`}
  document.getElementById("gradeChart").innerHTML=h+'</div><p class="note" style="margin:10px 0 0">Scale runs from F to A.</p>';
}
document.getElementById("jobShared").onclick=e=>{job="shared";e.target.setAttribute("aria-pressed",true);document.getElementById("jobEmbed").setAttribute("aria-pressed",false);drawGrades()};
document.getElementById("jobEmbed").onclick=e=>{job="embed";e.target.setAttribute("aria-pressed",true);document.getElementById("jobShared").setAttribute("aria-pressed",false);drawGrades()};
drawGrades();

/* ---------- Heat table ---------- */
(function(){
  const shade=v=>`color-mix(in srgb, var(--native) ${20+v*16}%, var(--paper))`;
  let h=`<thead><tr><th>Engine</th><th>Depth</th><th>Ops</th><th>Cost</th><th>Viability</th><th>Grade</th></tr></thead><tbody>`;
  for(const e of E){h+=`<tr><td>${engineLabel(e,true)}</td>`;
    for(const k of ["d","o","c","v"]){const v=e[k];h+=`<td class="num" style="background:${shade(v)};color:${v>=3?"#fff":"var(--ink)"}">${v}</td>`}
    h+=`<td>${e.shared?e.shared+(e.prov?" (prov.)":"")+" shared":""}${e.shared&&e.embed?", ":""}${e.embed?e.embed+" embedded":""}</td></tr>`}
  document.getElementById("heat").innerHTML=h+"</tbody>";
})();

/* ---------- Cost chart ---------- */
(function(){
  const rows=[...E].sort((a,b)=>((b.hi||b.cost)??-1)-((a.hi||a.cost)??-1)),max=5000,pct=v=>v/max*70;
  const fmt=v=>"$"+v.toLocaleString("en-GB");
  let h='<div class="bars">';
  for(const e of rows){h+=`<div class="blab">${engineLabel(e)}</div><div class="track">`;
    if(e.cost===null)h+=`<span class="val soft">${e.note}</span>`;
    else if(e.cost===0)h+=`<span class="val soft">$0 licence, own compute</span>`;
    else{h+=`<div class="bar" style="--c:${CV(e.cat)};width:${pct(e.cost)}%"></div>`;
      if(e.hi)h+=`<div class="rng" style="--c:${CV(e.cat)};width:${pct(e.hi-e.cost)}%"></div>`;
      h+=`<span class="val">${e.hi?fmt(e.cost)+" to "+fmt(e.hi):fmt(e.cost)}</span>`}
    h+='</div>'}
  document.getElementById("costChart").innerHTML=h+'</div><p class="note" style="margin:10px 0 0">USD per month. Bars scaled to $5,000.</p>';
})();

/* ---------- Labs ---------- */
// Cosine similarity of each node's text to the preset questions (all-MiniLM-L6-v2), generated by scripts/dense.py.
const DENSE={"Which dashboards use KNA1.REGION?":{"kna1":0.52,"region":0.685,"bqcust":0.464,"bqreg":0.602,"dbx":0.168,"policy":0.098,"lkreg":0.549,"lkchurn":0.476,"finance":0.447,"crm":0.093,"sfacct":0.088,"regret":0.125,"xlsx3":0.192,"xlsxf":0.185,"price":0.19},"What feeds the regulatory return?":{"kna1":0.138,"region":0.205,"bqcust":0.102,"bqreg":0.128,"dbx":-0.005,"policy":0.149,"lkreg":0.255,"lkchurn":0.121,"finance":0.308,"crm":0.154,"sfacct":0.138,"regret":0.611,"xlsx3":0.109,"xlsxf":0.108,"price":0.141},"Which workbook is the source of the price list?":{"kna1":0.245,"region":0.239,"bqcust":0.204,"bqreg":0.194,"dbx":0.124,"policy":0.094,"lkreg":0.284,"lkchurn":0.102,"finance":0.234,"crm":0.084,"sfacct":0.118,"regret":0.163,"xlsx3":0.708,"xlsxf":0.684,"price":0.618},"What does customer_sync write?":{"kna1":0.205,"region":0.24,"bqcust":0.201,"bqreg":0.258,"dbx":0.583,"policy":0.409,"lkreg":0.125,"lkchurn":0.272,"finance":0.257,"crm":0.46,"sfacct":0.38,"regret":0.218,"xlsx3":0.117,"xlsxf":0.098,"price":0.222}};
function labA(G, question, hops){
  // Hybrid text retrieval: BM25 over each node's text, fused with dense similarity by
  // reciprocal rank fusion (1 / (60 + rank) from each ranking). DENSE holds similarities from
  // all-MiniLM-L6-v2 for the preset questions; for any other question only BM25 is used.
  const stop=new Set(["what","is","the","in","of","and","a","s","which","by","does","were","was","to","use"]);
  const tok=s=>(s.toLowerCase().replace(/'s\b/g,"").match(/[a-z0-9]+/g)||[]).filter(w=>!stop.has(w));
  const docs=G.nodes.map(n=>tok(n.text)),q=tok(question),N=docs.length;
  const avg=docs.reduce((s,d)=>s+d.length,0)/N,df=w=>docs.filter(d=>d.includes(w)).length;
  const bm25=docs.map(d=>q.reduce((s,w)=>{const tf=d.filter(x=>x===w).length;if(!tf)return s;
    const idf=Math.log(1+(N-df(w)+0.5)/(df(w)+0.5));return s+idf*tf*2.5/(tf+1.5*(0.25+0.75*d.length/avg))},0));
  const dense=DENSE[question]||null;
  const ranks=sc=>{const r={};sc.map((s,i)=>[s,i]).filter(x=>x[0]>0).sort((a,b)=>b[0]-a[0]).forEach(([,i],k)=>r[i]=k+1);return r};
  const rb=ranks(bm25),rd=dense?ranks(G.nodes.map(n=>dense[n.id]||0)):{};
  const text=G.nodes.map((n,i)=>({n,s:(rb[i]?1/(60+rb[i]):0)+(rd[i]?1/(60+rd[i]):0)})).filter(x=>x.s>0)
    .sort((a,b)=>b.s-a.s).slice(0,5).map(x=>`${x.n.label}: ${x.n.text}`);

  // Graph retrieval: start at entities named in the question (full label, or an identifier
  // such as customer_sync inside it), then follow edges
  const ql=question.toLowerCase();
  const named=n=>[n.label,...n.label.split(" ").filter(w=>/[._]/.test(w))].some(s=>ql.includes(s.toLowerCase()));
  const seeds=G.nodes.filter(named).map(n=>n.id);
  const paths=[];const seen=new Set(seeds);
  let frontier=seeds.map(id=>({id,prev:null,path:G.node(id).label}));
  for(let h=0;h<hops;h++){const next=[];
    for(const f of frontier)for(const nb of G.neighbours(f.id)){
      if(nb.id===f.prev)continue;
      const step=nb.dir==="out"?` -${nb.rel}-> `:` <-${nb.rel}- `;
      const p=f.path+step+G.node(nb.id).label;paths.push(p);
      if(!seen.has(nb.id)){seen.add(nb.id);next.push({id:nb.id,prev:f.id,path:p})}}
    frontier=next}
  return {seeds:seeds.map(id=>G.node(id).label),text,hybrid:!!dense,paths:[...new Set(paths)]};
}
function labB(G, seed, budget){
  // Personalised PageRank from the seed node, then pack text into the budget
  const ids=G.nodes.map(n=>n.id),damp=0.85;let r=Object.fromEntries(ids.map(i=>[i,i===seed?1:0]));
  for(let it=0;it<40;it++){const nr=Object.fromEntries(ids.map(i=>[i,i===seed?1-damp:0]));
    for(const i of ids){const nb=G.neighbours(i);for(const n of nb)nr[n.id]+=damp*r[i]/nb.length}r=nr}
  const ranked=ids.map(i=>({node:G.node(i),score:r[i],tokens:G.tokens(G.node(i))})).sort((a,b)=>b.score-a.score);
  let used=0;for(const x of ranked){x.included=used+x.tokens<=budget;if(x.included)used+=x.tokens}
  return {ranked,used,budget};
}
function labC(thresholds, overconfidence, cases=3000, seed=42){
  // Seeded random numbers so runs are repeatable
  let s=seed;const rnd=()=>{s|=0;s=s+0x6D2B79F5|0;let t=Math.imul(s^s>>>15,1|s);t=t+Math.imul(t^t>>>7,61|t)^t;return((t^t>>>14)>>>0)/4294967296};
  const actions=[{name:"Approve schema change",errorCost:50},{name:"Route migration ticket",errorCost:2},{name:"Flag for review",errorCost:5}];
  const escalationCost=1;
  const stats=actions.map(a=>({...a,n:0,auto:0,wrong:0,cost:0}));let autoAllCost=0;
  for(let i=0;i<cases;i++){
    const k=Math.floor(rnd()*3),st=stats[k];st.n++;
    const stated=0.5+0.5*rnd();                         // what the model says
    const real=stated-overconfidence*(stated-0.5);        // what is actually true
    const correct=rnd()<real;
    if(!correct)autoAllCost+=st.errorCost;
    if(stated>=thresholds[k]){st.auto++;if(!correct){st.wrong++;st.cost+=st.errorCost}}
    else st.cost+=escalationCost;}
  const total=stats.reduce((t,x)=>t+x.cost,0);
  return {stats,total,escalateAll:cases*escalationCost,autoAll:autoAllCost};
}
function labD(servers, toolsPerServer, tokensPerTool, serversNeeded, contextWindow){
  // Tokens spent on tool definitions before the agent has done any work:
  // every tool in the shed, against only the servers a router picks for the task
  const perServer=toolsPerServer*tokensPerTool;
  const whole=servers*perServer;
  const routed=Math.min(serversNeeded,servers)*perServer;
  return {whole,routed,wholeShare:whole/contextWindow,routedShare:routed/contextWindow,contextWindow};
}

const DEFAULT={A:labA.toString(),B:labB.toString(),C:labC.toString(),D:labD.toString()};
const FN={A:labA,B:labB,C:labC,D:labD};
Object.keys(DEFAULT).forEach(k=>document.getElementById(k.toLowerCase()+"Code").value=DEFAULT[k]);

const $=id=>document.getElementById(id);
const esc=s=>String(s).replace(/[&<>]/g,c=>({"&":"&amp;","<":"&lt;",">":"&gt;"}[c]));
const num=v=>Math.round(v).toLocaleString("en-GB"),pc=v=>(v*100).toFixed(1)+"%";
function renderA(r){
  $("aOut").innerHTML=
   `<strong>${r.hybrid?"Hybrid text retrieval (BM25 and dense)":"BM25 text retrieval"}</strong> returned its top ${r.text.length} fragment${r.text.length===1?"":"s"}:`+r.text.map(x=>`<span class="path">${esc(x)}</span>`).join("")+
   `<p class="note" style="margin-top:8px">These are the fragments that look most like the question. None states a relationship that spans two facts, such as which dashboard reads the BigQuery column mapped from KNA1.REGION; that lives in the edges, not in any one text.</p>`+
   `<strong>Graph retrieval</strong> from ${esc(r.seeds.join(", ")||"no named entities")} found ${r.paths.length} paths:`+r.paths.map(x=>`<span class="path">${esc(x)}</span>`).join("");
}
function renderB(r){
  let h=`<p><strong>${r.used}</strong> of ${r.budget} tokens used.</p><div class="scroll"><table><tr><th>Rank</th><th>Node</th><th>Score</th><th>Tokens</th><th>In context</th></tr>`;
  r.ranked.forEach((x,i)=>h+=`<tr><td>${i+1}</td><td>${esc(x.node.label)}</td><td>${x.score.toFixed(3)}</td><td>${x.tokens}</td><td>${x.included?"Yes":"No"}</td></tr>`);
  $("bOut").innerHTML=h+"</table></div>";
}
function renderC(r){
  let h=`<div class="scroll"><table><tr><th>Action</th><th>Cost of error</th><th>Decided automatically</th><th>Error rate when automatic</th><th>Cost</th></tr>`;
  for(const s of r.stats)h+=`<tr><td>${esc(s.name)}</td><td>${s.errorCost}</td><td>${(s.auto/s.n*100).toFixed(0)}%</td><td>${s.auto?(s.wrong/s.auto*100).toFixed(1):"0.0"}%</td><td>${s.cost.toLocaleString("en-GB")}</td></tr>`;
  h+=`</table></div><p style="margin-top:8px">Total cost with thresholds: <strong>${r.total.toLocaleString("en-GB")}</strong>. Review everything: ${r.escalateAll.toLocaleString("en-GB")}. Decide everything automatically: ${r.autoAll.toLocaleString("en-GB")}.</p>`;
  $("cOut").innerHTML=h;
}
function renderD(r){
  const bar=(label,tok,share,c)=>`<div class="blab">${label}</div><div class="track"><div class="bar" style="--c:${c};width:${Math.max(Math.min(share,1)*70,0.5)}%"></div><span class="val">${num(tok)} tokens, ${pc(share)}${share>1?" (exceeds the window)":""}</span></div>`;
  $("dOut").innerHTML=`<div class="bars">${bar("Whole shed loaded",r.whole,r.wholeShare,"var(--reject)")}${bar("Routed subset",r.routed,r.routedShare,"var(--connect)")}</div>`+
   `<p style="margin-top:10px">Routing saves <strong>${num(r.whole-r.routed)}</strong> tokens of tool definitions on every request, out of a ${num(r.contextWindow)}-token context window. All inputs are assumptions; change them to match your estate.</p>`;
}
function params(k){
  if(k==="A")return [G,$("aQ").value,Math.min(4,Math.max(1,+$("aK").value||2))];
  if(k==="B")return [G,$("bSeed").value,+$("bBudget").value];
  if(k==="C")return [[+$("cT0").value,+$("cT1").value,+$("cT2").value],+$("cOver").value];
  return ["dServers","dTools","dTokens","dNeed","dWindow"].map(id=>Math.max(0,+$(id).value||0));
}
const RENDER={A:renderA,B:renderB,C:renderC,D:renderD};
function run(k,fn){const out=$(k.toLowerCase()+"Out");
  try{RENDER[k](fn(...params(k)))}catch(e){out.innerHTML=`<span class="err">Your code threw an error: ${esc(e.message)}. Fix it and run again, or reset the code.</span>`}}
document.querySelectorAll("[data-run]").forEach(b=>b.onclick=()=>run(b.dataset.run,FN[b.dataset.run]));
document.querySelectorAll("[data-reset]").forEach(b=>b.onclick=()=>{const k=b.dataset.reset;$(k.toLowerCase()+"Code").value=DEFAULT[k];run(k,FN[k])});
document.querySelectorAll("[data-runedit]").forEach(b=>b.onclick=()=>{const k=b.dataset.runedit,out=$(k.toLowerCase()+"Out");
  let fn;try{fn=new Function("return ("+$(k.toLowerCase()+"Code").value+")")()}
  catch(e){out.innerHTML=e instanceof EvalError?`<span class="err">This page's security settings block running edited code. Copy the code into your browser console or a Node.js file to experiment.</span>`:`<span class="err">Your code has a syntax error: ${esc(e.message)}</span>`;return}
  run(k,fn)});

const bSeed=$("bSeed");G.nodes.forEach(n=>bSeed.add(new Option(n.label,n.id,n.id==="bqcust",n.id==="bqcust")));
const bBudget=$("bBudget");bBudget.oninput=()=>{$("bBudgetV").textContent=bBudget.value;run("B",FN.B)};
bSeed.onchange=()=>run("B",FN.B);
const cOver=$("cOver");cOver.oninput=()=>{$("cOverV").textContent=(+cOver.value).toFixed(2);run("C",FN.C)};
["cT0","cT1","cT2"].forEach(id=>$(id).onchange=()=>run("C",FN.C));
["dServers","dTools","dTokens","dNeed","dWindow"].forEach(id=>$(id).oninput=()=>run("D",FN.D));
run("A",FN.A);run("B",FN.B);run("C",FN.C);run("D",FN.D);

/* ---------- Snippets: one lineage question in five dialects ---------- */
const SNIPS=[
 {t:"Cypher",who:"Neo4j, FalkorDB, Memgraph, Kuzu, Neptune (openCypher), PuppyGraph",c:`MATCH (:Column {name: 'KNA1.REGION'})-[:MAPS_TO]->(:Column)-[:USED_BY]->(d:Dashboard)
RETURN d.name;`},
 {t:"Gremlin",who:"Neptune, Cosmos DB, PuppyGraph",c:`g.V().has('Column', 'name', 'KNA1.REGION')
  .out('MAPS_TO').hasLabel('Column')
  .out('USED_BY').hasLabel('Dashboard')
  .values('name')`},
 {t:"GQL",who:"Spanner Graph needs the GRAPH line. Fabric graph runs against the graph you have open.",c:`GRAPH LineageGraph
MATCH (:Column {name: 'KNA1.REGION'})-[:MAPS_TO]->(:Column)-[:USED_BY]->(d:Dashboard)
RETURN d.name AS dashboard`},
 {t:"MongoDB",who:"MongoDB Atlas. Each document lists the ids it feeds in a feeds array, so the relationship type is lost and the result is filtered by node type instead. Keep maxDepth low: a $graphLookup stage that passes 100 MB of memory spills to disk, or fails if allowDiskUse is false.",c:`db.entities.aggregate([
  { $match: { type: "Column", name: "KNA1.REGION" } },
  { $graphLookup: {
      from: "entities",
      startWith: "$feeds",
      connectFromField: "feeds",
      connectToField: "_id",
      maxDepth: 1,
      depthField: "hops",
      as: "downstream"
  } },
  { $project: { _id: 0, dashboards: {
      $filter: { input: "$downstream", cond: { $eq: ["$$this.type", "Dashboard"] } }
  } } }
])`},
 {t:"Kuzu (Python)",who:"Embedded. Runs in memory with no server. Works the same on the community forks. Column is a reserved word in Kuzu, so the table is called DataColumn.",c:`import kuzu

db = kuzu.Database(":memory:")
conn = kuzu.Connection(db)

conn.execute("CREATE NODE TABLE DataColumn(name STRING PRIMARY KEY)")
conn.execute("CREATE NODE TABLE Dashboard(name STRING PRIMARY KEY)")
conn.execute("CREATE REL TABLE MAPS_TO(FROM DataColumn TO DataColumn)")
conn.execute("CREATE REL TABLE USED_BY(FROM DataColumn TO Dashboard)")
conn.execute("CREATE (:DataColumn {name: 'KNA1.REGION'}), (:DataColumn {name: 'customers.region'})")
conn.execute("CREATE (:Dashboard {name: 'Regional sales'}), (:Dashboard {name: 'Monthly churn'})")
conn.execute("""
  MATCH (a:DataColumn {name: 'KNA1.REGION'}), (b:DataColumn {name: 'customers.region'})
  CREATE (a)-[:MAPS_TO]->(b)
""")
conn.execute("""
  MATCH (b:DataColumn {name: 'customers.region'}), (d:Dashboard)
  CREATE (b)-[:USED_BY]->(d)
""")

r = conn.execute("""
  MATCH (:DataColumn {name: 'KNA1.REGION'})-[:MAPS_TO]->(:DataColumn)-[:USED_BY]->(d:Dashboard)
  RETURN d.name
""")
while r.has_next():
    print(r.get_next())`}
];
const tabs=$("snipTabs");let cur=0;
function showSnip(i){cur=i;$("snipCode").textContent=SNIPS[i].c;$("snipNote").textContent=SNIPS[i].who;
  [...tabs.children].forEach((b,j)=>{b.setAttribute("aria-pressed",j===i);b.setAttribute("aria-selected",j===i)})}
SNIPS.forEach((s,i)=>{const b=document.createElement("button");b.type="button";b.textContent=s.t;b.setAttribute("role","tab");b.onclick=()=>showSnip(i);tabs.appendChild(b)});
showSnip(0);
async function copyText(text,btn){try{await navigator.clipboard.writeText(text);btn.textContent="Copied"}catch(e){btn.textContent="Select and copy"}setTimeout(()=>btn.textContent="Copy",1600)}
$("snipCopy").onclick=e=>copyText(SNIPS[cur].c,e.target);

/* ---------- Local runs ---------- */
const RUNS=[
 {n:"Neo4j",cat:"native",c:"docker run -p 7474:7474 -p 7687:7687 -e NEO4J_AUTH=neo4j/change-me neo4j:5",note:"Browser UI on localhost:7474."},
 {n:"FalkorDB",cat:"native",c:"docker run -p 6379:6379 -p 3000:3000 -it --rm falkordb/falkordb:latest",note:"Browser UI on localhost:3000."},
 {n:"Memgraph",cat:"native",c:"docker run -p 7687:7687 -p 7444:7444 --name memgraph memgraph/memgraph-mage",note:"Includes the MAGE algorithms. The Memgraph Lab UI is a separate image: docker run -d -p 3000:3000 --name lab memgraph/lab. On Mac and Windows add -e QUICK_CONNECT_MG_HOST=host.docker.internal."},
 {n:"MongoDB",cat:"doc",c:"docker run -p 27017:27017 mongo:8",note:"Local server; Atlas Vector Search needs Atlas or the Atlas CLI local deployment."},
 {n:"Kuzu",cat:"embed",c:"pip install kuzu==0.11.3",note:"Final upstream release. Use a community fork for anything you will keep."},
 {n:"PuppyGraph",cat:"over",c:"docker run -p 8081:8081 -p 8182:8182 -p 7687:7687 -d --name puppy --rm puppygraph/puppygraph:stable",note:"Free Developer edition, single node. Check docs.puppygraph.com for the current image tag."}
];
$("runList").innerHTML=RUNS.map((r,i)=>`<div><span class="pill" style="--c:${CV(r.cat)}">${r.n}</span><div class="snip" style="margin-top:6px"><pre><code>${esc(r.c)}</code></pre><button class="copy" type="button" data-copyrun="${i}">Copy</button></div><p class="note" style="margin-top:4px">${r.note}</p></div>`).join("");
document.querySelectorAll("[data-copyrun]").forEach(b=>b.onclick=()=>copyText(RUNS[+b.dataset.copyrun].c,b));

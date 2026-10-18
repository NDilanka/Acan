import * as kit from "/home/factory-user/.factory/skills/anatomy/kit/iso-kit.mjs";
import { WORLD } from "./world-data.mjs";
import { ACQUIT } from "../acquit.mjs";
import { ASSETS } from "../assets.mjs";
import { renderScene, projectionFor, layoutCallouts, calloutHtml } from "../scene.mjs";

const AZ = 35, EL = 35;
const BOUND = { "acquit:verifier": "verifier", "acquit:OTLP boundary": "otlp", "acan:receiver": "receiver", "kvarm:train": "train", "format:reference validator": "validator", "site:static waitlist": "preview" };
const bindingKey = (plant, name) => BOUND[plant + ":" + name] || plant + ":" + name;
const entityId = (plant, name) => (plant.ids && plant.ids[name]) || name;
for (const p of WORLD.plants) for (const name of p.machines) {
  const key = bindingKey(p.id, name);
  if (WORLD.bindings[key]) continue;
  const acquit = p.id === "acquit" && ACQUIT.entities.find(e => e.id === entityId(p, name));
  WORLD.bindings[key] = acquit
    ? { entity: p.name + " / " + name, channel: "runtime.reading", value: acquit.reading, state: "stale", sourceClass: "observed", locator: "otel://sandbox/acquit/" + acquit.id, revision: p.revision, age: 41, unit: "reading", meaning: "Last reading, not current", policy: "stale after 5 s", reason: "OTLP export is 41 s old." }
    : { entity: p.name + " / " + name, channel: "runtime.status", value: null, state: "unknown", sourceClass: "observed", locator: "collector://not-attached/" + p.id + "/" + name.replaceAll(" ", "-"), revision: p.revision, age: null, unit: "status", meaning: "Runtime not reported", policy: "5 s heartbeat", reason: "Catalog shell is designed. Runtime evidence is absent." };
}
WORLD.machineBindings = Object.fromEntries(WORLD.plants.flatMap(p => p.machines.map(name => [p.id + ":" + name, bindingKey(p.id, name)])));
const DRAWN = { unknown: "fog", stale: "stale", zero: "zero", stopped: "stopped", simulated: "sim", valid: "fog" };

function plantScene(plant) {
  if (plant.id !== "acquit") {
    const scene = structuredClone(plant.scene);
    for (const e of scene.entities) e.state = DRAWN[WORLD.bindings[bindingKey(plant.id, e.id)].state];
    return scene;
  }
  const scene = structuredClone(ACQUIT);
  scene.entities = scene.entities.filter(e => !["bot", "scaffold", "crane", "inspection"].includes(e.kind));
  for (const e of scene.entities) if (e.state !== "ghost") e.state = "stale";
  const verifier = scene.entities.find(e => e.id === "verifier");
  verifier.params = { cells: WORLD.bindings.verifier.value, busy: 0, queued: 0 };
  verifier.state = "fog";
  delete scene.wall;
  return scene;
}

function territoryLabels(scene, P) {
  return (scene.territories || []).map(t => {
    const ents = t.entities.map(id => scene.entities.find(e => e.id === id));
    const x0 = Math.min(...ents.map(e => e.x - ASSETS[e.kind].w * (e.size || 1) / 2)) - 12;
    const y1 = Math.max(...ents.map(e => e.y + ASSETS[e.kind].d * (e.size || 1) / 2)) + 12;
    return { id: t.id, state: t.state, label: t.label, pt: kit.iso([x0 + 3, y1 + 7, 0], P) };
  });
}

function draw(scene, box, scale, pad) {
  const P = projectionFor(kit, scene, ASSETS, { width: box[0], height: box[1], azimuth: AZ, elevation: EL, scale, pad });
  const out = renderScene(kit, ASSETS, scene, P, { width: box[0], height: box[1] });
  const svg = out.svg.replace(/data-id="g1" data-state="unclaimed"/, 'data-id="g1" data-state="unmatched"');
  return { svg, anchors: out.anchors, labels: out.labels, P };
}

const scenes = Object.fromEntries(WORLD.plants.map(p => [p.id, plantScene(p)]));
const fit = (scene, box, pad) => projectionFor(kit, scene, ASSETS, { width: box[0], height: box[1], azimuth: AZ, elevation: EL, pad }).scale;
const THUMB = [460, 265], STAGE = [1100, 620];
const thumbScale = fit(scenes.acquit, THUMB, 14), stageScale = fit(scenes.acquit, STAGE, 60);
const worldArt = { thumb: {}, stage: {}, cut: {} };
for (const p of WORLD.plants) {
  const scene = scenes[p.id];
  const thumb = draw(scene, THUMB, thumbScale);
  worldArt.thumb[p.id] = { svg: thumb.svg, anchors: thumb.anchors };
  const stage = draw(scene, STAGE, Math.min(stageScale * 1.35, fit(scene, STAGE, 110)));
  worldArt.stage[p.id] = { svg: stage.svg, anchors: stage.anchors, territories: territoryLabels(scene, stage.P) };
  for (const name of p.machines) {
    const id = entityId(p, name), e = scene.entities.find(x => x.id === id);
    if (!ASSETS[e.kind].explodes) continue;
    const solo = { entities: [{ ...e, explode: 1 }] };
    const cut = draw(solo, STAGE, Math.min(stageScale * 2.2, fit(solo, STAGE, 150)));
    const labels = cut.labels.map(l => ({ ...l, text: l.text.replace(/ · (busy|idle)$/, " · occupancy not reported").replace(/^\d+ waiting$/, "waiting tray · not reported") }));
    worldArt.cut[p.id + "/" + name] = { svg: cut.svg + calloutHtml(layoutCallouts(labels, { gap: 22, pad: 36 }), STAGE[0], STAGE[1]), anchor: cut.anchors[id] };
  }
}

function worldMount(D, art) {
  if (!conceptActive("world")) return;
  const root = document.querySelector("#worldApp");
  document.body.dataset.world = "active";
  document.title = "Acan · My world";
  const q = s => root.querySelector(s), all = s => [...root.querySelectorAll(s)];
  const esc = v => String(v).replace(/[&<>"']/g,c=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]));
  const params = new URLSearchParams(location.hash.slice(1));
  const plants = Object.fromEntries(D.plants.map(p=>[p.id,p]));
  const sites = Object.fromEntries(D.plants.map(p=>[p.id,[p.x,p.y]]));
  const state = window.worldState = {
    focus: null, level: "world", selection: null, mode: "live", floor: "All plants",
    camera: {x:0,y:0,s:1}, history:{i:0,n:1}, bookmarks:[], queueOpen:false, queue:[...D.queue],
    kvarm:{state:params.get("kv")||"ok",pending:[],checked:false},
    egressBytes:0, network:[], sites, bindings:structuredClone(D.bindings), belts:true,
    appearance:params.get("theme")||"system", theme:"dark", overlay:null, question:null, contract:null, arrival:false
  };
  const machineBinding = (plant, name) => ({
    "acquit:verifier":"verifier", "acquit:OTLP boundary":"otlp", "acan:receiver":"receiver",
    "kvarm:train":"train", "format:reference validator":"validator", "site:static waitlist":"preview"
  }[plant+":"+name] || plant+":"+name);
  for (const p of D.plants) for (const name of p.machines) {
    const key = machineBinding(p.id,name);
    if (!state.bindings[key]) state.bindings[key] = {
      entity:p.name+" / "+name, channel:"runtime.status", value:null, state:"unknown",
      sourceClass:"observed", locator:"collector://not-attached/"+p.id+"/"+name.replaceAll(" ","-"),
      revision:p.revision, age:null, unit:"status", meaning:"Runtime not reported", policy:"5 s heartbeat",
      reason:"Catalog shell is designed. Runtime evidence is absent."
    };
  }
  let wheelSum=0, drag=null, alt=false, searchIndex=0;
  const current = () => ({focus:state.focus,level:state.level,selection:state.selection,mode:state.mode,floor:state.floor,camera:{...state.camera},question:state.question,contract:state.contract});
  const entries = [current()];
  function operation(op) {
    state.network.push({...op,destination:D.origin});
    state.egressBytes += op.bytes;
  }
  if(state.kvarm.state === "ok" || state.kvarm.state === "stale") D.operations.forEach(operation);
  if(state.kvarm.state === "offline") state.kvarm.pending=structuredClone(D.pending);
  function refreshRemote() {
    state.bindings.train = {...D.bindings.train};
    if(state.kvarm.state !== "ok") {
      state.bindings.train.state="stale";
      state.bindings.train.reason=state.kvarm.state==="offline"?"Disconnected. Last authenticated value, not current.":"Two missed polls. Last authenticated value, not current.";
    }
  }
  refreshRemote();
  history.replaceState({world:true,index:0,view:current()},"");
  const announce = text => {q("#worldAnnouncement").textContent=text;};
  function appearance(value) {
    state.appearance=value;
    state.theme=value==="system"?(matchMedia("(prefers-color-scheme: light)").matches?"light":"dark"):value;
    root.dataset.theme=state.theme;
    document.body.dataset.worldTheme=state.theme;
    all(".iso").forEach(e=>e.dataset.theme=state.theme);
    q("#worldAppearance").value=value;
  }
  const bindText = b => b.text || [b.entity,b.channel,String(b.value??"not reported"),b.state,b.sourceClass,b.locator,"revision "+b.revision,b.age===null?"age unknown":b.age+" s",b.reason||b.meaning].join(" · ");
  function stateMark(id, full=false) {
    const b=state.bindings[id], value=b.state==="unknown"?"?":b.state==="stopped"?"⏻":b.state==="simulated"?"SIM":String(b.value);
    return `<span class="world-state-mark" data-binding="${id}" data-evidence-state="${b.state}" tabindex="0" aria-label="${esc(bindText(b))}"><span class="world-bezel">${esc(value)}</span><span>${esc(b.state)}${b.state==="stale"?" · "+b.age+" s":""}${full?"<small>"+esc(b.entity)+"</small>":""}</span></span>`;
  }
  function canvas() {
    const figure = p => p.id === "acquit" && state.level === "cutaway" && state.selection === "verifier" ? art.acquitCutaway : art[p.id];
    const machineTargets = p => p.machines.map(name => {
      const alias = { "acquit-api": "api", "OTLP boundary": "radar" };
      const pt = figure(p).anchors[alias[name] || name];
      if (!pt) return "";
      return `<button data-machine="${esc(name)}" data-machine-plant="${p.id}" style="left:${20+pt[0]*280/460}px;top:${-1+pt[1]*162/265}px">${esc(name)}</button>`;
    }).join("");
    q("#worldSites").innerHTML=D.plants.map(p=>`
      <section class="world-site ${state.focus===p.id?"world-focused":""}" data-world-site="${p.id}" style="left:${p.x}px;top:${p.y}px">
        <button class="world-plant-drawing iso" data-plant="${p.id}" data-theme="${state.theme}" aria-label="Open ${p.name} plant">${figure(p).svg}</button>
        <button class="world-plant-label" data-plant="${p.id}"><span class="world-site-id">${String(D.plants.indexOf(p)+1).padStart(2,"0")}</span><strong>${p.name}</strong><span class="world-decision-count">${state.queue.filter(r=>r.plant===p.id).length} <span>waiting</span></span></button>
        <div class="world-site-detail">${esc(p.env)} · ${esc(p.id==="kvarm" && state.kvarm.state!=="ok" ? "remote stale · 8 s · "+state.kvarm.state : p.freshness)}</div>
        <div class="world-condition">${stateMark(p.binding)}<span>${esc(p.condition)}</span></div>
        <div class="world-machine-targets">${machineTargets(p)}</div>
      </section>`).join("")+D.contracts.map(c=>{
        const numbers=c.path.match(/-?\d+(?:\.\d+)?/g).map(Number);
        return [[c.from,numbers[0],numbers[1]],[c.to,numbers.at(-2),numbers.at(-1)]].map(([id,x,y])=>`<button class="world-contract-port" data-world-endpoint="${c.id}" data-endpoint-plant="${id}" style="left:${x}px;top:${y}px" title="${plants[id].name} / ${id===c.from?c.fromEntity:c.toEntity} · ${c.label}" aria-label="Inspect ${c.label} endpoint in ${plants[id].name}"></button>`).join("");
      }).join("");
    q("#worldRoutes").innerHTML=`<defs><pattern id="worldFogHatch" width="6" height="6" patternUnits="userSpaceOnUse"><rect width="6" height="6" fill="var(--w-shell)"/><path d="M-1 1L1 -1M0 6L6 0M5 7L7 5" stroke="var(--w-line2)" stroke-width=".6"/></pattern></defs>`+D.contracts.map(c=>`<g data-route="${c.id}" class="world-route" data-kind="${c.kind}"><path class="world-route-under" d="${c.path}"/><path class="world-route-line" d="${c.path}"/><text x="${c.labelAt[0]}" y="${c.labelAt[1]}" text-anchor="middle">${c.label}</text></g>`).join("");
    q("#worldRoutes").style.display=state.belts?"":"none";
    q("#worldSites").dataset.level=state.level;
    q("#worldMapSites").innerHTML=D.plants.map(p=>`<button style="left:${p.x/16}px;top:${p.y/10}px" data-map="${p.id}" title="${p.name}">${p.id==="format"?"fmt":p.id==="site"?"site":p.name}</button>`).join("");
  }
  function queueRow(r,i) {
    return `<button class="world-queue-row" data-queue="${r.id}">
      <span class="world-row-meta"><b>${plants[r.plant].name}</b><span>${r.kind}</span>${i<4?"<kbd>⇧ "+(i+1)+"</kbd>":""}</span>
      <strong>${r.title}</strong><span>${r.why}</span><small>Waiting costs · ${r.cost}</small><small>Latest safe response · ${r.safe}</small><span class="world-estimate">${r.estimate} to review <span>Open ↗</span></span>
    </button>`;
  }
  function queue() {
    q("#worldQueue").hidden=!state.queueOpen;
    q("#worldQueueCount").textContent=state.queue.length;
    q("#worldQueueTitleCount").textContent=state.queue.length;
    q("#worldQueueRows").innerHTML=state.queue.map(queueRow).join("");
  }
  function paint() {
    root.dataset.level=state.level;
    root.dataset.mode=state.mode;
    q("#worldViewport").style.transform=`translate(${state.camera.x}px,${state.camera.y}px) scale(${state.camera.s})`;
    q("#worldBreadcrumb").textContent=state.focus?plants[state.focus].name:"My world";
    q("#worldScope").textContent=state.focus?plants[state.focus].env+" · "+plants[state.focus].revision+" · "+state.mode:"5 stable sites · live summary";
    q("#worldLevel").textContent=state.level;
    q("#worldBack").disabled=state.history.i===0;
    q("#worldForward").disabled=state.history.i>=state.history.n-1;
    q("#worldKvarm").innerHTML=`<span class="world-connection-dot ${state.kvarm.state}"></span>Kvarm · ${D.origin.replace("https://","")} · 3 repos · ${state.kvarm.state==="ok"?"fresh 8 s":state.kvarm.state}`;
    q("#worldEgress").textContent="↗ "+state.egressBytes.toLocaleString()+" B · declared egress";
    q("#worldBeltToggle").setAttribute("aria-pressed",state.belts);
    all("[data-world-mode]").forEach(b=>b.setAttribute("aria-pressed",b.dataset.worldMode===state.mode));
    canvas();queue();inspector();card();layers();
    q("#worldBookmarksCount").textContent=state.bookmarks.length||"";
    const viewX=Math.max(1,Math.min(151,(-state.camera.x/state.camera.s)/16));
    const viewY=Math.max(1,Math.min(86,(-state.camera.y/state.camera.s)/10));
    q("#worldMapView").style.cssText=`left:${viewX}px;top:${viewY}px;width:${Math.min(155,160/state.camera.s)}px;height:${Math.min(85,90/state.camera.s)}px`;
  }
  function save() {
    entries[state.history.i]=current();
    history.replaceState({world:true,index:state.history.i,view:current()},"");
  }
  function navigate(view, message) {
    save();
    Object.assign(state,view);
    state.overlay=null;state.queueOpen=false;
    entries.splice(state.history.i+1);
    state.history.i++;
    state.history.n=state.history.i+1;
    entries.push(current());
    history.pushState({world:true,index:state.history.i,view:current()},"");
    setMode(state.mode);
    paint();announce(message||"Place restored. No edit gesture is armed.");
    q("#worldCanvas").focus({preventScroll:true});
  }
  function plantView(id, selection=null, level="plant") {
    const p=plants[id],s=level==="plant"?2.2:2.65;
    return {focus:id,selection,level,question:null,contract:null,camera:{x:640-p.x*s,y:430-p.y*s,s},mode:"live",floor:id};
  }
  function goPlant(id,selection=null,level="plant") { navigate(plantView(id,selection,level),"Opened "+plants[id].name+". Local evidence only."); }
  function openRow(id) {
    const row=state.queue.find(r=>r.id===id);
    if(!row)return;
    const target=plantView(row.plant,row.entity,"machine");
    target.question=row.id;
    navigate(target,"Opened "+row.title+" in "+plants[row.plant].name);
  }
  function openQueue() {
    state.queueOpen=!state.queueOpen;state.overlay=null;paint();
  }
  function inspector() {
    const el=q("#worldInspector");
    el.hidden=!state.selection || state.level==="world";
    if(el.hidden)return;
    const p=plants[state.focus],key=machineBinding(p.id,state.selection),b=state.bindings[key];
    el.innerHTML=`<header><span class="world-eyebrow">${p.name} / ${state.selection}</span><button data-close="inspector" aria-label="Close inspector">×</button></header>
      <h2>${esc(state.selection)}</h2><p>${p.env} · ${p.revision} · read only</p>
      <div class="world-inspect-tabs"><span>${state.level==="cutaway"?"Cutaway":"Machine"}</span><button data-action="cutaway">${state.level==="cutaway"?"Close cutaway":"Open cutaway"} <kbd>X</kbd></button></div>
      <div class="world-binding-reading">${state.selection==="verifier"?"<strong>"+b.value+"</strong> declared cell":stateMark(key,true)}</div>
      <h3>Evidence binding</h3><dl><dt>Channel</dt><dd>${b.channel}</dd><dt>Value / state</dt><dd>${esc(b.value??"not reported")} · ${b.state}</dd><dt>Source</dt><dd>${b.sourceClass}</dd><dt>Locator</dt><dd>${b.locator}</dd><dt>Revision</dt><dd>${b.revision}</dd><dt>Age / policy</dt><dd>${b.age??"unknown"}${b.age===null?"":" s"} · ${b.policy}</dd></dl>
      ${b.runtime?`<p class="world-warning">${b.runtime}</p>`:""}
      <button class="world-action" data-provenance="${key}">Inspect provenance <kbd>Alt + hover</kbd></button>
      <h3>Identity</h3><p>${p.worktree}<br>${p.branch} · ${p.env}</p>
      ${state.focus==="acquit"?`<h3>Agent evidence</h3>${D.bots.map(b=>`<div class="world-bot"><b>${b.principal}</b> · ${b.session}<small>${b.evidence} · ${b.note}</small></div>`).join("")}`:""}
      <footer>Catalog geometry is designed. Fasteners and seams are non-semantic. Runtime is never inferred from a shell.</footer>`;
  }
  function card() {
    const el=q("#worldCard");
    el.hidden=!state.question&&!state.contract;
    q("#worldDeparture").hidden=state.question!=="p-2307";
    if(state.question==="p-2307") q("#worldDeparture").innerHTML=`<span class="world-eyebrow">Where you left it</span><p>${D.queue[1].departure}</p>`;
    if(el.hidden)return;
    const p=plants[state.focus],alias={"acquit-api":"api","OTLP boundary":"radar"};
    const pt=art[p.id].anchors[alias[state.selection]||state.selection];
    if(pt) {
      const x=state.camera.x+(p.x-140+pt[0]*280/460)*state.camera.s;
      const y=state.camera.y+(p.y-119+pt[1]*162/265)*state.camera.s;
      el.style.left=Math.max(24,Math.min(innerWidth-720,x+52))+"px";
      el.style.top=Math.max(165,Math.min(innerHeight-490,y-65))+"px";
    }
    if(state.contract) {
      const c=D.contracts.find(c=>c.id===state.contract);
      const far=state.focus===c.from?c.to:c.from;
      el.innerHTML=`<header><span class="world-eyebrow">Declared contract / ${c.kind}</span><button data-close="card">×</button></header><h2>${c.label}</h2><p>${c.detail}</p><p class="world-warning">${c.evidence}</p><p>${plants[c.from].name} / ${c.fromEntity}<br>→ ${plants[c.to].name} / ${c.toEntity}</p><button class="world-primary" data-far-end="${c.id}">Jump to other end · ${plants[far].name} ↗</button><small>Back restores this endpoint.</small>`;
    } else {
      const r=state.queue.find(r=>r.id===state.question);
      el.innerHTML=`<header><span class="world-eyebrow">${plants[r.plant].name} / ${r.entity}</span><button data-close="card">×</button></header><span class="world-tag">${r.kind}</span><h2>${r.title}</h2><p>${r.why}.</p><p>${r.cost}. ${r.safe}.</p>${r.id==="p-2307"?`<div class="world-readback"><b>readback · Acan</b><p>Claude Code keeps a failed payout pending.<br>Droid removes it from the effects queue.</p><span class="world-warning">w3 is not locked. No agent has been assigned.</span></div>`:""}<small>${r.estimate} estimated review · inspection only</small>`;
    }
  }
  function provenance(id,anchor) {
    state.overlay="provenance";state.provenance=id;
    const b=state.bindings[id],el=q("#worldProvenance");
    const box=anchor?.getBoundingClientRect();
    el.innerHTML=`<header><span class="world-eyebrow">Binding / ${b.sourceClass}</span><button data-close="overlay" aria-label="Close provenance">×</button></header><h2>${b.entity}</h2><p id="worldProvenanceText">${bindText(b)}</p><dl><dt>Channel</dt><dd>${b.channel}</dd><dt>State</dt><dd>${b.state}</dd><dt>Freshness</dt><dd>${b.age===null?"unknown":b.age+" s"} · ${b.policy}</dd></dl>`;
    el.style.left=Math.max(24,Math.min(innerWidth-510,box?box.left-200:700))+"px";
    el.style.top=Math.max(120,Math.min(innerHeight-350,box?box.bottom+12:380))+"px";
    layers();
  }
  function connection() {
    if(state.kvarm.state==="offline") {
      state.kvarm.state="stale";state.kvarm.checked=true;
      operation({op:"check_heads_and_leases",bytes:384,reason:"Reconnect without replaying pending writes"});
      announce("Checked heads and leases. Confirm pending writes with Enter.");
    } else {
      state.kvarm.state="offline";state.kvarm.checked=false;
      state.kvarm.pending=structuredClone(D.pending);
      announce("Kvarm is offline. Remote fields are stale. Local inspection still works.");
    }
    refreshRemote();state.overlay="network";paint();
  }
  function confirmWrites() {
    if(!state.kvarm.checked)return;
    state.kvarm.pending.forEach(operation);state.kvarm.pending=[];
    state.kvarm.state="ok";state.kvarm.checked=false;refreshRemote();paint();
    announce("Confirmed pending writes sent to the fixture origin.");
  }
  function bookmark() {
    if(state.bookmarks.length<4) state.bookmarks.push({label:state.focus?plants[state.focus].name+" / "+(state.selection||state.level):"My world",view:current()});
    state.overlay="bookmarks";paint();announce("Place marked. Camera, selection, mode and Floor scope saved.");
  }
  function recall(i) {
    const b=state.bookmarks[i];if(b)navigate(structuredClone(b.view),"Recalled "+b.label);
  }
  function layers() {
    q("#worldPalette").hidden=state.overlay!=="search";
    q("#worldStates").hidden=state.overlay!=="states";
    q("#worldProvenance").hidden=state.overlay!=="provenance";
    q("#worldNetwork").hidden=state.overlay!=="network";
    q("#worldBookmarks").hidden=state.overlay!=="bookmarks";
    q("#worldStatesList").innerHTML=D.plants.map(p=>`<button data-provenance="${p.binding}">${stateMark(p.binding,true)}<p>${state.bindings[p.binding].meaning}</p><small>${state.bindings[p.binding].locator}</small></button>`).join("");
    q("#worldNetworkBody").innerHTML=`<span class="world-tag">${state.kvarm.state} · scripted fixture</span><h2>Kvarm connection</h2><p class="world-origin">${D.origin}</p><p>Your own origin · 3 repos · opt-in declared egress.<br>No browser network requests are made by this prototype.</p>
      <button class="world-primary" data-action="connection">${state.kvarm.state==="offline"?"Reconnect":"Go offline"}</button>
      ${state.kvarm.state!=="ok"?`<div class="world-remote-warning"><b>Remote fields are stale</b><p>Train depth keeps its last authenticated value with age. Local bindings keep their own freshness.</p></div>`:""}
      ${state.kvarm.checked?"<p class='world-warning'>checked heads and leases</p><button class='world-action' data-action='confirm'>Confirm pending writes <kbd>Enter</kbd></button>":""}
      <h3>Pending writes · ${state.kvarm.pending.length}</h3>${state.kvarm.pending.map(p=>`<div class="world-network-row" data-pending-write><b>${p.op}</b><span>${p.path}</span><small>${p.reason} · ${p.bytes} B held locally</small></div>`).join("")||"<p>None.</p>"}
      <h3>Network log · ${state.egressBytes} B</h3>${state.network.map(p=>`<div class="world-network-row"><b>${p.op}<span>${p.bytes} B</span></b><span>${p.destination}</span><small>${p.reason}</small></div>`).join("")||"<p>No outbound operations.</p>"}`;
    q("#worldBookmarksBody").innerHTML=state.bookmarks.length?state.bookmarks.map((b,i)=>`<button class="world-action" data-recall="${i}">${esc(b.label)}<kbd>⇧ ${i+1}</kbd></button>`).join(""):"<p>Mark a place with M. Nothing moves until you recall it.</p>";
  }
  function results(query) {
    const term=query.toLowerCase();
    const items=[];
    if(!term) {
      entries.slice().reverse().slice(0,3).forEach(v=>items.push({group:"Recent",label:v.focus?plants[v.focus].name:"My world",detail:v.selection||v.level,view:v}));
      state.bookmarks.forEach((b,i)=>items.push({group:"Bookmarks",label:b.label,detail:"Saved place",bookmark:i}));
      state.queue.forEach(r=>items.push({group:"Waiting",label:r.title,detail:plants[r.plant].name+" · "+plants[r.plant].env,row:r.id}));
    } else {
      D.plants.forEach(p=>{
        if(p.name.toLowerCase().includes(term))items.push({group:"Plants",label:p.name,detail:p.env+" · "+p.revision,plant:p.id});
        p.machines.filter(m=>m.toLowerCase().includes(term)).forEach(m=>items.push({group:"Machines",label:m,detail:p.name+" · "+p.env,plant:p.id,machine:m}));
      });
      state.queue.filter(r=>(r.title+" "+plants[r.plant].name).toLowerCase().includes(term)).forEach(r=>items.push({group:"Waiting",label:r.title,detail:plants[r.plant].name+" · "+r.kind,row:r.id}));
      const commands=[{label:"Go offline",action:"connection",detail:"Kvarm · "+D.origin},{label:"Light appearance",action:"light",detail:"This window only"},{label:"Dark appearance",action:"dark",detail:"This window only"},{label:"System appearance",action:"system",detail:"Follow the operating system"}];
      commands.filter(c=>c.label.toLowerCase().includes(term)).forEach(c=>items.push({group:"Commands",...c}));
    }
    return items;
  }
  let searchResults=[];
  function paintSearch() {
    searchResults=results(q("#worldSearch").value);searchIndex=0;
    q("#worldResults").innerHTML=searchResults.map((r,i)=>`<button data-result="${i}" class="${i===0?"world-result-active":""}"><span class="world-eyebrow">${r.group}</span><strong>${esc(r.label)}</strong><small>${esc(r.detail)}</small><span>↗</span></button>`).join("")||"<p>No matching place.</p>";
  }
  function openSearch() {
    state.overlay="search";state.queueOpen=false;paint();
    q("#worldSearch").value="";paintSearch();q("#worldSearch").focus();
  }
  function chooseResult(i) {
    const r=searchResults[i];if(!r)return;
    if(r.row)openRow(r.row);
    else if(r.bookmark!==undefined)recall(r.bookmark);
    else if(r.view)navigate(structuredClone(r.view));
    else if(r.plant)goPlant(r.plant,r.machine||null,r.machine?"machine":"plant");
    else if(r.action==="connection")connection();
    else {appearance(r.action);state.overlay=null;paint();}
  }
  function cutaway() {
    if(!state.focus)return;
    if(!state.selection)state.selection=plants[state.focus].machines[0];
    state.level=state.level==="cutaway"?"machine":"cutaway";paint();
    announce(state.level==="cutaway"?"Cutaway open. Declared shell, bound evidence.":"Cutaway closed.");
  }
  function openContract(id, endpoint) {
    const c=D.contracts.find(c=>c.id===id);
    const focus=endpoint||([c.from,c.to].includes(state.focus)?state.focus:c.from);
    const entity=focus===c.from?c.fromEntity:c.toEntity;
    if(state.focus!==focus) {
      const view=plantView(focus,entity,"plant");view.contract=id;
      navigate(view,"Selected "+c.label+" endpoint in "+plants[focus].name);return;
    }
    state.selection=entity;
    state.question=null;state.contract=id;paint();
  }
  root.addEventListener("click",e=>{
    const b=e.target.closest("button");
    if(!b)return;
    if(b.dataset.plant)goPlant(b.dataset.plant);
    else if(b.dataset.worldEndpoint)openContract(b.dataset.worldEndpoint,b.dataset.endpointPlant);
    else if(b.dataset.map)goPlant(b.dataset.map);
    else if(b.dataset.machine)goPlant(b.dataset.machinePlant,b.dataset.machine,"machine");
    else if(b.dataset.queue)openRow(b.dataset.queue);
    else if(b.dataset.result!==undefined)chooseResult(+b.dataset.result);
    else if(b.dataset.provenance)provenance(b.dataset.provenance,b);
    else if(b.dataset.recall!==undefined)recall(+b.dataset.recall);
    else if(b.dataset.contract)openContract(b.dataset.contract);
    else if(b.dataset.farEnd) {
      const c=D.contracts.find(c=>c.id===b.dataset.farEnd),far=state.focus===c.from?c.to:c.from;
      const view=plantView(far,far===c.to?c.toEntity:c.fromEntity,"machine");
      view.contract=c.id;navigate(view,"Far end of "+c.label+" in "+plants[far].name);
    } else if(b.dataset.worldMode) {
      state.mode=b.dataset.worldMode;setMode(state.mode);paint();
    } else if(b.dataset.close) {
      if(b.dataset.close==="overlay")state.overlay=null;
      if(b.dataset.close==="queue")state.queueOpen=false;
      if(b.dataset.close==="card"){state.question=null;state.contract=null;}
      if(b.dataset.close==="inspector"){state.selection=null;state.level="plant";}
      paint();
    } else {
      const a=b.dataset.action;
      if(a==="search")openSearch();
      if(a==="queue")openQueue();
      if(a==="back")history.back();
      if(a==="forward")history.forward();
      if(a==="world")navigate({focus:null,selection:null,level:"world",question:null,contract:null,camera:{x:0,y:0,s:1},mode:"live",floor:"All plants"});
      if(a==="fit")state.focus?goPlant(state.focus):navigate({camera:{x:0,y:0,s:1}});
      if(a==="belts"){state.belts=!state.belts;paint();}
      if(a==="states"){state.overlay="states";paint();}
      if(a==="network"){state.overlay="network";paint();}
      if(a==="connection")connection();
      if(a==="confirm")confirmWrites();
      if(a==="mark")bookmark();
      if(a==="bookmarks"){state.overlay="bookmarks";paint();}
      if(a==="cutaway")cutaway();
    }
  });
  root.addEventListener("pointerover",e=>{
    const part=e.target.closest("[data-binding], [data-machine]");
    if(alt&&part)provenance(part.dataset.binding||machineBinding(part.dataset.machinePlant,part.dataset.machine),part);
  });
  q("#worldAppearance").addEventListener("change",e=>appearance(e.target.value));
  q("#worldSearch").addEventListener("input",paintSearch);
  addEventListener("popstate",e=>{
    if(!conceptActive("world")||!e.state?.world)return;
    Object.assign(state,e.state.view);
    state.history.i=e.state.index;state.overlay=null;state.queueOpen=false;
    setMode(state.mode);paint();announce("Returned to the saved camera, selection and mode.");
  });
  addEventListener("keydown",e=>{
    if(!conceptActive("world"))return;
    const eat=()=>{e.preventDefault();e.stopImmediatePropagation();};
    if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==="k"){eat();openSearch();return;}
    if(e.altKey&&["ArrowLeft","ArrowRight"].includes(e.key)&&!typing(e)){eat();e.key==="ArrowLeft"?history.back():history.forward();return;}
    if(e.key==="Alt"){alt=true;root.classList.add("world-alt");return;}
    if(e.key==="Escape") {
      eat();
      if(typing(e)){e.target.blur();return;}
      if(state.overlay){state.overlay=null;paint();return;}
      if(state.question||state.contract){state.question=null;state.contract=null;paint();return;}
      if(state.selection){state.selection=null;state.level="plant";paint();return;}
      if(state.queueOpen){state.queueOpen=false;paint();return;}
      return;
    }
    if(state.overlay==="search"&&["Enter","ArrowDown","ArrowUp"].includes(e.key)) {
      eat();
      if(e.key==="Enter")chooseResult(searchIndex);
      else {searchIndex=Math.max(0,Math.min(searchResults.length-1,searchIndex+(e.key==="ArrowDown"?1:-1)));all("[data-result]").forEach((b,i)=>b.classList.toggle("world-result-active",i===searchIndex));all("[data-result]")[searchIndex]?.scrollIntoView({block:"nearest"});}
      return;
    }
    if(e.key==="F8"&&!e.ctrlKey&&!e.metaKey&&!e.altKey){eat();appearance(state.theme==="light"?"dark":"light");announce("Appearance changed in this window. Your view has not moved.");return;}
    if(!bare(e))return;
    const key=e.key.toLowerCase(),digit=Number(e.code.replace("Digit",""));
    if(e.shiftKey&&digit>=1&&digit<=4&&(state.overlay==="bookmarks"||state.queueOpen)) {
      eat();state.overlay==="bookmarks"?recall(digit-1):openRow(state.queue[digit-1]?.id);return;
    }
    if(key==="u"){eat();openQueue();}
    if(key==="i"){eat();const i=state.queue.findIndex(r=>r.id===state.question);openRow(state.queue[(i+1)%state.queue.length].id);}
    if(key==="v"){eat();state.belts=!state.belts;paint();}
    if(key==="p"){eat();if(e.shiftKey)navigate({focus:null,selection:null,question:null,contract:null,level:"world",mode:"live",floor:"All plants",camera:{x:0,y:0,s:1}});else if(state.focus)goPlant(state.focus);}
    if(key==="m"){eat();bookmark();}
    if(key==="x"){eat();cutaway();}
    if(e.key==="Enter"&&state.overlay==="network"){eat();confirmWrites();}
    if(!e.shiftKey&&["1","2","3","4"].includes(e.key)&&state.focus){eat();state.mode=["live","blueprint","simulate","replay"][+e.key-1];setMode(state.mode);paint();}
  },{capture:true});
  addEventListener("keyup",e=>{if(e.key==="Alt"){alt=false;root.classList.remove("world-alt");}});
  addEventListener("blur",()=>{alt=false;root.classList.remove("world-alt");});
  const canvasEl=q("#worldCanvas");
  canvasEl.addEventListener("wheel",e=>{
    e.preventDefault();
    wheelSum+=e.deltaY;
    if(Math.abs(wheelSum)<110)return;
    const inward=wheelSum<0;wheelSum=0;
    if(inward&&state.level==="world")goPlant(state.focus||"acquit");
    else if(inward&&state.level==="plant")goPlant(state.focus,plants[state.focus].machines[0],"machine");
    else if(!inward&&state.level==="machine"){state.level="plant";paint();}
    else if(!inward&&state.level==="plant")navigate({focus:null,selection:null,question:null,contract:null,level:"world",mode:"live",floor:"All plants",camera:{x:0,y:0,s:1}});
  },{passive:false});
  canvasEl.addEventListener("pointerdown",e=>{
    if(e.target.closest("button,[data-binding]"))return;
    drag={x:e.clientX,y:e.clientY,camera:{...state.camera}};
    canvasEl.setPointerCapture(e.pointerId);
  });
  canvasEl.addEventListener("pointermove",e=>{
    if(!drag)return;
    state.camera.x=drag.camera.x+e.clientX-drag.x;state.camera.y=drag.camera.y+e.clientY-drag.y;
    q("#worldViewport").style.transform=`translate(${state.camera.x}px,${state.camera.y}px) scale(${state.camera.s})`;
  });
  canvasEl.addEventListener("pointerup",()=>{if(drag){drag=null;paint();announce("Canvas panned. World view stays read only. Cross-boundary edits are cancelled.");}});
  matchMedia("(prefers-color-scheme: light)").addEventListener("change",()=>{if(state.appearance==="system")appearance("system");});
  appearance(state.appearance);paint();
  setTimeout(()=>{
    state.queue.push(D.arrival);state.arrival=true;
    q("#worldQueueRows").insertAdjacentHTML("beforeend",queueRow(D.arrival,state.queue.length-1));
    q("#worldQueueCount").textContent=state.queue.length;
    q("#worldQueueTitleCount").textContent=state.queue.length;
    q('[data-world-site="site"] .world-decision-count').innerHTML='1 <span>waiting</span>';
    announce("acan-site needs an artifact check. Added to Waiting on you. Your view has not moved.");
  },3000);
}

export default {
  id: "world",
  label: "World",
  hint: "U waiting · I next · Ctrl K jump · Alt ← / → history · V contracts · P fit plant · Shift P world · M mark · X cutaway · F8 appearance",
  script: `${worldMount.toString()}\nfunction worldInit(params) { worldMount(${JSON.stringify(WORLD)}, ${JSON.stringify(worldArt)}); }`,
  html: `
<main id="worldApp">
  <header class="world-topbar">
    <div class="world-brand"><span class="world-logo">◎</span><b>Acan</b><span class="world-divider"></span><button data-action="world">My world</button><span class="world-slash">/</span><span id="worldBreadcrumb">My world</span><small id="worldScope"></small></div>
    <button class="world-jump" data-action="search"><span>⌕</span> Jump to a place, machine, or decision <kbd>Ctrl K</kbd></button>
    <div class="world-window-tools"><label class="world-appearance-label">Appearance<select id="worldAppearance" aria-label="Appearance"><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></label><button id="worldEgress" data-action="network"></button></div>
  </header>
  <div class="world-title"><span class="world-eyebrow">Personal workspace</span><h1>Your world.</h1><p>Five stable sites.<br>One place to return to.</p></div>
  <nav class="world-toolbar" aria-label="World navigation"><button id="worldBack" data-action="back" title="Back · Alt Left">←</button><button id="worldForward" data-action="forward" title="Forward · Alt Right">→</button><span></span><button data-action="world">Fit world <kbd>⇧ P</kbd></button><button data-action="fit">Fit plant <kbd>P</kbd></button><button id="worldBeltToggle" data-action="belts" aria-pressed="true">Contracts <kbd>V</kbd></button></nav>
  <div id="worldCanvas" tabindex="-1" aria-label="Infinite canvas of five stable plant sites"><div class="world-drafting-grid"></div><div id="worldViewport"><svg id="worldRoutes" viewBox="0 0 1600 900"></svg><div id="worldSites"></div></div></div>
  <nav class="world-contract-list" aria-label="Declared contracts"><span class="world-eyebrow">Boundary contracts</span><button data-contract="otlp">Acquit → Acan <span>OTLP / HTTP ↗</span></button><button data-contract="rest">Acan → Kvarm <span>REST · egress ↗</span></button><button data-contract="package">acan-format → Acan <span>v0.3.2 ↗</span></button><button data-contract="artifact">Acan → acan-site <span>web bundle ↗</span></button></nav>
  <div class="world-canvas-tools"><button data-action="states">Evidence states <span>? ◷ 0 ⏻ SIM</span></button><button data-action="mark">Mark place <kbd>M</kbd></button><button data-action="bookmarks">Bookmarks <span id="worldBookmarksCount"></span></button></div>
  <div class="world-bottom-left"><button class="world-waiting-button" data-action="queue"><span class="world-amber-dot"></span>Waiting on you <b id="worldQueueCount">5</b><kbd>U</kbd></button><button id="worldKvarm" data-action="connection"></button><span class="world-origin-hint">own origin · 3 repos · fixture</span></div>
  <aside id="worldMinimap"><header><span>Stable sites</span><span>N ↑</span></header><div id="worldMapSites"></div><div id="worldMapView"></div></aside>
  <footer class="world-footer"><span><b id="worldLevel">world</b> · read only · no activity animation</span><span>U waiting · I next · Ctrl K jump · Alt ← / → back / forward · F8 appearance</span><span class="world-fixture">Scripted evidence · no network</span></footer>
  <nav class="world-local-modes"><span>Focused plant only</span><button data-world-mode="live">Live <kbd>1</kbd></button><button data-world-mode="blueprint">Blueprint <kbd>2</kbd></button><button data-world-mode="simulate">Simulate <kbd>3</kbd></button><button data-world-mode="replay">Replay <kbd>4</kbd></button></nav>
  <section id="worldQueue" class="world-panel" hidden><header><div><span class="world-eyebrow">One human review allowance</span><h2>Waiting on you <span id="worldQueueTitleCount">5</span></h2></div><button data-close="queue" aria-label="Close Waiting on you">×</button></header><p class="world-queue-intro">Publication rules are enforced by Kvarm.<br>Recommended holds do not prevent self-claim.</p><div id="worldQueueRows"></div></section>
  <aside id="worldInspector" class="world-panel" hidden></aside>
  <section id="worldCard" class="world-panel" hidden></section>
  <aside id="worldDeparture" class="world-panel" hidden></aside>
  <section id="worldPalette" class="world-panel" role="dialog" aria-label="Global jump" hidden><header><span class="world-eyebrow">Jump anywhere / all plants</span><button data-close="overlay" aria-label="Close global jump">×</button></header><div class="world-search-field"><span>⌕</span><input id="worldSearch" placeholder="Plant, machine, decision, or command" autocomplete="off"></div><div id="worldResults"></div><footer>↑ ↓ choose · Enter jump · Esc dismisses without moving</footer></section>
  <section id="worldStates" class="world-panel" hidden><header><div><span class="world-eyebrow">Evidence, not activity</span><h2>Five states. Five different meanings.</h2></div><button data-close="overlay">×</button></header><p>Missing is not zero. A stopped process is not missing telemetry.</p><div id="worldStatesList"></div><footer>Each mark opens the exact binding that draws it. Failure and fog may coexist.</footer></section>
  <section id="worldProvenance" class="world-panel" hidden></section>
  <section id="worldNetwork" class="world-panel" hidden><header><span class="world-eyebrow">Declared egress / your connection</span><button data-close="overlay">×</button></header><div id="worldNetworkBody"></div></section>
  <section id="worldBookmarks" class="world-panel" hidden><header><div><span class="world-eyebrow">Places, not tabs</span><h2>Marked places</h2></div><button data-close="overlay">×</button></header><div id="worldBookmarksBody"></div><footer>Shift 1 to 4 recalls a place while this card is open.</footer></section>
  <div id="worldAnnouncement" role="status" aria-live="polite"></div>
</main>`,
  css: `
#worldApp{display:none}
body[data-world=active]>:not(#worldApp){display:none!important}
body[data-world=active]{background:#101112}
body[data-world=active] *{animation:none!important;transition:none!important}
#worldApp{--w-paper:#111314;--w-grid:#202324;--w-major:#2a2d2d;--w-ink:#e0e1dd;--w-dim:#999f9f;--w-faint:#7b8283;--w-line:#343b3c;--w-line2:#505a5b;--w-panel:#191d1e;--w-top:#202627;--w-accent:#c9a45c;--w-brass-fill:#463c29;--w-ok:#54b7a4;--w-wait:#e1b15b;--w-fail:#ef826e;--w-blue:#89b6d7;--w-shell:#161a1b;--w-shadow:0 12px 40px #0005;position:fixed;inset:0;background:var(--w-paper);color:var(--w-ink);font:14px/1.5 var(--sans);isolation:isolate;overflow:hidden}
#worldApp{--ok:var(--w-ok);--strain:var(--w-wait);--wait:var(--w-wait);--fail:var(--w-fail);--idle:var(--w-faint);--ghost:var(--w-blue);--text:var(--w-ink);--dim:var(--w-dim);--faint:var(--w-faint);--glass:var(--w-panel);--line:var(--w-line);--line2:var(--w-line2)}
body[data-world=active] #worldApp{display:block}
#worldApp[data-theme=light]{--w-paper:#F4F1EA;--w-grid:#E5E0D5;--w-major:#D6CFBF;--w-ink:#24282D;--w-dim:#50575E;--w-faint:#60656B;--w-line:#D4CFC3;--w-line2:#BDB5A5;--w-panel:#FCFAF5;--w-top:#F8F5ED;--w-accent:#795A27;--w-brass-fill:#E1D1AA;--w-ok:#17685D;--w-wait:#805400;--w-fail:#AC392B;--w-blue:#28658B;--w-shell:#EEEBE3;--w-shadow:0 8px 24px #29251F14,0 1px 3px #29251F0D}
#worldApp[data-theme=light][data-mode=blueprint] .world-focused{--w-mode-paper:#EDF3F8}
#worldApp[data-theme=light][data-mode=simulate] .world-focused{--w-mode-paper:#F0F4EE}
#worldApp[data-theme=light][data-mode=replay] .world-focused{--w-mode-paper:#F5ECDC}
#worldApp[data-theme=dark][data-mode=blueprint] .world-focused{--w-mode-paper:#0F2136}
#worldApp[data-theme=dark][data-mode=simulate] .world-focused{--w-mode-paper:#141B17}
#worldApp[data-theme=dark][data-mode=replay] .world-focused{--w-mode-paper:#1B160F}
#worldApp [hidden]{display:none!important}
#worldApp button{color:inherit;line-height:inherit}
#worldApp button:hover{color:var(--w-accent)}
#worldApp button:focus-visible,#worldApp input:focus-visible,#worldApp select:focus-visible,#worldApp [tabindex]:focus-visible{outline:2px solid var(--w-accent);outline-offset:4px}
#worldApp button:disabled{opacity:.3;cursor:default}
#worldApp kbd{color:var(--w-dim);border-color:var(--w-line2);font-size:10px;white-space:nowrap}
#worldApp h2{font-size:20px;line-height:1.3;letter-spacing:-.035em;font-weight:550;margin:8px 0 10px}
#worldApp h3{font:10px var(--mono);text-transform:uppercase;letter-spacing:.1em;color:var(--w-dim);margin:24px 0 9px}
#worldApp p{margin:8px 0 14px;color:var(--w-dim)}
#worldApp small{font:11px/1.5 var(--mono);color:var(--w-dim)}
#worldApp .world-eyebrow{font:10px/1.5 var(--mono);letter-spacing:.11em;text-transform:uppercase;color:var(--w-accent)}
#worldApp .world-topbar{position:absolute;z-index:25;left:0;right:0;top:0;height:72px;display:flex;align-items:center;justify-content:space-between;padding:0 28px;border-bottom:1px solid var(--w-line);background:var(--w-paper);gap:24px}
#worldApp .world-brand{display:flex;align-items:center;gap:12px;white-space:nowrap;position:relative;min-width:370px;font-size:13px}
#worldApp .world-brand>b{font-size:15px;letter-spacing:-.03em}
#worldApp .world-logo{font-size:28px;color:var(--w-accent);line-height:1}
#worldApp .world-divider{width:1px;height:20px;background:var(--w-line);margin:0 3px}
#worldApp .world-slash{color:var(--w-faint)}
#worldApp #worldBreadcrumb{font-weight:550}
#worldApp #worldScope{position:absolute;left:130px;top:28px;font:10px var(--mono);color:var(--w-dim)}
#worldApp[data-level=world] .world-slash,#worldApp[data-level=world] #worldBreadcrumb{display:none}
#worldApp .world-jump{width:350px;flex:none;display:flex;align-items:center;gap:10px;padding:9px 12px;border:1px solid var(--w-line);border-radius:7px;color:var(--w-dim);font-size:12px;background:var(--w-panel)}
#worldApp .world-jump>span{font-size:21px;line-height:16px}
#worldApp .world-jump kbd{margin-left:auto}
#worldApp .world-window-tools{display:flex;align-items:center;gap:20px}
#worldApp .world-appearance-label{display:flex;gap:7px;align-items:center;color:var(--w-dim);font-size:11px}
#worldApp select{background:var(--w-panel);color:var(--w-ink);border:1px solid var(--w-line);border-radius:5px;padding:5px 4px;font:11px var(--sans)}
#worldApp #worldEgress{font:10px var(--mono);color:var(--w-ok)}
#worldApp .world-title{position:absolute;left:40px;top:103px;z-index:3;pointer-events:none}
#worldApp .world-title h1{font-size:29px;letter-spacing:-.045em;font-weight:450;margin:6px 0 3px;line-height:1.25}
#worldApp .world-title p{font-size:13px;margin:0;color:var(--w-dim)}
#worldApp:not([data-level=world]) .world-title{display:none}
#worldApp .world-toolbar{position:absolute;right:30px;top:99px;z-index:8;display:flex;align-items:center;gap:16px;font-size:11px}
#worldApp .world-toolbar>span{width:1px;background:var(--w-line);height:16px}
#worldApp .world-toolbar button{display:flex;gap:7px;align-items:center}
#worldApp #worldBack,#worldApp #worldForward{font-size:18px}
#worldApp #worldCanvas{position:absolute;inset:73px 0 40px;outline:none;overflow:hidden;touch-action:none}
#worldApp .world-drafting-grid{position:absolute;inset:0;background-image:linear-gradient(var(--w-major) 1px,transparent 1px),linear-gradient(90deg,var(--w-major) 1px,transparent 1px),linear-gradient(var(--w-grid) 1px,transparent 1px),linear-gradient(90deg,var(--w-grid) 1px,transparent 1px);background-size:128px 128px,128px 128px,32px 32px,32px 32px;opacity:.45;mask-image:radial-gradient(ellipse at center,#0008,transparent 78%)}
#worldApp #worldViewport{position:absolute;left:0;top:-73px;width:1600px;height:900px;transform-origin:0 0}
#worldApp #worldRoutes{position:absolute;inset:0;width:1600px;height:900px;overflow:visible;pointer-events:none}
#worldApp .world-route-under{stroke:var(--w-paper);stroke-width:7;fill:none}
#worldApp .world-route-line{stroke:var(--w-line2);stroke-width:1.5;fill:none}
#worldApp .world-route[data-kind=runtime] .world-route-line{stroke-dasharray:7 4}
#worldApp .world-route[data-kind=egress] .world-route-line{stroke:var(--w-accent);stroke-dasharray:7 3}
#worldApp .world-route[data-kind=package] .world-route-line{stroke-dasharray:2 4}
#worldApp .world-route[data-kind=artifact] .world-route-line{stroke-dasharray:10 3 2 3}
#worldApp .world-route text{font:10px var(--mono);fill:var(--w-dim);paint-order:stroke;stroke:var(--w-paper);stroke-width:6;stroke-linejoin:round}
#worldApp:not([data-level=world]) .world-route text{display:none}
#worldApp .world-contract-port{position:absolute;width:9px;height:9px;border:1px solid var(--w-accent);background:var(--w-paper);border-radius:50%;transform:translate(-50%,-50%)}
#worldApp:not([data-level=world]) .world-contract-port{display:none}
#worldApp .world-site{position:absolute;width:320px;height:236px;transform:translate(-50%,-50%)}
#worldApp .world-focused::before{content:"";position:absolute;inset:-10px -12px;background:var(--w-mode-paper,transparent);border:1px solid var(--w-line);border-radius:5px}
#worldApp .world-plant-drawing{position:absolute;left:20px;top:-1px;width:280px;height:162px;background:transparent}
#worldApp .world-plant-drawing>svg{display:block;width:100%;height:100%;overflow:visible}
#worldApp .world-plant-label{position:absolute;left:5px;right:5px;top:161px;display:flex;align-items:center;gap:8px;text-align:left}
#worldApp .world-site-id{font:10px var(--mono);color:var(--w-accent)}
#worldApp .world-plant-label strong{font-size:19px;font-weight:500;letter-spacing:-.025em}
#worldApp .world-decision-count{margin-left:auto;border:1px solid var(--w-line2);border-radius:4px;padding:1px 6px;font:11px var(--mono);color:var(--w-wait);white-space:nowrap}
#worldApp .world-decision-count>span{font-size:10px;color:var(--w-dim)}
#worldApp .world-site-detail{position:absolute;left:27px;top:193px;font:10px var(--mono);color:var(--w-dim);white-space:nowrap}
#worldApp .world-condition{position:absolute;left:27px;top:214px;display:flex;align-items:center;gap:8px;font-size:11px;white-space:nowrap}
#worldApp .world-condition>.world-state-mark{font-size:9px}
#worldApp .world-condition>.world-state-mark>span:last-child{display:none}
#worldApp .world-state-mark{display:inline-flex;gap:8px;align-items:center;font:12px var(--mono);color:var(--w-dim)}
#worldApp .world-state-mark small{display:block;margin-top:3px}
#worldApp .world-bezel{border:1.5px solid var(--w-dim);width:34px;height:28px;display:inline-flex;align-items:center;justify-content:center;border-radius:50%;font:13px var(--mono);background:var(--w-shell);color:var(--w-ink)}
#worldApp [data-evidence-state=unknown] .world-bezel{background:repeating-linear-gradient(135deg,transparent 0 3px,var(--w-line2) 3px 4px);border-style:dotted}
#worldApp [data-evidence-state=stale] .world-bezel{border-style:dashed;color:var(--w-faint);text-decoration:line-through}
#worldApp [data-evidence-state=zero] .world-bezel{border-color:var(--w-ok);color:var(--w-ok);box-shadow:inset 0 -3px var(--w-ok)}
#worldApp [data-evidence-state=stopped] .world-bezel{border-radius:4px;background:var(--w-line);border-color:var(--w-dim)}
#worldApp [data-evidence-state=simulated] .world-bezel{border-radius:3px;border-color:var(--w-blue);color:var(--w-blue);font-size:9px;background-image:linear-gradient(var(--w-line) 1px,transparent 1px),linear-gradient(90deg,var(--w-line) 1px,transparent 1px);background-size:6px 6px}
#worldApp .world-machine-targets{display:none}
#worldApp:not([data-level=world]) .world-focused .world-machine-targets{display:block;position:absolute;inset:0;pointer-events:none}
#worldApp .world-machine-targets button{position:absolute;border:1px solid var(--w-line2);background:var(--w-panel);border-radius:3px;padding:2px 4px;font:6px var(--mono);white-space:nowrap;transform:translate(-50%,calc(-100% - 5px));pointer-events:auto}
#worldApp .world-machine-targets [data-machine=verifier]{border-color:var(--w-accent)}
#worldApp:not([data-level=world]) .world-site:not(.world-focused){opacity:.32}
#worldApp:not([data-level=world]) .world-plant-label strong{font-size:12px}
#worldApp:not([data-level=world]) .world-site-detail,#worldApp:not([data-level=world]) .world-condition{font-size:6px}
#worldApp:not([data-level=world]) .world-decision-count{font-size:6px}
#worldApp:not([data-level=world]) .world-decision-count>span{font-size:5px}
#worldApp:not([data-level=world]) .world-site-id{font-size:6px}
#worldApp:not([data-level=world]) .world-condition .world-bezel{width:15px;height:12px;font-size:6px;border-width:.7px}
#worldApp[data-level=cutaway] .world-focused::before{border-color:var(--w-accent);border-style:dashed}
#worldApp .world-contract-list{position:absolute;left:36px;bottom:142px;z-index:5;display:flex;flex-direction:column;gap:5px;width:245px}
#worldApp .world-contract-list>.world-eyebrow{margin-bottom:3px}
#worldApp .world-contract-list button{display:flex;justify-content:space-between;text-align:left;font-size:10px;color:var(--w-dim)}
#worldApp .world-contract-list button span{font:9px var(--mono);color:var(--w-faint)}
#worldApp .world-canvas-tools{position:absolute;right:30px;top:149px;display:flex;flex-direction:column;gap:10px;align-items:flex-end;z-index:7;font-size:11px}
#worldApp .world-canvas-tools button{display:flex;gap:10px;align-items:center}
#worldApp .world-canvas-tools span{font:10px var(--mono);color:var(--w-dim)}
#worldApp .world-bottom-left{position:absolute;left:30px;bottom:57px;display:grid;grid-template-columns:auto auto;column-gap:14px;align-items:center;z-index:12}
#worldApp .world-waiting-button{grid-row:span 2;display:flex;align-items:center;gap:10px;padding:13px 16px;border:1px solid var(--w-line2);border-radius:7px;background:var(--w-panel);box-shadow:var(--w-shadow);font-size:13px}
#worldApp .world-waiting-button b{border-radius:3px;padding:0 5px;background:var(--w-brass-fill);color:var(--w-accent);font:11px var(--mono)}
#worldApp .world-amber-dot{width:6px;height:6px;border-radius:50%;background:var(--w-wait)}
#worldApp #worldKvarm{font:10px var(--mono);text-align:left;display:flex;align-items:center;gap:6px}
#worldApp .world-connection-dot{width:5px;height:5px;border-radius:50%;background:var(--w-ok)}
#worldApp .world-connection-dot.offline,#worldApp .world-connection-dot.stale{background:var(--w-wait)}
#worldApp .world-origin-hint{font:9px var(--mono);color:var(--w-faint)}
#worldApp #worldMinimap{position:absolute;bottom:62px;right:30px;width:184px;height:139px;padding:10px 12px;border:1px solid var(--w-line);border-radius:6px;background:var(--w-panel);z-index:10}
#worldApp #worldMinimap header{display:flex;justify-content:space-between;font:9px var(--mono);color:var(--w-dim)}
#worldApp #worldMapSites{position:relative;margin-top:5px;width:160px;height:90px}
#worldApp #worldMapSites button{position:absolute;font:8px var(--mono);transform:translate(-50%,-50%);z-index:2}
#worldApp #worldMapSites button::before{content:"";display:block;width:4px;height:4px;border:1px solid var(--w-accent);margin:0 auto 2px;background:var(--w-brass-fill)}
#worldApp #worldMapView{position:absolute;margin:30px 12px 0 0;pointer-events:none;top:0;border:1px solid var(--w-line2);background:var(--w-accent);opacity:.06}
#worldApp .world-footer{position:absolute;bottom:0;left:0;right:0;height:39px;display:flex;justify-content:space-between;align-items:center;padding:0 30px;border-top:1px solid var(--w-line);font:9px var(--mono);color:var(--w-dim);background:var(--w-paper);z-index:25}
#worldApp .world-footer b{color:var(--w-accent);text-transform:uppercase;font-weight:500}
#worldApp .world-fixture{color:var(--w-faint)}
#worldApp .world-local-modes{position:absolute;top:96px;left:32px;display:flex;align-items:center;gap:3px;padding:4px;border:1px solid var(--w-line);border-radius:6px;background:var(--w-panel);z-index:8}
#worldApp[data-level=world] .world-local-modes{display:none}
#worldApp .world-local-modes>span{font-size:10px;color:var(--w-dim);padding:0 9px}
#worldApp .world-local-modes button{padding:5px 9px;font-size:11px;border-radius:3px}
#worldApp .world-local-modes button[aria-pressed=true]{background:var(--w-brass-fill);color:var(--w-accent)}
#worldApp .world-panel{position:absolute;z-index:18;background:var(--w-panel);border:1px solid var(--w-line2);border-radius:9px;box-shadow:var(--w-shadow);padding:20px;color:var(--w-ink)}
#worldApp .world-panel>header{display:flex;align-items:flex-start;justify-content:space-between;gap:10px}
#worldApp .world-panel header>button{font-size:21px;line-height:18px;color:var(--w-dim);width:20px;height:24px}
#worldApp .world-panel>header h2{margin-bottom:0}
#worldApp .world-panel>footer{padding-top:15px;margin-top:16px;border-top:1px solid var(--w-line);font:10px/1.6 var(--mono);color:var(--w-dim)}
#worldApp .world-panel dl{display:grid;grid-template-columns:85px 1fr;gap:9px;font:11px/1.6 var(--mono);margin:14px 0}
#worldApp .world-panel dt{color:var(--w-faint)}
#worldApp .world-panel dd{margin:0;overflow-wrap:anywhere}
#worldApp #worldQueue{left:30px;top:120px;width:430px;max-height:calc(100vh - 230px);overflow:hidden;padding:17px;display:flex;flex-direction:column}
#worldApp #worldQueue>header,#worldApp .world-queue-intro{flex-shrink:0}
#worldApp #worldQueueRows{overflow:auto;min-height:0;padding-right:2px}
#worldApp #worldQueue h2{font-size:22px}
#worldApp #worldQueueTitleCount{font:13px var(--mono);color:var(--w-accent);margin-left:9px}
#worldApp .world-queue-intro{font-size:11px;margin:8px 0 11px}
#worldApp .world-queue-row{width:100%;text-align:left;border-top:1px solid var(--w-line);padding:6px 0;display:flex;flex-direction:column;gap:1px;line-height:1.25}
#worldApp .world-row-meta{display:flex;align-items:center;gap:9px;width:100%;font:9px var(--mono)}
#worldApp .world-row-meta b{font-weight:500;color:var(--w-accent)}
#worldApp .world-row-meta>span{font-size:8px;color:var(--w-dim)}
#worldApp .world-row-meta kbd{margin-left:auto;font-size:9px}
#worldApp .world-queue-row>strong{font-size:14px;font-weight:550;line-height:1.3}
#worldApp .world-queue-row>span:not(.world-row-meta){font-size:11px;line-height:1.3;color:var(--w-dim)}
#worldApp .world-queue-row>small{font:10px/1.25 var(--sans);color:var(--w-dim)}
#worldApp .world-estimate{display:flex;justify-content:space-between;width:100%;padding-top:3px}
#worldApp .world-estimate>span{font-size:10px;color:var(--w-accent)}
#worldApp #worldInspector{top:154px;right:30px;width:320px;max-height:calc(100vh - 222px);overflow:auto;font-size:12px}
#worldApp #worldInspector h2{font-size:25px;margin-top:12px}
#worldApp .world-inspect-tabs{display:flex;align-items:center;justify-content:space-between;border-top:1px solid var(--w-line);border-bottom:1px solid var(--w-line);padding:9px 0;font-size:10px}
#worldApp .world-inspect-tabs>span{color:var(--w-accent)}
#worldApp .world-binding-reading{padding:17px 0 0;font:12px var(--mono)}
#worldApp .world-binding-reading>strong{font-size:36px;color:var(--w-accent);font-weight:400}
#worldApp .world-warning{color:var(--w-wait)!important;font-size:11px}
#worldApp .world-bot{font:10px/1.6 var(--mono);padding:8px 0;border-top:1px solid var(--w-line)}
#worldApp .world-bot small{display:block;font-size:9px}
#worldApp .world-bot b{font-weight:500}
#worldApp .world-action{display:flex;align-items:center;justify-content:space-between;gap:15px;padding:10px 12px;border:1px solid var(--w-line2);border-radius:5px;width:100%;font-size:11px}
#worldApp .world-action kbd{font-size:9px}
#worldApp .world-primary{display:block;border-radius:5px;background:var(--w-brass-fill);border:1px solid var(--w-accent);color:var(--w-ink);padding:10px 13px;font-size:12px;width:100%;margin:16px 0 10px;text-align:left}
#worldApp #worldCard{top:250px;left:710px;width:340px;font-size:12px}
#worldApp #worldCard h2{font-size:22px}
#worldApp .world-tag{display:inline-block;border:1px solid var(--w-line2);border-radius:3px;padding:2px 5px;font:9px var(--mono);color:var(--w-wait);margin:9px 0}
#worldApp .world-readback{border-top:1px solid var(--w-line);border-bottom:1px solid var(--w-line);padding:12px 0;margin:15px 0}
#worldApp .world-readback b{font:10px var(--mono);color:var(--w-accent)}
#worldApp #worldDeparture{left:32px;bottom:160px;width:310px;font-size:12px;border-style:dashed}
#worldApp #worldDeparture p{margin-bottom:0}
#worldApp #worldPalette{top:172px;left:50%;transform:translateX(-50%);width:620px;padding:18px 0 12px;z-index:35}
#worldApp #worldPalette>header,#worldApp #worldPalette>footer{margin:0 22px}
#worldApp #worldPalette>footer{font-size:10px}
#worldApp .world-search-field{display:flex;gap:14px;align-items:center;margin:14px 22px;padding:9px 0 15px;border-bottom:1px solid var(--w-line)}
#worldApp .world-search-field>span{font-size:29px;line-height:25px;color:var(--w-accent)}
#worldApp .world-search-field input{background:none;border:none;color:var(--w-ink);font:18px var(--sans);width:100%;outline:none}
#worldApp .world-search-field input::placeholder{color:var(--w-faint);font-size:16px}
#worldApp #worldResults{max-height:420px;overflow:auto;padding:0 10px}
#worldApp #worldResults>button{display:grid;width:100%;grid-template-columns:1fr 35px;text-align:left;padding:10px 12px;border-radius:5px;gap:3px}
#worldApp #worldResults>button .world-eyebrow{font-size:8px}
#worldApp #worldResults>button strong{grid-column:1;font-weight:500}
#worldApp #worldResults>button small{grid-column:1;font-size:10px}
#worldApp #worldResults>button>span:last-child{grid-column:2;grid-row:2;color:var(--w-accent)}
#worldApp .world-result-active{background:var(--w-brass-fill)}
#worldApp #worldStates{width:1040px;top:270px;left:50%;transform:translateX(-50%);z-index:24}
#worldApp #worldStates>p{font-size:13px}
#worldApp #worldStatesList{display:grid;grid-template-columns:repeat(5,1fr);gap:0;margin-top:24px}
#worldApp #worldStatesList>button{text-align:left;padding:15px;border-left:1px solid var(--w-line)}
#worldApp #worldStatesList>button:first-child{border-left:0;padding-left:0}
#worldApp #worldStatesList .world-state-mark{align-items:flex-start;flex-direction:column;gap:13px}
#worldApp #worldStatesList .world-bezel{width:58px;height:49px;font-size:21px}
#worldApp #worldStatesList [data-evidence-state=simulated] .world-bezel{font-size:14px}
#worldApp #worldStatesList p{font-size:12px;min-height:54px;margin:17px 0}
#worldApp #worldStatesList small{font-size:9px;overflow-wrap:anywhere}
#worldApp #worldProvenance{width:480px;z-index:40;font-size:13px}
#worldApp #worldProvenanceText{line-height:1.9}
#worldApp #worldProvenanceText::first-line{color:var(--w-ink)}
#worldApp #worldNetwork{right:30px;top:154px;width:430px;max-height:calc(100vh - 222px);overflow:auto;z-index:30;font-size:12px}
#worldApp #worldNetworkBody>.world-tag{margin-top:12px}
#worldApp .world-origin{font:13px var(--mono);color:var(--w-ink)}
#worldApp .world-remote-warning{border:1px dashed var(--w-wait);border-radius:5px;padding:11px;margin:16px 0}
#worldApp .world-remote-warning>b{color:var(--w-wait);font-size:12px}
#worldApp .world-remote-warning p{margin:4px 0 0;font-size:11px}
#worldApp #worldNetwork h3{margin:16px 0 7px}
#worldApp .world-network-row{display:flex;flex-direction:column;gap:3px;border-top:1px solid var(--w-line);padding:8px 0;font:10px var(--mono)}
#worldApp .world-network-row>b{font-weight:500;display:flex;justify-content:space-between}
#worldApp .world-network-row>span{color:var(--w-dim)}
#worldApp .world-network-row small{font:10px/1.5 var(--sans)}
#worldApp #worldBookmarks{left:50%;transform:translateX(-50%);top:275px;width:470px;z-index:28}
#worldApp #worldBookmarksBody{display:grid;gap:10px;margin-top:20px}
#worldApp #worldAnnouncement{position:absolute;top:76px;left:50%;transform:translateX(-50%);background:var(--w-panel);border:1px solid var(--w-line);border-radius:4px;padding:3px 10px;z-index:22;max-width:700px;font-size:9px;line-height:1.4;color:var(--w-dim);pointer-events:none}
#worldApp #worldAnnouncement:empty{display:none}
#worldApp .iso{--anatomy-shade-0:#121617;--anatomy-shade-1:#1a2021;--anatomy-shade-2:#242c2d;--anatomy-shade-3:#303839;--anatomy-top:#353d3e;--anatomy-hi:#bdc5c5;--anatomy-mid:#7e8989;--anatomy-lo:#4b5657;--anatomy-faint:#384344;--anatomy-weight:.8px}
#worldApp[data-theme=light] .iso{--anatomy-paper:#F2EEE4;--anatomy-face:#F8F5ED;--anatomy-top:#FCFAF5;--anatomy-shade-0:#C5C0B4;--anatomy-shade-1:#D5D0C5;--anatomy-shade-2:#E4E0D6;--anatomy-shade-3:#EEEBE3;--anatomy-lit-top:#FCFAF5;--anatomy-lit-shade:#E4E0D6;--anatomy-hi:#343A40;--anatomy-mid:#59616A;--anatomy-lo:#929187;--anatomy-faint:#CBC7BD;--anatomy-lit:#1B1F24;--anatomy-halo:#7B7465;--anatomy-halo-opacity:.12;--anatomy-dot:#795A27;--anatomy-red:#AC392B;--anatomy-green:#17685D;--anatomy-blue:#28658B;--anatomy-weight:1px}
#worldApp .iso-face-text{fill:var(--w-ink)}
#worldApp .ent[data-state=fog]{filter:none;opacity:1}
#worldApp .ent[data-state=fog] .iso-top{fill:url(#worldFogHatch)}
#worldApp .ent[data-state=fog] .iso-edge{stroke-dasharray:2 2}
#worldApp .ent[data-state=fog] .lamp{display:none}
#worldApp .ent[data-state=fog] .iso-dots{opacity:.5}
#worldApp.world-alt .world-machine-targets{display:block}
@media(max-width:1200px){#worldApp .world-topbar{gap:12px;padding:0 18px}#worldApp .world-brand{min-width:280px}#worldApp .world-jump{width:290px;font-size:10px}#worldApp .world-window-tools{gap:9px}#worldApp .world-appearance-label{font-size:0}#worldApp #worldEgress{font-size:8px}}
`
};

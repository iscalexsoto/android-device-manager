'use strict';

const TAURI = window.__TAURI__;
const invoke = (cmd, args) => TAURI.core.invoke(cmd, args);
const appWindow = TAURI.window.getCurrentWindow();

/* ------------------------------------------------------------------ */
/* Iconos · rejilla 24, trazo 1.75, currentColor, sin relleno           */
/* ------------------------------------------------------------------ */
const P = {
  phone: '<rect x="7" y="3" width="10" height="18" rx="2"/><path d="M11 18h2"/>',
  screens: '<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>',
  camera: '<rect x="3" y="6" width="18" height="13" rx="2"/><circle cx="12" cy="12.5" r="3.5"/><path d="M9 6l1.5-2h3L15 6"/>',
  install: '<path d="M12 4v10M8 10l4 4 4-4"/><path d="M5 15v3.5A1.5 1.5 0 0 0 6.5 20h11a1.5 1.5 0 0 0 1.5-1.5V15"/>',
  wifi: '<path d="M4.5 10a10.5 10.5 0 0 1 15 0"/><path d="M7.5 13.2a6.2 6.2 0 0 1 9 0"/><path d="M10.4 16.3a2.3 2.3 0 0 1 3.2 0"/><path d="M12 19.5h.01"/>',
  restart: '<path d="M20 12a8 8 0 1 1-2.34-5.66"/><path d="M20 4v4h-4"/>',
  chevron: '<path d="M6 9l6 6 6-6"/>',
  link: '<path d="M10 14a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1"/><path d="M14 10a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1"/>',
  server: '<rect x="4" y="4" width="16" height="7" rx="1.5"/><rect x="4" y="13" width="16" height="7" rx="1.5"/><path d="M8 7.5h.01M8 16.5h.01"/>',
  log: '<path d="M9 6h11M9 12h11M9 18h11M4.5 6h.01M4.5 12h.01M4.5 18h.01"/>',
  settings: '<circle cx="12" cy="12" r="3.5"/><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1"/>',
  min: '<path d="M5 12h14"/>',
  max: '<rect x="5.5" y="5.5" width="13" height="13" rx="1.5"/>',
  restore: '<rect x="8" y="8" width="11" height="11" rx="1.5"/><path d="M5 15V6.5A1.5 1.5 0 0 1 6.5 5H15"/>',
  close: '<path d="M6.5 6.5l11 11M17.5 6.5l-11 11"/>',
  refresh: '<path d="M20 11a8 8 0 0 0-14.9-3.5"/><path d="M4 4v3.5h3.5"/><path d="M4 13a8 8 0 0 0 14.9 3.5"/><path d="M20 20v-3.5h-3.5"/>',
  usb: '<path d="M9 3v4M15 3v4"/><path d="M7 7h10v4a5 5 0 0 1-10 0z"/><path d="M12 16v5"/>',
  emulator: '<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M9 10l-2 2 2 2M15 10l2 2-2 2"/>',
  terminal: '<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M7 10l3 2-3 2M12.5 15h4.5"/>',
  folder: '<path d="M3 7.5a1.5 1.5 0 0 1 1.5-1.5h4l2 2.5h8A1.5 1.5 0 0 1 20 10v7.5a1.5 1.5 0 0 1-1.5 1.5h-14A1.5 1.5 0 0 1 3 17.5z"/>',
  stop: '<rect x="6.5" y="6.5" width="11" height="11" rx="1.5"/>',
  play: '<path d="M8 5.5v13l10-6.5z"/>',
  front: '<path d="M14 4h6v6M20 4l-8.5 8.5"/><path d="M18 14v4.5a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 4 18.5v-11A1.5 1.5 0 0 1 5.5 6H10"/>',
  warning: '<path d="M12 4l9 16H3z"/><path d="M12 10v4M12 17h.01"/>',
  check: '<path d="M5 12.5l4.5 4.5L19 7"/>',
  kill: '<circle cx="12" cy="12" r="8.5"/><path d="M9 9l6 6M15 9l-6 6"/>',
  power: '<path d="M12 3v8"/><path d="M6.3 6.8a8 8 0 1 0 11.4 0"/>',
  copy: '<rect x="8" y="8" width="12" height="12" rx="1.5"/><path d="M16 8V5.5A1.5 1.5 0 0 0 14.5 4h-9A1.5 1.5 0 0 0 4 5.5v9A1.5 1.5 0 0 0 5.5 16H8"/>',
  record: '<circle cx="12" cy="12" r="8.5"/><circle cx="12" cy="12" r="3.5" fill="currentColor" stroke="none"/>',
  search: '<circle cx="11" cy="11" r="6.5"/><path d="M16 16l4 4"/>',
  trash: '<path d="M4 7h16M9 7V4.5h6V7M6.5 7l1 12.5h9l1-12.5"/>',
};
function icon(name, size = 18, sw = 1.75) {
  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="${sw}" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${P[name] || ''}</svg>`;
}
/** Arco que gira: indica trabajo en curso. */
function spinner(size = 16) {
  return `<svg class="spin" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="8.5" opacity=".25"/><path d="M12 3.5a8.5 8.5 0 0 1 8.5 8.5"/></svg>`;
}

/* ------------------------------------------------------------------ */
/* Estado                                                              */
/* ------------------------------------------------------------------ */
const S = {
  snap: null,
  view: 'device',
  selected: null,
  logs: [],
  openLogs: new Set(),
  instances: null,
  consoles: {},       // serial -> { html, history: [], idx }
  busy: new Set(),    // acciones en curso
  installs: new Map(), // id -> último evento `install`
  tools: new Map(),    // adb | scrcpy -> último evento `tool`
};

const INSTALL_ACTIVE = ['uninstall', 'copy', 'install'];
const activeInstall = (serial) => [...S.installs.values()].find((x) => x.serial === serial && INSTALL_ACTIVE.includes(x.phase));

const $ = (sel, root = document) => root.querySelector(sel);
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

/** Reemplaza el HTML solo si cambió (conserva foco y selects abiertos). */
function patch(el, html) {
  if (!el) return;
  if (el._html !== html) { el._html = html; el.innerHTML = html; }
}

const devices = () => S.snap?.devices ?? [];
/** Ventanas de scrcpy abiertas fuera de esta instancia (p. ej. de una ejecución anterior). */
const externalMirrors = () => {
  const own = new Set([...(S.snap?.sessions ?? []), ...(S.snap?.recordings ?? [])].map((s) => s.pid));
  return (S.instances?.scrcpy ?? []).filter((x) => !own.has(x.pid));
};
const externalFor = (serial) => externalMirrors().find((x) => x.serial === serial);
const hasScreen = (d) => d && (d.session || externalFor(d.serial));
const recordingOf = (serial) => (S.snap?.recordings ?? []).find((r) => r.serial === serial);
const selectedDevice = () => devices().find((d) => d.serial === S.selected) || null;
const isOnline = (d) => d && d.state === 'device';

function deviceName(d) {
  if (!d) return '';
  const det = d.details;
  return (det && (det.name || det.model)) || d.model || d.serial;
}

const STATE_INFO = {
  device: ['green', 'En línea'],
  unauthorized: ['orange', 'Sin autorizar'],
  authorizing: ['orange', 'Autorizando'],
  connecting: ['orange', 'Conectando'],
  offline: ['danger', 'Sin conexión'],
  recovery: ['orange', 'Recovery'],
  bootloader: ['orange', 'Bootloader'],
  sideload: ['orange', 'Sideload'],
  rescue: ['orange', 'Rescate'],
};
const stateInfo = (s) => STATE_INFO[s] || (s?.startsWith('no permissions') ? ['danger', 'Sin permisos'] : ['orange', s || 'Desconocido']);
const TRANSPORT = { usb: ['usb', 'USB'], wifi: ['wifi', 'Wi-Fi'], emulator: ['emulator', 'Emulador'] };

/* ------------------------------------------------------------------ */
/* Tema                                                                */
/* ------------------------------------------------------------------ */
const mq = window.matchMedia('(prefers-color-scheme: dark)');
function applyTheme() {
  const pref = S.snap?.settings?.theme || 'system';
  const dark = pref === 'dark' || (pref === 'system' && mq.matches);
  document.documentElement.dataset.theme = dark ? 'dark' : 'light';
}
mq.addEventListener('change', applyTheme);

/* ------------------------------------------------------------------ */
/* Render: marco                                                        */
/* ------------------------------------------------------------------ */
function renderChrome() {
  const srv = S.snap?.server;
  const chip = $('#server-chip');
  let cls = 'danger', text = 'Sin servidor';
  if (srv?.starting) { cls = 'orange'; text = 'Iniciando'; }
  else if (srv?.running) { cls = 'green'; text = S.snap.settings.serverPort === 5037 ? 'Conectado' : 'Servidor aislado'; }
  patch(chip, `<span class="dot ${cls}" data-tauri-drag-region></span><span data-tauri-drag-region>${text}</span>${srv ? `<span class="mono" data-tauri-drag-region>:${srv.port}</span>` : ''}`);

  // Barra de herramientas: acciones sobre el dispositivo seleccionado.
  const d = selectedDevice();
  const on = isOnline(d);
  const dis = on ? '' : 'disabled';
  const det = d?.details;
  const rec = d && recordingOf(d.serial);
  const inst = d && activeInstall(d.serial);
  const busy = (k) => S.busy.has(k);
  const readFull = on && det
    ? [det.model || d.model, det.android && `Android ${det.android}`, det.resolution, det.battery != null && `${det.battery} %`].filter(Boolean).join(' · ')
    : d ? `${d.serial} · ${stateInfo(d.state)[1]}` : 'Sin dispositivo';
  const readShort = on && det ? [det.android && `Android ${det.android}`, det.battery != null && `${det.battery} %`].filter(Boolean).join(' · ') : '';
  patch($('#toolbar'), `
    <button class="btn" data-act="mirror" ${dis} data-tip="Ver pantalla">${icon(hasScreen(d) ? 'front' : 'screens')}<span class="lbl">${hasScreen(d) ? 'Traer pantalla' : 'Ver pantalla'}</span></button>
    <button class="btn" data-act="screenshot" ${dis} data-tip="Captura">${busy('shot') ? spinner(18) : icon('camera')}<span class="lbl">Captura</span></button>
    ${rec
      ? `<button class="btn recording" data-act="record-stop" data-tip="Detener grabación"><span class="dot danger pulse"></span><span class="lbl">Grabando <span data-since="${rec.started}">${fmtDur(Date.now() - rec.started)}</span></span></button>`
      : `<button class="btn" data-act="record-pop" ${dis} data-tip="Grabar pantalla">${icon('record')}<span class="lbl">Grabar</span></button>`}
    ${inst
      ? `<button class="btn selected" data-tip="${esc(installLabel(inst))}" aria-live="polite">${spinner(18)}<span class="lbl">${esc(installLabel(inst))}</span></button>`
      : `<button class="btn" data-act="install" ${dis} data-tip="Instalar APK">${icon('install')}<span class="lbl">Instalar APK</span></button>`}
    <button class="btn" data-act="wifi" ${on && d.transport === 'usb' ? '' : 'disabled'} data-tip="Depuración por Wi-Fi">${icon('wifi')}<span class="lbl">Wi-Fi</span></button>
    <button class="btn" data-act="device-menu" ${d ? '' : 'disabled'} data-tip="Dispositivo">${icon('restart')}<span class="lbl">Dispositivo</span><span class="caret">${icon('chevron', 14)}</span></button>
    <div class="tb-spacer"></div>
    <div class="readout mono"><span class="ro-full">${esc(readFull)}</span><span class="ro-short">${esc(readShort)}</span></div>
    <div class="tb-sep"></div>
    <button class="btn" data-act="connect-pop" data-tip="Conectar por IP">${icon('link')}<span class="lbl">Conectar</span></button>
  `);

  // Rail derecho.
  const sessions = (S.snap?.sessions?.length || 0) + externalMirrors().length;
  const recording = (S.snap?.recordings?.length || 0) > 0;
  const issues = S.instances?.issues || [];
  const worst = issues.some((i) => i.level === 'danger') ? 'danger' : issues.some((i) => i.level === 'warning') ? 'orange' : '';
  const r = (view, ic, tip, extra = '') =>
    `<button class="icon-btn ${S.view === view ? 'selected' : ''}" data-act="nav" data-view="${view}" data-tip="${tip}" data-tip-side="left">${icon(ic, 20)}${extra}</button>`;
  patch($('#rail'), `
    ${r('device', 'phone', 'Dispositivo')}
    ${r('mirror', 'screens', 'Pantallas y grabaciones',
      recording ? '<span class="badge-dot pulse" style="background:var(--danger)"></span>' : sessions ? `<span class="badge">${sessions}</span>` : '')}
    <div class="rail-div"></div>
    ${r('adb', 'server', 'Servidor ADB', worst ? `<span class="badge-dot" style="background:var(--${worst})"></span>` : '')}
    ${r('log', 'log', 'Registro')}
    <div class="rail-spacer"></div>
    ${r('settings', 'settings', 'Ajustes')}
  `);
}

function renderSidebar() {
  const list = devices();
  patch($('#sb-title'), `Dispositivos · ${list.length}`);
  if (!list.length) {
    const srv = S.snap?.server;
    patch($('#device-list'), `<div class="sb-empty">${srv?.running
      ? 'Conecta un teléfono por USB con la depuración USB activada, o conéctalo por IP.'
      : 'El servidor adb no está en ejecución.'}</div>`);
  } else {
    patch($('#device-list'), list.map((d) => {
      const [dot, label] = stateInfo(d.state);
      const [tic] = TRANSPORT[d.transport] || ['phone'];
      const sub = isOnline(d) && d.details?.android ? `Android ${d.details.android} · ${d.serial}` : `${label} · ${d.serial}`;
      return `<div class="dev-item ${d.serial === S.selected ? 'selected' : ''}" data-act="select" data-serial="${esc(d.serial)}">
        <div class="dev-ico">${icon(tic, 16)}</div>
        <div class="dev-main"><div class="dev-name">${esc(deviceName(d))}</div><div class="dev-sub mono">${esc(sub)}</div></div>
        <div class="dev-side">${recordingOf(d.serial) ? '<span class="dot danger pulse" title="Grabando"></span>' : ''}${hasScreen(d) ? `<span title="Pantalla abierta">${icon('screens', 14)}</span>` : ''}<span class="dot ${dot}"></span></div>
      </div>`;
    }).join(''));
  }
  const s = S.snap;
  const ver = S.instances?.clientVersion;
  patch($('#sb-foot'), s ? `adb${ver ? ' ' + esc(ver.tools) : ''} · 127.0.0.1:${s.server.port}` : '');
}

/* ------------------------------------------------------------------ */
/* Render: vistas                                                       */
/* ------------------------------------------------------------------ */
function renderStage() {
  const stage = $('#stage');
  if (stage.dataset.view !== S.view) {
    stage.dataset.view = S.view;
    stage._html = null;
    stage.innerHTML = '';
    stage.scrollTop = 0;
  }
  ({ device: viewDevice, mirror: viewMirror, adb: viewAdb, log: viewLog, settings: viewSettings })[S.view](stage);
}

/* ---- Dispositivo ---- */
function viewDevice(stage) {
  const d = selectedDevice();
  const srv = S.snap?.server;
  if (!d) {
    stage.dataset.serial = '';
    if (!S.snap.adbPath) {
      patch(stage, `<div class="empty">
        <img src="logo.svg" alt="">
        <h2>No se encontró adb</h2>
        <p>Esta app necesita adb (Android SDK Platform-Tools) para hablar con los teléfonos. Si usas Android Studio, instálalo desde SDK Manager; si no, descárgalo aquí desde Google.</p>
        <div class="row">${toolButton('adb')}<button class="btn btn-secondary" data-act="nav" data-view="settings">${icon('settings')}Indicar ruta</button></div>
      </div>`);
      return;
    }
    patch(stage, `<div class="empty">
      <img src="logo.svg" alt="">
      <h2>${srv?.running ? 'Sin dispositivos' : 'Servidor adb detenido'}</h2>
      <p>${srv?.running
        ? 'Conecta un teléfono por USB y acepta la huella de depuración en la pantalla del teléfono. También puedes conectarlo por Wi-Fi.'
        : 'Inicia el servidor para detectar los dispositivos. Esta app usa el mismo adb que Android Studio, así que no se pisan.'}</p>
      ${srv?.running
        ? `<div class="row"><button class="btn btn-primary" data-act="connect-pop">${icon('link')}Conectar por IP</button></div>
           <div class="steps mono">Ajustes → Opciones de desarrollador → Depuración USB</div>`
        : `<button class="btn btn-primary" data-act="server" data-action="start">${icon('play')}Iniciar servidor</button>`}
    </div>`);
    return;
  }

  // Estructura estable: cabecera, métricas y opciones se parchean; la consola se crea una vez.
  if (stage.dataset.serial !== d.serial || !$('#dv-head', stage)) {
    stage.dataset.serial = d.serial;
    stage._html = null;
    stage.scrollTop = 0;
    stage.innerHTML = `<div class="view">
      <div id="dv-head"></div><div id="dv-banner"></div><div id="dv-stats"></div>
      <div class="grid-2"><div id="dv-options"></div><div id="dv-console"></div></div>
    </div>`;
    buildConsole(d.serial);
  }

  const det = d.details;
  const [dot, label] = stateInfo(d.state);
  const [tic, tlabel] = TRANSPORT[d.transport] || ['phone', d.transport];
  const on = isOnline(d);
  const meta = [det?.manufacturer && cap(det.manufacturer), det?.android && `Android ${det.android}`, det?.sdk && `SDK ${det.sdk}`].filter(Boolean);

  patch($('#dv-head'), `<div class="dev-header">
    <div class="grow col" style="gap:6px">
      <div class="dev-title">${esc(deviceName(d))}</div>
      <div class="dev-meta">
        <span class="chip"><span class="dot ${dot}"></span>${label}</span>
        <span class="chip">${icon(tic, 12, 2)}${tlabel}</span>
        ${meta.length ? `<span>${esc(meta.join(' · '))}</span>` : ''}
        <span class="mono faint selectable">${esc(d.serial)}</span>
      </div>
    </div>
    <div class="row">
      ${on ? `<button class="icon-btn" data-act="refresh-device" data-tip="Actualizar datos">${icon('refresh')}</button>` : ''}
      ${hasScreen(d)
        ? `${d.session
            ? `<button class="btn btn-danger" data-act="mirror-stop" data-id="${d.session}">${icon('stop')}Cerrar pantalla</button>`
            : `<button class="btn btn-danger" data-act="ext-close" data-pid="${externalFor(d.serial).pid}">${icon('stop')}Cerrar pantalla</button>`}
           <button class="btn btn-primary" data-act="mirror">${icon('front')}Traer pantalla</button>`
        : `<button class="btn btn-primary" data-act="mirror" ${on ? '' : 'disabled'}>${icon('play')}Ver pantalla</button>`}
    </div>
  </div>`);

  let banner = '';
  if (d.state === 'unauthorized') banner = ['orange', 'Autoriza la depuración en el teléfono', 'Desbloquea el teléfono y acepta el diálogo «¿Permitir depuración USB?». Si no aparece, desconecta y vuelve a conectar el cable, o revoca las autorizaciones en Opciones de desarrollador.'];
  else if (d.state === 'offline') banner = ['danger', 'El dispositivo no responde', 'Suele pasar cuando otro servidor adb tomó el dispositivo o el cable se desconectó. Revisa la vista Servidor ADB o reinicia el servidor.'];
  else if (!on) banner = ['orange', `Modo ${label}`, 'El dispositivo está conectado pero no en modo normal; la mayoría de acciones no están disponibles.'];
  patch($('#dv-banner'), banner ? `<div class="card banner"><span class="dot ${banner[0]}"></span><div class="col"><div class="card-title">${banner[1]}</div><div class="card-sub" style="line-height:1.5">${banner[2]}</div></div>
    ${d.state === 'offline' ? `<button class="btn btn-secondary sm" data-act="nav" data-view="adb">Ver servidor</button>` : ''}</div>` : '');

  if (on) {
    const bat = det?.battery;
    const batColor = bat == null ? 'green' : bat <= 15 ? 'danger' : bat <= 30 ? 'orange' : 'green';
    const conn = d.transport === 'wifi' ? d.serial : det?.ip ? `IP ${det.ip}` : d.transport === 'emulator' ? 'Local' : 'Sin Wi-Fi';
    const stat = (lab, val, sub, extra = '') => `<div class="card stat"><div class="section-label">${lab}</div><div class="stat-value">${val}</div>${extra}<div class="stat-sub mono">${sub}</div></div>`;
    patch($('#dv-stats'), `<div class="stats">
      ${stat('Sistema', det?.android ? `Android ${esc(det.android)}` : '—', esc([det?.sdk && `SDK ${det.sdk}`, det?.abi].filter(Boolean).join(' · ') || 'Leyendo…'))}
      ${stat('Pantalla', esc(det?.resolution || '—'), esc(det?.density ? `${det.density} dpi` : ' '))}
      ${stat('Batería', bat != null ? `${bat} %` : '—', det ? (det.charging ? 'Cargando' : 'Con batería') : ' ', bat != null ? `<div class="meter"><div style="width:${bat}%;background:var(--${batColor})"></div></div>` : '')}
      ${stat('Conexión', tlabel, esc(conn))}
    </div>`);
  } else {
    patch($('#dv-stats'), '');
  }

  // Opciones y consola solo tienen sentido con el dispositivo en línea.
  patch($('#dv-options'), on ? optionsCard(on) : '');
  const grid = $('#dv-options')?.parentElement;
  if (grid) grid.style.display = on ? '' : 'none';
}

function cap(s) { return s ? s.charAt(0).toUpperCase() + s.slice(1) : s; }

function optionsCard(on) {
  const o = S.snap.settings.scrcpy;
  const sel = (key, opts) => `<select class="select" data-opt="${key}" ${on ? '' : 'disabled'}>${opts.map(([v, l]) => `<option value="${v}" ${Number(o[key]) === v ? 'selected' : ''}>${l}</option>`).join('')}</select>`;
  const tog = (key, t, dsc) => `<label class="opt-row"><div class="grow"><span class="t">${t}</span><span class="d">${dsc}</span></div>
    <span class="toggle"><input type="checkbox" data-opt="${key}" ${o[key] ? 'checked' : ''}><span></span></span></label>`;
  const noScrcpy = !S.snap.scrcpyPath;
  return `<div class="card">
    <div class="card-head"><div class="grow col" style="gap:2px"><div class="card-title">Opciones de pantalla</div><div class="card-sub">Se aplican al abrir la pantalla con scrcpy</div></div></div>
    ${noScrcpy ? `<div class="banner" style="border-bottom:1px solid var(--line)"><span class="dot orange"></span><div class="col"><div class="card-title">scrcpy no está instalado</div><div class="card-sub">Descárgalo desde su página oficial en GitHub o indica su ruta en Ajustes.</div><div class="row" style="margin-top:8px">${toolButton('scrcpy', 'sm')}</div></div></div>` : ''}
    <div class="opt-grid">
      <div class="field"><label>Resolución máx.</label>${sel('maxSize', [[0, 'Original'], [2560, '2560 px'], [1920, '1920 px'], [1280, '1280 px'], [1024, '1024 px'], [800, '800 px']])}</div>
      <div class="field"><label>Bitrate</label>${sel('bitRate', [[2, '2 Mbps'], [4, '4 Mbps'], [8, '8 Mbps'], [16, '16 Mbps'], [24, '24 Mbps']])}</div>
      <div class="field"><label>FPS máx.</label>${sel('maxFps', [[0, 'Sin límite'], [30, '30 FPS'], [60, '60 FPS'], [90, '90 FPS'], [120, '120 FPS']])}</div>
    </div>
    <div class="opt-list">
      ${tog('stayAwake', 'Mantener despierto', 'El teléfono no se bloquea mientras está conectado')}
      ${tog('turnScreenOff', 'Apagar pantalla del teléfono', 'Se sigue viendo y controlando en el escritorio')}
      ${tog('showTouches', 'Mostrar toques', 'Dibuja los toques físicos en la pantalla')}
      ${tog('alwaysOnTop', 'Siempre encima', 'La ventana queda sobre las demás')}
      ${tog('noAudio', 'Sin audio', 'No reenviar el audio del teléfono (Android 11+)')}
    </div>
  </div>`;
}

function buildConsole(serial) {
  const c = S.consoles[serial] || (S.consoles[serial] = { html: '', history: [], idx: -1 });
  const el = $('#dv-console');
  el.innerHTML = `<div class="card">
    <div class="card-head"><div class="grow col" style="gap:2px"><div class="card-title">Consola</div><div class="card-sub">Ejecuta <span class="mono">adb shell</span> en el dispositivo · límite 20 s</div></div>
      <button class="icon-btn sm" data-act="console-clear" data-tip="Limpiar">${icon('trash', 16)}</button></div>
    <form class="console-form" id="console-form" autocomplete="off">
      <div class="prompt"><span class="mono">$</span><input class="mono" id="console-input" placeholder="getprop ro.build.version.release" spellcheck="false"></div>
      <button class="btn btn-secondary sm" type="submit">Ejecutar</button>
    </form>
    <pre class="console-out mono" id="console-out">${c.html || '<span class="faint" style="word-break:normal">Ejemplos: pm list packages -3 · dumpsys battery · wm size · input keyevent 26</span>'}</pre>
  </div>`;
}

/* ---- Pantallas (scrcpy) ---- */
function viewMirror(stage) {
  const s = S.snap;
  const sessions = s.sessions;
  const rows = sessions.map((x) => `<div class="list-row">
      <span class="dot green"></span>
      <div class="main"><div class="t">${esc(x.title)}</div>
        <div class="s mono">PID ${x.pid} · <span data-since="${x.started}">${fmtDur(Date.now() - x.started)}</span> · ${esc(x.summary)}</div>
        ${x.recordPath ? `<div class="s mono">${esc(x.recordPath)}</div>` : ''}</div>
      <button class="btn sm" data-act="mirror-focus" data-id="${x.id}">${icon('front', 16)}Traer al frente</button>
      <button class="btn btn-danger sm" data-act="mirror-stop" data-id="${x.id}">Cerrar</button>
    </div>`).join('');
  const ext = externalMirrors().map((x) => {
    const d = devices().find((dv) => dv.serial === x.serial);
    return `<div class="list-row">
      <span class="dot teal"></span>
      <div class="main"><div class="t">${esc(d ? deviceName(d) : x.serial || 'scrcpy')}</div>
        <div class="s mono">PID ${x.pid} · <span data-since="${x.started * 1000}">${fmtDur(Date.now() - x.started * 1000)}</span>${x.serial ? ` · ${esc(x.serial)}` : ''}</div></div>
      <button class="btn sm" data-act="ext-focus" data-pid="${x.pid}">${icon('front', 16)}Traer al frente</button>
      <button class="btn btn-danger sm" data-act="ext-close" data-pid="${x.pid}">Cerrar</button>
    </div>`;
  }).join('');
  const recs = (s.recordings || []).map((x) => {
    const d = devices().find((dv) => dv.serial === x.serial);
    const ext = x.path.split('.').pop().toUpperCase();
    return `<div class="list-row">
      <span class="dot danger pulse"></span>
      <div class="main"><div class="t">${esc(d ? deviceName(d) : x.serial)}</div>
        <div class="s mono"><span data-since="${x.started}">${fmtDur(Date.now() - x.started)}</span> · ${ext} · ${x.audio ? 'con audio' : 'sin audio'} · PID ${x.pid}</div>
        <div class="s mono">${esc(x.path)}</div></div>
      <button class="btn btn-danger sm" data-act="record-stop" data-serial="${esc(x.serial)}">${icon('stop', 16)}Detener</button>
    </div>`;
  }).join('');
  patch(stage, `<div class="view">
    <div class="page-head"><div class="grow col" style="gap:4px"><div class="page-title">Pantallas y grabaciones</div>
      <div class="page-sub">Cada pantalla es una ventana de scrcpy en el escritorio. Se controla con mouse y teclado; al cerrarla termina la sesión. Las grabaciones corren en segundo plano, sin ventana.</div></div></div>
    ${recs ? `<div class="card"><div class="card-head"><div class="card-title grow">Grabaciones en curso</div></div>${recs}</div>` : ''}
    ${s.scrcpyPath ? '' : `<div class="card banner"><span class="dot danger"></span><div class="col grow" style="gap:6px"><div class="card-title">No se encontró scrcpy</div>
      <div class="card-sub" style="line-height:1.6">Descárgalo desde su página oficial en GitHub, o instálalo por tu cuenta (<span class="code">winget install Genymobile.scrcpy</span>) y pulsa Volver a detectar.</div>
      <div class="row" style="margin-top:4px">${toolButton('scrcpy', 'sm')}<button class="btn btn-secondary sm" data-act="detect">${icon('search', 16)}Volver a detectar</button><button class="btn sm" data-act="pick" data-kind="scrcpy">${icon('folder', 16)}Elegir scrcpy.exe</button></div></div></div>`}
    <div class="card">${rows || `<div class="list-empty">No hay pantallas abiertas desde esta ventana. Elige un dispositivo y pulsa Ver pantalla.</div>`}</div>
    ${ext ? `<div class="card"><div class="card-head"><div class="grow col" style="gap:2px"><div class="card-title">Otras ventanas de scrcpy</div><div class="card-sub">Abiertas por otra ejecución de esta app o desde una terminal</div></div></div>${ext}</div>` : ''}
    <div class="card card-pad col" style="gap:8px">
      <div class="section-label">Cómo evita conflictos</div>
      <div class="card-sub" style="line-height:1.6">scrcpy trae su propio adb. Si se usara, reiniciaría el servidor de Android Studio cuando las versiones no coinciden. Esta app lo lanza apuntando al adb configurado:</div>
      <div class="mono xs muted selectable" style="line-height:1.8">ADB=${esc(s.adbPath || '—')}<br>ANDROID_ADB_SERVER_PORT=${s.server.port}<br>${esc(s.scrcpyPath || 'scrcpy.exe')}</div>
    </div>
  </div>`);
}

function fmtDur(ms) {
  const t = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(t / 3600), m = Math.floor((t % 3600) / 60), s = t % 60;
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/* ---- Servidor ADB ---- */
function viewAdb(stage) {
  const r = S.instances;
  const s = S.snap;
  if (!r) { patch(stage, `<div class="view"><div class="page-title">Servidor ADB</div><div class="card list-empty">Analizando procesos…</div></div>`); return; }
  const srv = s.server;
  const isolated = s.settings.serverPort !== 5037;
  const levelDot = { danger: 'danger', warning: 'orange', info: 'primary' };

  const statusText = srv.starting ? 'Iniciando servidor' : srv.running ? `Servidor en ejecución` : 'Servidor detenido';
  const statusMeta = [
    `127.0.0.1:${srv.port}`,
    srv.version != null && `protocolo v${srv.version}`,
    r.serverPid && `PID ${r.serverPid}`,
  ].filter(Boolean).join(' · ');

  const issues = r.issues.length
    ? r.issues.map((i) => `<div class="list-row"><span class="dot ${levelDot[i.level] || 'primary'}"></span>
        <div class="main"><div class="t">${esc(i.title)}</div><div class="d">${esc(i.detail)}</div></div>
        ${i.action ? `<button class="btn btn-secondary sm" data-act="issue" data-action="${esc(i.action)}">${esc(i.actionLabel || 'Aplicar')}</button>` : ''}</div>`).join('')
    : `<div class="list-row"><span class="dot green"></span><div class="main"><div class="t">Sin conflictos detectados</div><div class="d">El servidor y el cliente usan la misma versión de protocolo.</div></div></div>`;

  const procs = r.processes.length
    ? `<table class="table"><thead><tr><th>PID</th><th>Origen</th><th>Versión</th><th>Puerto</th><th>Ruta</th><th></th></tr></thead><tbody>${
      r.processes.map((p) => `<tr>
        <td class="mono nowrap">${p.pid}</td>
        <td class="nowrap">${esc(p.origin)} ${p.isConfigured ? '<span class="tag primary">Esta app</span>' : ''}</td>
        <td class="mono nowrap">${p.version ? `${esc(p.version.tools)} · v${p.version.protocol}` : '—'}</td>
        <td class="mono nowrap">${p.ports.length ? p.ports.join(', ') : `<span class="faint">${p.role === 'scrcpy' ? 'túnel scrcpy' : 'cliente'}</span>`}</td>
        <td class="mono path" title="${esc(p.path)}\n${esc(p.command)}">${esc(p.path || 'Sin acceso')}<div class="faint" style="overflow:hidden;text-overflow:ellipsis">${esc(p.command)}</div></td>
        <td class="nowrap"><button class="icon-btn sm" data-act="kill" data-pid="${p.pid}" data-tip="Terminar proceso">${icon('kill', 16)}</button></td>
      </tr>`).join('')}</tbody></table>`
    : `<div class="list-empty">No hay procesos adb en ejecución.</div>`;

  const bins = r.binaries.length
    ? `<table class="table"><thead><tr><th>Origen</th><th>Versión</th><th>Ruta</th><th></th></tr></thead><tbody>${
      r.binaries.map((b) => `<tr>
        <td class="nowrap">${esc(b.origin)}</td>
        <td class="mono nowrap">${b.version ? `${esc(b.version.tools)} · v${b.version.protocol}` : '—'}</td>
        <td class="mono path" title="${esc(b.path)}">${esc(b.path)}</td>
        <td class="nowrap">${b.isConfigured ? '<span class="tag primary">En uso</span>' : `<button class="btn sm" data-act="issue" data-action="use-adb:${esc(b.path)}">Usar este</button>`}</td>
      </tr>`).join('')}</tbody></table>`
    : `<div class="list-empty">No se encontró ningún adb.exe.</div>`;

  patch(stage, `<div class="view">
    <div class="page-head"><div class="grow col" style="gap:4px"><div class="page-title">Servidor ADB</div>
      <div class="page-sub">Android Studio levanta un servidor adb en el puerto 5037. Si otro programa usa un adb de versión distinta en el mismo puerto, ambos se reinician mutuamente y los dispositivos se caen.</div></div>
      <button class="icon-btn" data-act="refresh-instances" data-tip="Volver a analizar">${icon('refresh')}</button></div>

    <div class="card">
      <div class="card-head">
        <span class="dot ${srv.starting ? 'orange' : srv.running ? 'green' : 'danger'}"></span>
        <div class="grow col" style="gap:2px"><div class="card-title">${statusText}</div><div class="mono xs faint">${esc(statusMeta)}</div></div>
        ${srv.running
          ? `<button class="btn btn-secondary sm" data-act="server" data-action="restart">${icon('restart', 16)}Reiniciar</button>
             <button class="btn btn-danger sm" data-act="server" data-action="stop">Detener</button>`
          : `<button class="btn btn-primary sm" data-act="server" data-action="start" ${srv.starting ? 'disabled' : ''}>${icon('play', 16)}Iniciar servidor</button>`}
      </div>
      <div class="settings-row">
        <div class="lab"><b>Android Studio</b><span>${r.studioRunning ? 'Abierto' : 'Cerrado'}</span></div>
        <div class="row"><span class="dot ${r.studioRunning ? 'green' : ''}"></span><span class="small muted">${r.studioRunning ? 'Está en ejecución y comparte los dispositivos con este servidor.' : 'No está en ejecución.'}</span></div>
      </div>
      <div class="settings-row">
        <div class="lab"><b>Cliente adb</b><span>${r.clientVersion ? `${esc(r.clientVersion.tools)} · v${r.clientVersion.protocol}` : 'No encontrado'}</span></div>
        <div class="mono xs muted path selectable" style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap" title="${esc(r.clientPath || '')}">${esc(r.clientPath || 'Indica la ruta en Ajustes')}</div>
      </div>
    </div>

    <div class="card">
      <div class="card-head"><div class="grow col" style="gap:2px"><div class="card-title">Modo del servidor</div><div class="card-sub">Dónde escucha el servidor que usa esta app</div></div>
        <div class="seg"><button class="${isolated ? '' : 'on'}" data-act="mode" data-mode="shared">Compartido</button><button class="${isolated ? 'on' : ''}" data-act="mode" data-mode="isolated">Aislado</button></div></div>
      ${isolated
        ? `<div class="settings-row"><div class="lab"><b>Puerto propio</b><span>Distinto de 5037</span></div>
            <div class="row"><input class="input mono" id="port-input" style="width:120px" value="${s.settings.serverPort}" inputmode="numeric"><button class="btn btn-secondary sm" data-act="apply-port">Aplicar</button></div></div>
           <div class="settings-row"><div class="lab"><b>Al cerrar la app</b><span>Servidor aislado</span></div>
            <label class="row" style="cursor:pointer"><span class="toggle"><input type="checkbox" data-set="stopIsolatedOnExit" ${s.settings.stopIsolatedOnExit ? 'checked' : ''}><span></span></span><span class="small muted">Detener el servidor del puerto ${s.settings.serverPort}</span></label></div>
           <div class="banner" style="border-top:1px solid var(--line)"><span class="dot orange"></span><div class="card-sub" style="line-height:1.6">Un dispositivo USB solo puede ser atendido por un servidor a la vez. En modo aislado úsalo para dispositivos por Wi-Fi, o cuando Android Studio no esté usando ese teléfono.</div></div>`
        : `<div class="banner"><span class="dot green"></span><div class="card-sub" style="line-height:1.6">Esta app y Android Studio comparten el mismo servidor en el puerto 5037 usando el adb del SDK. Es la opción recomendada: los dos ven los mismos dispositivos y nadie reinicia el servidor del otro.</div></div>`}
    </div>

    <div class="card"><div class="card-head"><div class="card-title grow">Diagnóstico</div></div>${issues}</div>
    <div class="card"><div class="card-head"><div class="card-title grow">Procesos adb en ejecución</div><span class="mono xs faint">${r.processes.length}</span></div>${procs}</div>
    <div class="card"><div class="card-head"><div class="card-title grow">Binarios adb en el equipo</div><span class="mono xs faint">${r.binaries.length}</span></div>${bins}</div>
  </div>`);
}

/* ---- Registro ---- */
function viewLog(stage) {
  const rows = [...S.logs].reverse().map((l) => {
    const open = S.openLogs.has(l.id);
    return `<div class="log-row" data-act="log-toggle" data-id="${l.id}">
      <span class="mono faint">${esc(l.time)}</span>
      <span class="dot ${l.ok ? 'green' : l.code == null && ['iniciado', 'ok'].includes(l.output) ? 'primary' : 'danger'}"></span>
      <span class="mono c">${esc(l.command)}</span>
      <span class="mono faint">${l.code != null ? `código ${l.code} · ` : ''}${l.ms ? `${l.ms} ms` : ''}</span>
    </div>${open ? `<pre class="log-out mono">${esc(l.output || 'Sin salida')}</pre>` : ''}`;
  }).join('');
  patch(stage, `<div class="view">
    <div class="page-head"><div class="grow col" style="gap:4px"><div class="page-title">Registro</div>
      <div class="page-sub">Comandos de adb y scrcpy que ejecutó esta app. Pulsa una línea para ver su salida.</div></div>
      <button class="btn btn-secondary sm" data-act="log-clear" ${S.logs.length ? '' : 'disabled'}>Limpiar</button></div>
    <div class="card">${rows || '<div class="list-empty">Todavía no hay comandos.</div>'}</div>
  </div>`);
}

/* ---- Ajustes ---- */
function viewSettings(stage) {
  const s = S.snap.settings;
  const theme = s.theme || 'system';
  const pathRow = (key, label, sub, placeholder, resolved, kind) => `<div class="settings-row">
    <div class="lab"><b>${label}</b><span>${sub}</span></div>
    <div style="min-width:0"><div class="path-field">
      <input class="input mono" data-set="${key}" value="${esc(s[key])}" placeholder="${esc(placeholder)}" spellcheck="false">
      <button class="btn btn-secondary sm" data-act="pick" data-kind="${kind}">Examinar</button>
      ${kind === 'folder' ? `<button class="icon-btn sm" data-act="open-captures" data-tip="Abrir carpeta">${icon('folder', 16)}</button>` : `<button class="icon-btn sm" data-act="clear-path" data-key="${key}" data-tip="Detección automática">${icon('search', 16)}</button>`}
    </div><div class="hint mono" title="${esc(resolved || '')}">${resolved ? `En uso: ${esc(resolved)}` : `No encontrado${kind === 'folder' ? '' : ` · ${toolButton(kind, 'link')}`}`}</div></div>
  </div>`;
  patch(stage, `<div class="view">
    <div class="page-head"><div class="grow col" style="gap:4px"><div class="page-title">Ajustes</div><div class="page-sub">Los cambios se guardan al momento.</div></div></div>
    <div class="card">
      <div class="card-head"><div class="card-title">Herramientas</div></div>
      ${pathRow('adbPath', 'adb.exe', 'Vacío = SDK de Android', 'Automático (SDK de Android)', S.snap.adbPath, 'adb')}
      ${pathRow('scrcpyPath', 'scrcpy.exe', 'Vacío = detección automática', 'Automático (PATH, WinGet, Scoop)', S.snap.scrcpyPath, 'scrcpy')}
      ${pathRow('captureDir', 'Capturas', 'Imágenes y grabaciones', 'Imágenes\\Android Device Manager', S.captureDir, 'folder')}
    </div>
    <div class="card">
      <div class="card-head"><div class="card-title">Servidor</div></div>
      <div class="settings-row"><div class="lab"><b>Al abrir</b><span>Arranque automático</span></div>
        <label class="row" style="cursor:pointer"><span class="toggle"><input type="checkbox" data-set="autoStartServer" ${s.autoStartServer ? 'checked' : ''}><span></span></span><span class="small muted">Iniciar el servidor adb si no está en ejecución</span></label></div>
    </div>
    <div class="card">
      <div class="card-head"><div class="card-title">Apariencia</div></div>
      <div class="settings-row"><div class="lab"><b>Tema</b><span>Claro u oscuro</span></div>
        <div><div class="seg">${[['system', 'Sistema'], ['light', 'Claro'], ['dark', 'Oscuro']].map(([v, l]) => `<button class="${theme === v ? 'on' : ''}" data-act="theme" data-theme="${v}">${l}</button>`).join('')}</div></div></div>
    </div>
  </div>`);
}

/* ------------------------------------------------------------------ */
/* Render general                                                      */
/* ------------------------------------------------------------------ */
function render() {
  if (!S.snap) return;
  const list = devices();
  if (!list.some((d) => d.serial === S.selected)) {
    S.selected = (list.find(isOnline) || list[0])?.serial ?? null;
  }
  applyTheme();
  renderChrome();
  renderSidebar();
  renderStage();
}

/* ------------------------------------------------------------------ */
/* Capas: menús, popovers, tooltips, toasts                             */
/* ------------------------------------------------------------------ */
const layer = $('#layer');
let openLayer = null;

function closeLayer() {
  if (openLayer) { openLayer.el.remove(); openLayer.resolve?.(false); openLayer = null; }
}

function placeNear(el, anchor, align = 'end') {
  layer.appendChild(el);
  const a = anchor.getBoundingClientRect();
  const w = el.offsetWidth, h = el.offsetHeight;
  let left = align === 'start' ? a.left : a.right - w;
  let top = a.bottom + 6;
  if (top + h > innerHeight - 8) top = Math.max(8, a.top - h - 6);
  left = Math.min(Math.max(8, left), innerWidth - w - 8);
  el.style.left = `${left}px`;
  el.style.top = `${top}px`;
}

function openMenu(anchor, title, items, align = 'start') {
  closeLayer();
  const el = document.createElement('div');
  el.className = 'menu';
  el.innerHTML = (title ? `<div class="section-label">${esc(title)}</div>` : '') + items.map((it, i) =>
    it === '-' ? '<div class="menu-sep"></div>'
      : `<button class="menu-item ${it.danger ? 'danger' : ''}" data-i="${i}">${it.icon ? icon(it.icon, 16) : ''}<span>${esc(it.label)}</span>${it.kbd ? `<span class="kbd mono">${esc(it.kbd)}</span>` : ''}</button>`).join('');
  el.addEventListener('click', (e) => {
    const b = e.target.closest('[data-i]');
    if (!b) return;
    const it = items[Number(b.dataset.i)];
    closeLayer();
    it.run();
  });
  placeNear(el, anchor, align);
  openLayer = { el, anchor };
}

/** Confirmación en popover para toda acción destructiva o de estado. */
function confirmPop(anchor, { title, text, ok = 'Confirmar', danger = true }) {
  closeLayer();
  return new Promise((resolve) => {
    const el = document.createElement('div');
    el.className = 'popover';
    el.innerHTML = `<div class="t">${esc(title)}</div>${text ? `<div class="d">${esc(text)}</div>` : ''}
      <div class="actions"><button class="btn btn-secondary" data-r="0">Cancelar</button><button class="btn ${danger ? 'btn-danger' : 'btn-primary'}" data-r="1">${esc(ok)}</button></div>`;
    el.addEventListener('click', (e) => {
      const b = e.target.closest('[data-r]');
      if (!b) return;
      const yes = b.dataset.r === '1';
      openLayer.resolve = null;
      closeLayer();
      resolve(yes);
    });
    placeNear(el, anchor);
    openLayer = { el, anchor, resolve };
    el.querySelector('[data-r="1"]').focus();
  });
}

/** Opciones de grabación; «Grabar» es la confirmación de la acción de estado. */
function recordPop(anchor, d) {
  closeLayer();
  const o = S.snap.settings.record;
  const opt = (v, l, cur) => `<option value="${v}" ${Number(cur) === v ? 'selected' : ''}>${l}</option>`;
  const el = document.createElement('div');
  el.className = 'popover';
  el.style.width = '300px';
  el.innerHTML = `
    <div class="t">Grabar pantalla</div>
    <div class="d">Graba ${esc(deviceName(d))} en segundo plano, sin abrir ventana. El video se guarda en la carpeta de capturas.</div>
    <div class="row" style="gap:8px">
      <div class="field grow"><label>Resolución</label><select class="select" data-rec="maxSize">${[[0, 'Original'], [1920, '1920 px'], [1280, '1280 px'], [1024, '1024 px']].map(([v, l]) => opt(v, l, o.maxSize)).join('')}</select></div>
      <div class="field grow"><label>Bitrate</label><select class="select" data-rec="bitRate">${[[4, '4 Mbps'], [8, '8 Mbps'], [16, '16 Mbps'], [24, '24 Mbps']].map(([v, l]) => opt(v, l, o.bitRate)).join('')}</select></div>
    </div>
    <div class="row" style="justify-content:space-between">
      <span class="small muted" style="font-weight:600">Formato</span>
      <div class="seg">${[['mp4', 'MP4'], ['mkv', 'MKV']].map(([v, l]) => `<button type="button" class="${o.format === v ? 'on' : ''}" data-rec-format="${v}">${l}</button>`).join('')}</div>
    </div>
    <label class="row" style="cursor:pointer;justify-content:space-between">
      <span class="col" style="gap:0"><span class="small" style="font-weight:600">Audio del teléfono</span><span class="xs faint">Android 11 o superior</span></span>
      <span class="toggle"><input type="checkbox" data-rec="audio" ${o.audio ? 'checked' : ''}><span></span></span>
    </label>
    <div class="actions"><button class="btn btn-secondary" data-r="0">Cancelar</button><button class="btn btn-primary" data-r="1">${icon('record', 16)}Grabar</button></div>`;
  const current = () => S.snap.settings.record;
  el.addEventListener('change', (e) => {
    const k = e.target.dataset.rec;
    if (!k) return;
    const v = e.target.type === 'checkbox' ? e.target.checked : Number(e.target.value);
    saveSettings((s) => { s.record[k] = v; });
  });
  el.addEventListener('click', async (e) => {
    const f = e.target.closest('[data-rec-format]');
    if (f) {
      el.querySelectorAll('[data-rec-format]').forEach((b) => b.classList.toggle('on', b === f));
      saveSettings((s) => { s.record.format = f.dataset.recFormat; });
      return;
    }
    const b = e.target.closest('[data-r]');
    if (!b) return;
    closeLayer();
    if (b.dataset.r !== '1') return;
    run('record', async () => {
      await invoke('start_recording', { serial: d.serial, options: current() });
      toast('info', `Grabando ${deviceName(d)}`);
    });
  });
  placeNear(el, anchor, 'start');
  openLayer = { el, anchor };
  el.querySelector('[data-r="1"]').focus();
}

function connectPop(anchor) {
  closeLayer();
  const el = document.createElement('div');
  el.className = 'popover';
  el.style.width = '300px';
  el.innerHTML = `
    <div class="t">Conectar por IP</div>
    <div class="d">Dispositivos con depuración por Wi-Fi o <span class="mono">adb tcpip</span>. Si omites el puerto se usa 5555.</div>
    <form class="row" data-f="connect"><input class="input mono" name="addr" placeholder="192.168.1.20:5555" spellcheck="false"><button class="btn btn-primary sm" type="submit">Conectar</button></form>
    <div class="pop-sep"></div>
    <div class="t">Emparejar · Android 11+</div>
    <div class="d">En el teléfono: Depuración inalámbrica → Vincular con código. Usa la IP y el puerto de vinculación que muestra.</div>
    <form class="col" style="gap:8px" data-f="pair">
      <input class="input mono" name="addr" placeholder="192.168.1.20:37123" spellcheck="false">
      <div class="row"><input class="input mono" name="code" placeholder="Código de 6 dígitos" maxlength="6" inputmode="numeric"><button class="btn btn-secondary sm" type="submit">Emparejar</button></div>
    </form>`;
  el.addEventListener('submit', async (e) => {
    e.preventDefault();
    const f = e.target;
    const btn = f.querySelector('[type=submit]');
    btn.disabled = true;
    try {
      if (f.dataset.f === 'connect') {
        const msg = await invoke('connect', { addr: f.addr.value });
        toast('success', msg);
        closeLayer();
      } else {
        const msg = await invoke('pair', { addr: f.addr.value, code: f.code.value });
        toast('success', msg);
        const other = el.querySelector('[data-f=connect] input');
        other.value = f.addr.value.split(':')[0] + ':';
        other.focus();
      }
    } catch (err) { toast('danger', err); }
    btn.disabled = false;
  });
  placeNear(el, anchor);
  openLayer = { el, anchor };
  el.querySelector('input').focus();
}

document.addEventListener('mousedown', (e) => {
  if (openLayer && !openLayer.el.contains(e.target) && !openLayer.anchor.contains(e.target)) closeLayer();
});
document.addEventListener('keydown', (e) => { if (e.key === 'Escape') closeLayer(); });
window.addEventListener('resize', closeLayer);

// Tooltips a los 400 ms.
let tipTimer = null, tipEl = null;
document.addEventListener('mouseover', (e) => {
  const t = e.target.closest('[data-tip]');
  clearTimeout(tipTimer);
  if (tipEl) { tipEl.remove(); tipEl = null; }
  if (!t) return;
  // En la barra con etiquetas visibles no hace falta tooltip.
  const lbl = t.querySelector('.lbl');
  if (lbl && lbl.offsetParent !== null) return;
  tipTimer = setTimeout(() => {
    if (!document.body.contains(t)) return;
    tipEl = document.createElement('div');
    tipEl.className = 'tooltip';
    tipEl.textContent = t.dataset.tip;
    layer.appendChild(tipEl);
    const a = t.getBoundingClientRect();
    if (t.dataset.tipSide === 'left') {
      tipEl.style.left = `${a.left - tipEl.offsetWidth - 8}px`;
      tipEl.style.top = `${a.top + (a.height - tipEl.offsetHeight) / 2}px`;
    } else {
      tipEl.style.left = `${Math.min(innerWidth - tipEl.offsetWidth - 8, Math.max(8, a.left + (a.width - tipEl.offsetWidth) / 2))}px`;
      tipEl.style.top = `${a.bottom + 6}px`;
    }
  }, 400);
});
document.addEventListener('mousedown', () => { clearTimeout(tipTimer); if (tipEl) { tipEl.remove(); tipEl = null; } });

function toast(level, text, action) {
  const dot = { success: 'green', danger: 'danger', warning: 'orange', info: 'primary' }[level] || 'primary';
  const el = document.createElement('div');
  el.className = 'toast';
  el.innerHTML = `<span class="dot ${dot}"></span><div class="txt">${esc(text)}</div>${action ? `<button class="btn">${esc(action.label)}</button>` : ''}`;
  if (action) el.querySelector('.btn').addEventListener('click', () => { action.run(); dismiss(); });
  const box = $('#toasts');
  box.appendChild(el);
  // Solo se recortan los avisos simples; las tarjetas de tareas en curso se quedan.
  const plain = [...box.children].filter((c) => !c.classList.contains('task'));
  while (plain.length > 4) plain.shift().remove();
  let timer = setTimeout(dismiss, level === 'danger' ? 7000 : 4000);
  el.addEventListener('mouseenter', () => clearTimeout(timer));
  el.addEventListener('mouseleave', () => { timer = setTimeout(dismiss, 2500); });
  function dismiss() { el.classList.add('out'); setTimeout(() => el.remove(), 220); }
}

/* ------------------------------------------------------------------ */
/* Instalación de APK: tarjeta de progreso persistente                  */
/* ------------------------------------------------------------------ */
const mb = (b) => (b / 1048576).toLocaleString('es', { minimumFractionDigits: 1, maximumFractionDigits: 1 });
const pctOf = (x) => (x.total ? Math.min(100, Math.floor((x.sent / x.total) * 100)) : 0);
const secs = (ms) => `${Math.max(0, Math.round(ms / 1000))} s`;

function installLabel(x) {
  if (x.phase === 'uninstall') return 'Desinstalando…';
  if (x.phase === 'copy') return `Instalando ${pctOf(x)} %`;
  return 'Instalando…';
}

function onInstallEvent(x) {
  const prev = S.installs.get(x.id);
  x.startedAt = prev?.startedAt ?? Date.now() - x.elapsedMs;
  S.installs.set(x.id, x);
  renderTask(x);
  renderChrome();
  if (x.phase === 'done') setTimeout(() => dismissTask(x.id), 6000);
}

function dismissTask(id) {
  S.installs.delete(id);
  const el = document.getElementById(`task-${id}`);
  if (el) { el.classList.add('out'); setTimeout(() => el.remove(), 220); }
  renderChrome();
}

function renderTask(x) {
  let el = document.getElementById(`task-${x.id}`);
  if (!el) {
    el = document.createElement('div');
    el.id = `task-${x.id}`;
    el.className = 'toast task';
    el.setAttribute('role', 'status');
    $('#toasts').appendChild(el);
  }
  const f = x.failure;
  const active = INSTALL_ACTIVE.includes(x.phase);
  const pct = pctOf(x);
  const head = {
    uninstall: ['Desinstalando la versión anterior', `${x.device} · ${x.file}`],
    copy: [`Instalando ${x.file}`, `${x.device} · Copiando ${mb(x.sent)} de ${mb(x.total)} MB`],
    install: [`Instalando ${x.file}`, `${x.device} · Verificando e instalando`],
    done: [`${x.file} instalado`, `${x.device} · ${secs(x.elapsedMs)}`],
    error: [`No se pudo instalar ${x.file}`, x.device],
  }[x.phase];
  const lead = active ? `<span class="task-ico">${spinner(16)}</span>`
    : `<span class="dot ${x.phase === 'done' ? 'green' : 'danger'}"></span>`;
  const bar = x.phase === 'error' ? ''
    : `<div class="task-bar ${x.phase === 'install' || x.phase === 'uninstall' ? 'indeterminate' : ''} ${x.phase === 'done' ? 'ok' : ''}"><div style="width:${x.phase === 'done' ? 100 : pct}%"></div></div>`;
  const fixes = [];
  if (f?.fix === 'uninstall' && f.package) fixes.push(`<button class="btn btn-danger sm" data-act="install-fix" data-id="${x.id}" data-mode="uninstall:${esc(f.package)}">Desinstalar y reintentar</button>`);
  if (f?.fix === 'downgrade') fixes.push(`<button class="btn btn-secondary sm" data-act="install-fix" data-id="${x.id}" data-mode="downgrade">Instalar de todos modos</button>`);
  if (f && !f.fix) fixes.push(`<button class="btn btn-secondary sm" data-act="install-fix" data-id="${x.id}" data-mode="">Reintentar</button>`);

  el.innerHTML = `
    <div class="task-row">
      ${lead}
      <div class="txt"><div class="task-title">${esc(head[0])}</div>
        <div class="task-sub mono">${esc(head[1])}${active ? ` · <span data-elapsed="${x.startedAt}">${secs(Date.now() - x.startedAt)}</span>` : ''}</div></div>
      ${x.phase === 'copy' ? `<span class="task-pct mono">${pct} %</span>` : ''}
      ${active ? '' : `<button class="icon-btn sm" data-act="task-close" data-id="${x.id}" aria-label="Cerrar">${icon('close', 14)}</button>`}
    </div>
    ${bar}
    ${f ? `<div class="task-msg">${esc(f.message)}${f.code ? `<div class="mono xs faint selectable" style="margin-top:4px">${esc(f.code)}${f.package ? ` · ${esc(f.package)}` : ''}</div>` : ''}</div>
      <div class="task-actions"><button class="btn sm" data-act="nav" data-view="log">Ver registro</button>${fixes.join('')}</div>` : ''}`;
}

/* ------------------------------------------------------------------ */
/* Descarga de adb y scrcpy                                             */
/* ------------------------------------------------------------------ */
const TOOL_ACTIVE = ['resolve', 'download', 'extract'];
const TOOL_NAME = { adb: 'adb', scrcpy: 'scrcpy' };
const toolActive = (tool) => TOOL_ACTIVE.includes(S.tools.get(tool)?.phase);
const toolPct = (x) => (x.total ? Math.min(100, Math.floor((x.received / x.total) * 100)) : 0);

/** Botón «Descargar e instalar»; mientras descarga muestra el avance. `variant`: '' | 'sm' | 'link'. */
function toolButton(tool, variant = '') {
  const x = S.tools.get(tool);
  const active = toolActive(tool);
  const text = !active ? `Descargar e instalar ${TOOL_NAME[tool]}`
    : x.phase === 'download' && x.total ? `Descargando ${toolPct(x)} %`
      : x.phase === 'extract' ? 'Descomprimiendo…' : 'Descargando…';
  if (variant === 'link') {
    return `<button class="link-btn" data-act="install-tool" data-tool="${tool}" ${active ? 'disabled' : ''}>${esc(text)}</button>`;
  }
  return `<button class="btn btn-primary ${variant}" data-act="install-tool" data-tool="${tool}" ${active ? 'disabled' : ''}>${active ? spinner(16) : icon('install', variant === 'sm' ? 16 : 18)}${esc(text)}</button>`;
}

function onToolEvent(x) {
  const prev = S.tools.get(x.tool);
  x.startedAt = prev && TOOL_ACTIVE.includes(prev.phase) ? prev.startedAt : Date.now() - x.elapsedMs;
  S.tools.set(x.tool, x);
  renderToolTask(x);
  renderStage();
  if (x.phase === 'done') {
    refreshInstances();
    setTimeout(() => { if (S.tools.get(x.tool) === x) dismissToolTask(x.tool); }, 8000);
  }
}

function dismissToolTask(tool) {
  if (toolActive(tool)) return;
  S.tools.delete(tool);
  const el = document.getElementById(`tool-${tool}`);
  if (el) { el.classList.add('out'); setTimeout(() => el.remove(), 220); }
  renderStage();
}

function renderToolTask(x) {
  let el = document.getElementById(`tool-${x.tool}`);
  if (!el) {
    el = document.createElement('div');
    el.id = `tool-${x.tool}`;
    el.className = 'toast task';
    el.setAttribute('role', 'status');
    $('#toasts').appendChild(el);
  }
  const active = TOOL_ACTIVE.includes(x.phase);
  const pct = toolPct(x);
  const head = {
    resolve: [`Descargando ${x.label}`, 'Buscando la versión más reciente'],
    download: [`Descargando ${x.label}`, x.total ? `${mb(x.received)} de ${mb(x.total)} MB` : `${mb(x.received)} MB`],
    extract: [`Instalando ${x.label}`, 'Descomprimiendo'],
    done: [`${x.label} instalado`, x.version && x.version !== 'más reciente' ? `Versión ${x.version} · ${secs(x.elapsedMs)}` : secs(x.elapsedMs)],
    error: [`No se pudo descargar ${x.label}`, ''],
  }[x.phase];
  const lead = active ? `<span class="task-ico">${spinner(16)}</span>`
    : `<span class="dot ${x.phase === 'done' ? 'green' : 'danger'}"></span>`;
  const bar = x.phase === 'error' ? ''
    : `<div class="task-bar ${x.phase === 'download' && x.total ? '' : active ? 'indeterminate' : ''} ${x.phase === 'done' ? 'ok' : ''}"><div style="width:${x.phase === 'done' ? 100 : pct}%"></div></div>`;
  el.innerHTML = `
    <div class="task-row">
      ${lead}
      <div class="txt"><div class="task-title">${esc(head[0])}</div>
        <div class="task-sub mono">${esc(head[1])}${active ? `${head[1] ? ' · ' : ''}<span data-elapsed="${x.startedAt}">${secs(Date.now() - x.startedAt)}</span>` : ''}</div></div>
      ${x.phase === 'download' && x.total ? `<span class="task-pct mono">${pct} %</span>` : ''}
      ${active ? '' : `<button class="icon-btn sm" data-act="tool-close" data-tool="${x.tool}" aria-label="Cerrar">${icon('close', 14)}</button>`}
    </div>
    ${bar}
    ${x.phase === 'done' && x.path ? `<div class="task-msg mono xs faint selectable" style="overflow-wrap:anywhere">${esc(x.path)}</div>` : ''}
    ${x.phase === 'error' ? `<div class="task-msg">${esc(x.message)}</div>
      <div class="task-actions"><button class="btn sm" data-act="nav" data-view="log">Ver registro</button><button class="btn btn-secondary sm" data-act="install-tool" data-tool="${x.tool}">Reintentar</button></div>` : ''}`;
}

/* ------------------------------------------------------------------ */
/* Acciones                                                            */
/* ------------------------------------------------------------------ */
async function saveSettings(mutate) {
  const next = structuredClone(S.snap.settings);
  mutate(next);
  try { await invoke('save_settings', { settings: next }); }
  catch (err) { toast('danger', err); }
}

async function run(key, fn) {
  if (S.busy.has(key)) return;
  S.busy.add(key);
  renderChrome();
  try { await fn(); }
  catch (err) { toast('danger', String(err)); }
  finally { S.busy.delete(key); renderChrome(); }
}

async function refreshInstances() {
  try {
    S.instances = await invoke('get_instances');
    renderChrome();
    renderSidebar();
    if (['adb', 'mirror', 'device'].includes(S.view)) renderStage();
  } catch (err) { console.error(err); }
}

async function refreshCaptureDir() {
  try { S.captureDir = await invoke('capture_dir'); } catch { /* sin importancia */ }
}

const actions = {
  'win-min': () => appWindow.minimize(),
  'win-max': () => appWindow.toggleMaximize(),
  'win-close': () => appWindow.close(),

  nav: (el) => {
    S.view = el.dataset.view;
    if (S.view === 'adb' || S.view === 'mirror') refreshInstances();
    if (S.view === 'log') invoke('get_logs').then((l) => { S.logs = l; if (S.view === 'log') renderStage(); });
    if (S.view === 'settings') refreshCaptureDir().then(render);
    render();
  },
  select: (el) => {
    S.selected = el.dataset.serial;
    if (S.view !== 'device') S.view = 'device';
    render();
  },

  mirror: () => {
    const d = selectedDevice();
    if (!d) return;
    if (d.session) return invoke('focus_mirror', { id: d.session });
    const ext = externalFor(d.serial);
    if (ext) return invoke('external_mirror', { pid: ext.pid, action: 'focus' }).catch((e) => toast('danger', e));
    return run('mirror', async () => {
      const msg = await invoke('start_mirror', { serial: d.serial, options: S.snap.settings.scrcpy });
      toast('info', `${msg} · ${deviceName(d)}`);
    });
  },
  'mirror-focus': (el) => invoke('focus_mirror', { id: Number(el.dataset.id) }),
  'ext-focus': (el) => invoke('external_mirror', { pid: Number(el.dataset.pid), action: 'focus' }).catch((e) => toast('danger', e)),
  'ext-close': async (el) => {
    const ok = await confirmPop(el, { title: 'Cerrar pantalla', text: 'Se cerrará esa ventana de scrcpy.', ok: 'Cerrar' });
    if (!ok) return;
    try { await invoke('external_mirror', { pid: Number(el.dataset.pid), action: 'close' }); }
    catch (err) { toast('danger', err); }
    setTimeout(refreshInstances, 1200);
  },
  'mirror-stop': async (el) => {
    const ok = await confirmPop(el, { title: 'Cerrar pantalla', text: 'Se cerrará la ventana de scrcpy de este dispositivo.', ok: 'Cerrar' });
    if (ok) invoke('stop_mirror', { id: Number(el.dataset.id) }).catch((e) => toast('danger', e));
  },

  screenshot: () => {
    const d = selectedDevice();
    return run('shot', async () => {
      const path = await invoke('screenshot', { serial: d.serial });
      toast('success', `Captura guardada · ${path.split('\\').pop()}`, { label: 'Mostrar', run: () => invoke('open_path', { path }) });
    });
  },
  'record-pop': (el) => { const d = selectedDevice(); if (isOnline(d)) recordPop(el, d); },
  'record-stop': async (el) => {
    const serial = el.dataset.serial || selectedDevice()?.serial;
    const rec = recordingOf(serial);
    if (!rec) return;
    const ok = await confirmPop(el, { title: 'Detener grabación', text: `Se guardará ${rec.path.split('\\').pop()}.`, ok: 'Detener' });
    if (ok) invoke('stop_recording', { serial }).catch((e) => toast('danger', e));
  },
  // El progreso y el resultado llegan por eventos `install` (tarjeta persistente).
  install: () => {
    const d = selectedDevice();
    return run('install', () => invoke('install_apk', { serial: d.serial }));
  },
  'install-fix': async (el) => {
    const x = S.installs.get(Number(el.dataset.id));
    if (!x) return;
    const mode = el.dataset.mode;
    if (mode.startsWith('uninstall:')) {
      const ok = await confirmPop(el, {
        title: 'Desinstalar y reintentar',
        text: `Se desinstalará ${mode.slice(10)} de ${x.device} y se borrarán sus datos. Después se instalará ${x.file}.`,
        ok: 'Desinstalar',
      });
      if (!ok) return;
    }
    dismissTask(x.id);
    invoke('install_apk_path', { serial: x.serial, path: x.path, mode }).catch((e) => toast('danger', e));
  },
  'task-close': (el) => dismissTask(Number(el.dataset.id)),
  'install-tool': (el) => {
    const tool = el.dataset.tool;
    if (toolActive(tool)) return;
    invoke('install_tool', { tool }).catch((e) => toast('danger', e));
  },
  'tool-close': (el) => dismissToolTask(el.dataset.tool),
  wifi: async (el) => {
    const d = selectedDevice();
    const ok = await confirmPop(el, { title: 'Activar depuración por Wi-Fi', text: 'Se ejecuta adb tcpip 5555 y se conecta a la IP del teléfono. Después puedes desconectar el cable.', ok: 'Activar', danger: false });
    if (ok) run('wifi', async () => toast('success', await invoke('enable_wifi', { serial: d.serial })));
  },
  'device-menu': (el) => {
    const d = selectedDevice();
    if (!d) return;
    const on = isOnline(d);
    const reboot = (mode, label) => async () => {
      const ok = await confirmPop(el, { title: label, text: `${deviceName(d)} se reiniciará${mode ? ` en modo ${mode}` : ''}.`, ok: 'Reiniciar' });
      if (ok) invoke('reboot', { serial: d.serial, mode }).then(() => toast('info', `Reiniciando ${deviceName(d)}`)).catch((e) => toast('danger', e));
    };
    const items = [];
    if (on) {
      items.push({ label: 'Reiniciar', icon: 'restart', run: reboot('', 'Reiniciar dispositivo') });
      items.push({ label: 'Reiniciar en recovery', icon: 'restart', run: reboot('recovery', 'Reiniciar en recovery') });
      items.push({ label: 'Reiniciar en bootloader', icon: 'restart', run: reboot('bootloader', 'Reiniciar en bootloader') });
      items.push('-');
      items.push({ label: 'Actualizar datos', icon: 'refresh', run: () => invoke('refresh_device', { serial: d.serial }) });
      items.push({ label: 'Copiar número de serie', icon: 'copy', run: () => navigator.clipboard.writeText(d.serial).then(() => toast('info', 'Número de serie copiado')) });
    }
    if (d.transport === 'wifi') {
      if (items.length) items.push('-');
      items.push({
        label: 'Desconectar', icon: 'power', danger: true, run: async () => {
          const ok = await confirmPop(el, { title: 'Desconectar', text: `Se cerrará la conexión con ${d.serial}.`, ok: 'Desconectar' });
          if (ok) invoke('disconnect', { serial: d.serial }).catch((e) => toast('danger', e));
        },
      });
    }
    if (!items.length) items.push({ label: 'Copiar número de serie', icon: 'copy', run: () => navigator.clipboard.writeText(d.serial) });
    openMenu(el, 'Dispositivo', items);
  },
  'connect-pop': (el) => connectPop(el),
  'refresh-device': () => { const d = selectedDevice(); if (d) invoke('refresh_device', { serial: d.serial }); },
  'console-clear': () => {
    const d = selectedDevice();
    if (!d) return;
    S.consoles[d.serial].html = '';
    $('#console-out').innerHTML = '';
  },

  server: async (el) => {
    const action = el.dataset.action;
    if (action === 'stop') {
      const ok = await confirmPop(el, {
        title: 'Detener servidor adb',
        text: S.instances?.studioRunning && S.snap.settings.serverPort === 5037
          ? 'Android Studio también usa este servidor: perderá los dispositivos hasta que alguien lo vuelva a iniciar.'
          : 'Se desconectarán todos los dispositivos de este servidor.',
        ok: 'Detener',
      });
      if (!ok) return;
    }
    if (action === 'restart') {
      const ok = await confirmPop(el, { title: 'Reiniciar servidor adb', text: 'Los dispositivos se desconectan un momento. Las pantallas abiertas se cerrarán.', ok: 'Reiniciar' });
      if (!ok) return;
    }
    run('server', async () => {
      await invoke('server_action', { action });
      toast('success', { start: 'Servidor iniciado', stop: 'Servidor detenido', restart: 'Servidor reiniciado' }[action]);
      refreshInstances();
    });
  },
  'refresh-instances': () => refreshInstances(),
  kill: async (el) => {
    const pid = Number(el.dataset.pid);
    const ok = await confirmPop(el, { title: `Terminar adb · PID ${pid}`, text: 'Si es el servidor, los dispositivos se desconectarán y Android Studio también los perderá.', ok: 'Terminar' });
    if (!ok) return;
    try { await invoke('kill_process', { pid }); toast('success', `Proceso ${pid} terminado`); }
    catch (err) { toast('danger', err); }
    refreshInstances();
  },
  issue: async (el) => {
    const a = el.dataset.action;
    try {
      if (a.startsWith('use-adb:')) {
        await invoke('use_adb', { path: a.slice(8) });
        toast('success', 'adb actualizado');
      } else if (a.startsWith('kill:')) {
        el.dataset.pid = a.slice(5);
        return actions.kill(el);
      } else if (a.startsWith('install-tool:')) {
        el.dataset.tool = a.slice(13);
        return actions['install-tool'](el);
      } else if (a === 'shared-port') {
        await invoke('set_port', { port: 5037 });
        toast('success', 'Modo compartido en el puerto 5037');
      }
    } catch (err) { toast('danger', err); }
    refreshInstances();
  },
  mode: async (el) => {
    const want = el.dataset.mode;
    const isolated = S.snap.settings.serverPort !== 5037;
    if ((want === 'isolated') === isolated) return;
    const port = want === 'isolated' ? 5038 : 5037;
    const ok = await confirmPop(el, {
      title: want === 'isolated' ? 'Pasar a servidor aislado' : 'Volver a compartido',
      text: want === 'isolated'
        ? 'Esta app usará su propio servidor en el puerto 5038. Los dispositivos USB que ya atiende Android Studio seguirán en el 5037.'
        : 'Esta app volverá a usar el servidor del puerto 5037, el mismo que Android Studio.',
      ok: 'Cambiar', danger: false,
    });
    if (!ok) return;
    try { await invoke('set_port', { port }); } catch (err) { toast('danger', err); }
    refreshInstances();
  },
  'apply-port': async () => {
    const port = Number($('#port-input').value);
    if (!Number.isInteger(port) || port < 1024 || port > 65535 || port === 5037) return toast('warning', 'Usa un puerto entre 1024 y 65535, distinto de 5037');
    try { await invoke('set_port', { port }); toast('success', `Servidor aislado en el puerto ${port}`); } catch (err) { toast('danger', err); }
    refreshInstances();
  },

  'log-toggle': (el) => {
    const id = Number(el.dataset.id);
    S.openLogs.has(id) ? S.openLogs.delete(id) : S.openLogs.add(id);
    renderStage();
  },
  'log-clear': async () => { await invoke('clear_logs'); S.logs = []; S.openLogs.clear(); renderStage(); },

  pick: async (el) => {
    const kind = el.dataset.kind;
    const path = await invoke('pick_path', { kind }).catch((e) => { toast('danger', e); return null; });
    if (!path) return;
    const key = { adb: 'adbPath', scrcpy: 'scrcpyPath', folder: 'captureDir' }[kind];
    await saveSettings((s) => { s[key] = path; });
    if (kind === 'folder') await refreshCaptureDir();
    if (kind === 'adb') refreshInstances();
    render();
  },
  'clear-path': async (el) => {
    await saveSettings((s) => { s[el.dataset.key] = ''; });
    toast('info', 'Se usará la detección automática');
  },
  detect: async () => {
    const found = await invoke('detect_paths');
    await saveSettings((s) => { s.scrcpyPath = ''; });
    toast(found.scrcpy ? 'success' : 'warning', found.scrcpy ? `scrcpy encontrado · ${found.scrcpy}` : 'No se encontró scrcpy');
  },
  'open-captures': () => invoke('open_path', { path: S.captureDir || '' }).catch((e) => toast('danger', e)),
  theme: (el) => saveSettings((s) => { s.theme = el.dataset.theme; }),
};

document.addEventListener('click', (e) => {
  const el = e.target.closest('[data-act]');
  if (!el || el.disabled) return;
  const fn = actions[el.dataset.act];
  if (fn) { e.preventDefault(); fn(el); }
});

// Opciones de scrcpy y ajustes simples.
document.addEventListener('change', (e) => {
  const t = e.target;
  if (t.dataset.opt) {
    const key = t.dataset.opt;
    const val = t.type === 'checkbox' ? t.checked : Number(t.value);
    saveSettings((s) => { s.scrcpy[key] = val; });
  } else if (t.dataset.set) {
    const key = t.dataset.set;
    const val = t.type === 'checkbox' ? t.checked : t.value.trim();
    saveSettings((s) => { s[key] = val; }).then(() => {
      if (key === 'captureDir') refreshCaptureDir().then(render);
      if (key === 'adbPath') refreshInstances();
    });
  }
});

// Consola adb shell.
document.addEventListener('submit', async (e) => {
  if (e.target.id !== 'console-form') return;
  e.preventDefault();
  const d = selectedDevice();
  const input = $('#console-input');
  const cmd = input.value.trim();
  if (!d || !cmd) return;
  const c = S.consoles[d.serial];
  c.history.push(cmd);
  c.idx = c.history.length;
  input.value = '';
  const out = $('#console-out');
  const append = (html) => {
    if (!c.html) out.innerHTML = '';
    c.html += html;
    if (S.selected === d.serial && $('#console-out') === out) { out.insertAdjacentHTML('beforeend', html); out.scrollTop = out.scrollHeight; }
  };
  append(`<span class="cmd">$ ${esc(cmd)}</span>\n`);
  try { append(`${esc(await invoke('shell', { serial: d.serial, command: cmd }))}\n\n`); }
  catch (err) { append(`<span class="err">${esc(err)}</span>\n\n`); }
});
document.addEventListener('keydown', (e) => {
  if (e.target.id !== 'console-input' || !['ArrowUp', 'ArrowDown'].includes(e.key)) return;
  const c = S.consoles[S.selected];
  if (!c?.history.length) return;
  e.preventDefault();
  c.idx = Math.max(0, Math.min(c.history.length, c.idx + (e.key === 'ArrowUp' ? -1 : 1)));
  e.target.value = c.history[c.idx] ?? '';
});
document.addEventListener('keydown', (e) => {
  if (e.target.id === 'port-input' && e.key === 'Enter') actions['apply-port']();
});

/* ------------------------------------------------------------------ */
/* Arranque                                                            */
/* ------------------------------------------------------------------ */
function setWinIcons(max) {
  $('[data-act="win-min"]').innerHTML = icon('min', 14, 1.9);
  $('#win-max').innerHTML = max ? icon('restore', 13, 1.9) : icon('max', 13, 1.9);
  $('#win-max').setAttribute('aria-label', max ? 'Restaurar' : 'Maximizar');
  $('[data-act="win-close"]').innerHTML = icon('close', 13, 1.9);
}

async function init() {
  setWinIcons(false);
  appWindow.onResized(async () => setWinIcons(await appWindow.isMaximized()));

  TAURI.event.listen('state', (e) => {
    const prev = `${S.snap?.sessions?.length}/${S.snap?.recordings?.length}`;
    S.snap = e.payload;
    render();
    if (prev !== `${S.snap.sessions.length}/${S.snap.recordings.length}`) refreshInstances();
  });
  TAURI.event.listen('toast', (e) => {
    const { level, text, path } = e.payload;
    toast(level, text, path ? { label: 'Mostrar', run: () => invoke('open_path', { path }) } : undefined);
  });
  TAURI.event.listen('install', (e) => onInstallEvent(e.payload));
  TAURI.event.listen('tool', (e) => onToolEvent(e.payload));
  TAURI.event.listen('log', (e) => {
    S.logs.push(e.payload);
    if (S.logs.length > 400) S.logs.shift();
    if (S.view === 'log') renderStage();
  });

  S.snap = await invoke('get_state');
  S.logs = await invoke('get_logs');
  await refreshCaptureDir();
  render();
  await appWindow.show();
  appWindow.setFocus();
  refreshInstances();

  // Duraciones de sesión y reanálisis periódico del servidor.
  setInterval(() => {
    document.querySelectorAll('[data-since]').forEach((el) => { el.textContent = fmtDur(Date.now() - Number(el.dataset.since)); });
    document.querySelectorAll('[data-elapsed]').forEach((el) => { el.textContent = secs(Date.now() - Number(el.dataset.elapsed)); });
  }, 1000);
  setInterval(() => { if (S.view === 'adb' || S.view === 'mirror' || !S.instances) refreshInstances(); }, 5000);
  setInterval(() => { if (S.view !== 'adb' && S.view !== 'mirror') refreshInstances(); }, 20000);
}

init().catch((err) => {
  console.error(err);
  appWindow.show();
});

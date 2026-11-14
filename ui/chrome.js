(function () {
  "use strict";

  const $ = (id) => document.getElementById(id);

  const ENGINES = {
    duckduckgo: { name: "DuckDuckGo", search: (q) => "https://duckduckgo.com/?q=" + encodeURIComponent(q) },
    google: { name: "Google", search: (q) => "https://www.google.com/search?q=" + encodeURIComponent(q) },
    bing: { name: "Bing", search: (q) => "https://www.bing.com/search?q=" + encodeURIComponent(q) },
    brave: { name: "Brave", search: (q) => "https://search.brave.com/search?q=" + encodeURIComponent(q) },
    startpage: { name: "Startpage", search: (q) => "https://www.startpage.com/sp/search?query=" + encodeURIComponent(q) }
  };

  const CLOSE_ICON = '<svg viewBox="0 0 24 24"><path d="M6 6l12 12M18 6 6 18"/></svg>';
  const GEAR_ICON = '<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="3.2"/><path d="M12 3.2v2.1M12 18.7v2.1M4.8 7.8l1.8 1M17.4 15.2l1.8 1M4.8 16.2l1.8-1M17.4 8.8l1.8-1"/></svg>';

  let prefs = {
    theme: "paper",
    accent: "#9a3f8c",
    wallpaper: "none",
    compactTabs: false,
    animations: true,
    rounded: true,
    searchEngine: "google",
    language: "en",
    background: "",
    customTheme: {
      base: "#f6f7f9",
      surface: "#ffffff",
      ink: "#1d1f23",
      muted: "#71757c"
    }
  };

  let state = { tabs: [], active: 0, loading: false };
  let lastActive = 0;

  const tabsEl = $("tabs");
  const address = $("address");
  const form = $("address-form");
  const progress = $("progress");
  const security = $("security");
  const siteIcon = $("site-icon");
  const siteFavicon = $("site-favicon");
  const panel = $("panel");
  const panelMenu = $("panel-menu");
  const findbar = $("findbar");
  const findInput = $("find-input");
  const zoomValue = $("zoom-value");
  const menuButton = $("menu");

  const MENU_HEIGHT = 432;
  const FIND_HEIGHT = 152;
  let panelOpen = false;

  function ipc(message) {
    if (window.ipc) {
      window.ipc.postMessage(JSON.stringify(message));
    }
  }

  function engine() {
    return ENGINES[prefs.searchEngine] || ENGINES.google;
  }

  function applyPrefs(next) {
    prefs = Object.assign({}, prefs, next || {});
    document.body.dataset.theme = prefs.theme || "paper";
    document.documentElement.style.setProperty("--accent", prefs.accent || "#9a3f8c");
    if (window.mestyApplyTheme) window.mestyApplyTheme(prefs.theme || "paper", prefs.customTheme);
    document.body.classList.toggle("compact", !!prefs.compactTabs);
    document.body.classList.toggle("no-anim", !prefs.animations);
    document.body.classList.toggle("square", !prefs.rounded);
    if (window.mestyApplyI18n) window.mestyApplyI18n(prefs.language || "en", document);
    address.placeholder = window.mestyT
      ? window.mestyT(prefs.language || "en", "search", { engine: engine().name })
      : "Search " + engine().name + " or enter address";
  }

  function activeTab() {
    return state.tabs.find((tab) => tab.id === state.active) || null;
  }

  function hostOf(url) {
    try {
      return new URL(url).hostname.replace(/^www\./, "");
    } catch (error) {
      return "";
    }
  }

  function isWebLike(tab) {
    return tab.kind === "web" || /^https?:\/\//i.test(tab.url || "");
  }

  function letterFor(tab) {
    const source = isWebLike(tab) ? hostOf(tab.url) || tab.title : tab.title;
    return (String(source || "").trim()[0] || "M").toUpperCase();
  }

  function faviconImage(source, tab, fallback) {
    const img = document.createElement("img");
    img.src = source;
    img.alt = "";
    img.draggable = false;
    img.addEventListener("error", () => {
      img.remove();
      fallback();
    }, { once: true });
    return img;
  }

  function makeFavicon(tab) {
    const favicon = document.createElement("span");
    favicon.className = "favicon";

    if (tab.kind === "settings") {
      favicon.innerHTML = GEAR_ICON;
      return favicon;
    }

    if (tab.favicon) {
      favicon.appendChild(faviconImage(tab.favicon, tab, () => {
        favicon.textContent = letterFor(tab);
      }));
      return favicon;
    }

    if (tab.kind === "start" && !isWebLike(tab)) {
      const brand = document.querySelector(".brand img");
      if (brand && brand.src) {
        favicon.appendChild(faviconImage(brand.src, tab, () => {
          favicon.textContent = letterFor(tab);
        }));
        return favicon;
      }
    }

    favicon.textContent = letterFor(tab);
    return favicon;
  }

  function renderTabs() {
    const fragment = document.createDocumentFragment();

    state.tabs.forEach((tab) => {
      const element = document.createElement("div");
      element.className = "tab"
        + (tab.id === state.active ? " active" : "")
        + (tab.incognito ? " incognito" : "")
        + (tab.loading ? " loading" : "");
      element.dataset.id = String(tab.id);
      element.title = tab.title || "New Tab";
      element.draggable = true;

      element.appendChild(makeFavicon(tab));

      const title = document.createElement("span");
      title.className = "title";
      title.textContent = tab.title || "New Tab";
      element.appendChild(title);

      const close = document.createElement("button");
      close.className = "close";
      close.type = "button";
      close.title = "Close tab";
      close.setAttribute("aria-label", "Close tab");
      close.innerHTML = CLOSE_ICON;
      element.appendChild(close);

      fragment.appendChild(element);
    });

    tabsEl.textContent = "";
    tabsEl.appendChild(fragment);
  }

  function renderState(next) {
    const previousActive = lastActive;
    state = next || { tabs: [], active: 0, loading: false };

    renderTabs();

    const active = activeTab();
    const url = active ? active.url || "" : "";
    const incognito = !!(active && active.incognito);
    const editing = document.activeElement === address;

    document.body.classList.toggle("incognito", incognito);
    const incognitoChip = $("incognito-chip");
    if (incognitoChip) incognitoChip.hidden = !incognito;

    if (!editing || previousActive !== state.active) {
      address.value = url;
    }

    if (url.startsWith("https://")) {
      security.className = "security secure";
      security.title = "Secure connection";
    } else if (url.startsWith("http://")) {
      security.className = "security warn";
      security.title = "Not secure";
    } else {
      security.className = "security";
      security.title = "MestyBrovser";
    }

    if (siteIcon && siteFavicon) {
      if (active && active.favicon) {
        if (siteFavicon.getAttribute("src") !== active.favicon) {
          siteFavicon.src = active.favicon;
        }
        siteIcon.hidden = false;
      } else {
        siteIcon.hidden = true;
        siteFavicon.removeAttribute("src");
      }
    }

    if (zoomValue) {
      const zoom = active && typeof active.zoom === "number" ? active.zoom : 1;
      zoomValue.textContent = Math.round(zoom * 100) + "%";
    }

    if (previousActive && previousActive !== state.active) {
      closePanel();
    }

    progress.classList.toggle("on", !!state.loading);
    lastActive = state.active;
  }

  function setPanelHeight(height) {
    ipc({ cmd: "panel", height: height });
  }

  function openMenu() {
    panel.hidden = false;
    panelMenu.hidden = false;
    findbar.hidden = true;
    panelOpen = true;
    setPanelHeight(MENU_HEIGHT);
  }

  function openFind() {
    panel.hidden = false;
    panelMenu.hidden = true;
    findbar.hidden = false;
    panelOpen = true;
    setPanelHeight(FIND_HEIGHT);
    window.setTimeout(() => {
      findInput.focus();
      findInput.select();
    }, 40);
  }

  function closePanel() {
    if (!panelOpen) return;
    panelOpen = false;
    panel.hidden = true;
    findbar.hidden = true;
    panelMenu.hidden = true;
    setPanelHeight(86);
  }

  function runFind(backwards) {
    const text = findInput.value;
    if (!text) return;
    ipc({ cmd: "find", text: text, backwards: !!backwards });
  }

  function isLikelyUrl(value) {
    if (/\s/.test(value)) return false;
    if (/^localhost(:\d+)?([/?#].*)?$/i.test(value)) return true;
    if (/^(\d{1,3}\.){3}\d{1,3}(:\d+)?([/?#].*)?$/.test(value)) return true;
    return /^[^\s/]+\.[^\s/]{2,}(:\d+)?([/?#].*)?$/.test(value);
  }

  function resolve(input) {
    const value = String(input || "").trim();
    if (!value) return null;
    if (/^(https?|file):\/\//i.test(value)) return value;
    if (/^(mesty|about):/i.test(value)) return value;
    if (/^[a-z][a-z0-9+.-]*:\/\//i.test(value)) return null;
    if (isLikelyUrl(value)) return "https://" + value;
    return engine().search(value);
  }

  function focusAddress() {
    address.focus();
    address.select();
  }

  function submitAddress() {
    const target = resolve(address.value);
    if (!target) return;
    ipc({ cmd: "navigate", url: target });
    address.blur();
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    submitAddress();
  });

  address.addEventListener("focus", () => {
    address.select();
  });

  address.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      const tab = activeTab();
      address.value = tab ? tab.url || "" : "";
      address.blur();
    }
  });

  $("back").addEventListener("click", () => ipc({ cmd: "back" }));
  $("forward").addEventListener("click", () => ipc({ cmd: "forward" }));
  $("reload").addEventListener("click", () => ipc({ cmd: "reload" }));
  $("new-tab").addEventListener("click", () => ipc({ cmd: "new_tab" }));
  $("customize").addEventListener("click", () => ipc({ cmd: "open_settings" }));

  menuButton.addEventListener("click", (event) => {
    event.stopPropagation();
    if (panelOpen && !panelMenu.hidden) {
      closePanel();
    } else {
      openMenu();
    }
  });

  panelMenu.addEventListener("click", (event) => {
    const button = event.target.closest("[data-action]");
    if (!button) return;
    const action = button.dataset.action;

    if (action === "find") {
      openFind();
      return;
    }

    if (action === "new_tab") ipc({ cmd: "new_tab" });
    else if (action === "new_incognito") ipc({ cmd: "new_incognito" });
    else if (action === "settings") ipc({ cmd: "open_settings" });
    else if (action === "home_editor") ipc({ cmd: "home_editor" });
    else if (action === "reopen_tab") ipc({ cmd: "reopen_tab" });
    else if (action === "fullscreen") ipc({ cmd: "fullscreen" });
    else if (action === "zoom_in") { ipc({ cmd: "zoom", action: "in" }); return; }
    else if (action === "zoom_out") { ipc({ cmd: "zoom", action: "out" }); return; }
    else if (action === "zoom_reset") { ipc({ cmd: "zoom", action: "reset" }); return; }

    closePanel();
  });

  findInput.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      runFind(event.shiftKey);
    } else if (event.key === "Escape") {
      event.preventDefault();
      closePanel();
    }
  });

  $("find-prev").addEventListener("click", () => runFind(true));
  $("find-next").addEventListener("click", () => runFind(false));
  $("find-close").addEventListener("click", () => closePanel());

  document.addEventListener("click", (event) => {
    if (!panelOpen) return;
    if (event.target.closest("#panel") || event.target.closest("#menu")) return;
    closePanel();
  });

  tabsEl.addEventListener("click", (event) => {
    const tabEl = event.target.closest(".tab");
    if (!tabEl) return;
    const id = Number(tabEl.dataset.id);
    if (event.target.closest(".close")) {
      ipc({ cmd: "close_tab", id: id });
    } else {
      ipc({ cmd: "switch_tab", id: id });
    }
  });

  tabsEl.addEventListener("auxclick", (event) => {
    if (event.button !== 1) return;
    const tabEl = event.target.closest(".tab");
    if (!tabEl) return;
    ipc({ cmd: "close_tab", id: Number(tabEl.dataset.id) });
  });

  let draggedTab = null;

  tabsEl.addEventListener("dragstart", (event) => {
    const tabEl = event.target.closest(".tab");
    if (!tabEl) return;
    draggedTab = Number(tabEl.dataset.id);
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", String(draggedTab));
    tabEl.classList.add("dragging");
  });

  tabsEl.addEventListener("dragover", (event) => {
    if (draggedTab === null) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  });

  tabsEl.addEventListener("drop", (event) => {
    if (draggedTab === null) return;
    event.preventDefault();
    const ids = state.tabs.map((tab) => tab.id).filter((id) => id !== draggedTab);
    const target = event.target.closest(".tab");
    if (target) {
      const index = ids.indexOf(Number(target.dataset.id));
      ids.splice(index < 0 ? ids.length : index, 0, draggedTab);
    } else {
      ids.push(draggedTab);
    }
    ipc({ cmd: "reorder_tabs", ids: ids });
    draggedTab = null;
  });

  tabsEl.addEventListener("dragend", () => {
    draggedTab = null;
    tabsEl.querySelectorAll(".dragging").forEach((tab) => tab.classList.remove("dragging"));
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "F11") {
      event.preventDefault();
      ipc({ cmd: "fullscreen" });
      return;
    }

    if (event.key === "Escape" && panelOpen) {
      event.preventDefault();
      closePanel();
      return;
    }

    if (event.altKey && event.key === "ArrowLeft") {
      event.preventDefault();
      ipc({ cmd: "back" });
      return;
    }
    if (event.altKey && event.key === "ArrowRight") {
      event.preventDefault();
      ipc({ cmd: "forward" });
      return;
    }

    const mod = event.ctrlKey || event.metaKey;
    if (!mod) return;

    if (event.key === "Tab") {
      event.preventDefault();
      if (!state.tabs.length) return;
      let index = state.tabs.findIndex((tab) => tab.id === state.active);
      if (index < 0) index = 0;
      const step = event.shiftKey ? -1 : 1;
      const next = state.tabs[(index + step + state.tabs.length) % state.tabs.length];
      ipc({ cmd: "switch_tab", id: next.id });
      return;
    }

    switch (event.key.toLowerCase()) {
      case "l":
        event.preventDefault();
        focusAddress();
        break;
      case "f":
        event.preventDefault();
        openFind();
        break;
      case "t":
        event.preventDefault();
        ipc({ cmd: event.shiftKey ? "reopen_tab" : "new_tab" });
        break;
      case "n":
        event.preventDefault();
        ipc({ cmd: event.shiftKey ? "new_incognito" : "home_editor" });
        break;
      case "w":
        event.preventDefault();
        ipc({ cmd: "close_active" });
        break;
      case "r":
        event.preventDefault();
        ipc({ cmd: "reload" });
        break;
      case "=":
      case "+":
        event.preventDefault();
        ipc({ cmd: "zoom", action: "in" });
        break;
      case "-":
        event.preventDefault();
        ipc({ cmd: "zoom", action: "out" });
        break;
      case "0":
        event.preventDefault();
        ipc({ cmd: "zoom", action: "reset" });
        break;
      default:
        break;
    }
  });

  window.mestySetPrefs = applyPrefs;
  window.mestySetState = renderState;
  window.mestyFocusAddress = focusAddress;
  window.mestyOpenFind = openFind;

  applyPrefs(prefs);
  ipc({ cmd: "ready" });
})();

const invoke = window.__TAURI__?.core?.invoke;
const openDialog = window.__TAURI__?.dialog?.open;
const demoMode = !invoke && new URLSearchParams(window.location.search).has("demo");

const state = {
  bindings: [],
  selectedId: null,
  query: "",
};

const els = {
  list: document.querySelector("#binding-list"),
  search: document.querySelector("#search"),
  form: document.querySelector("#binding-form"),
  id: document.querySelector("#binding-id"),
  name: document.querySelector("#name"),
  filePath: document.querySelector("#file-path"),
  shortcut: document.querySelector("#shortcut"),
  modeInputs: Array.from(document.querySelectorAll("input[name='mode']")),
  volume: document.querySelector("#volume"),
  enableAll: document.querySelector("#enable-all"),
  browseFile: document.querySelector("#browse-file"),
  inputDevice: document.querySelector("#input-device"),
  outputDevice: document.querySelector("#output-device"),
  orchestratorEnabled: document.querySelector("#orchestrator-enabled"),
  refreshDevices: document.querySelector("#refresh-devices"),
  audioRouteStatus: document.querySelector("#audio-route-status"),
  title: document.querySelector("#editor-title"),
  add: document.querySelector("#add-sound"),
  clear: document.querySelector("#clear-form"),
  delete: document.querySelector("#delete-binding"),
  stopAll: document.querySelector("#stop-all"),
  toast: document.querySelector("#toast"),
};

function showToast(message) {
  els.toast.textContent = message;
  els.toast.classList.add("visible");
  window.clearTimeout(showToast.timeout);
  showToast.timeout = window.setTimeout(() => els.toast.classList.remove("visible"), 2400);
}

async function call(command, args = {}) {
  if (!invoke) {
    showToast("Run with Tauri to use native commands.");
    return null;
  }

  try {
    return await invoke(command, args);
  } catch (error) {
    showToast(String(error));
    return null;
  }
}

async function loadAudioDevices() {
  if (demoMode) {
    renderInputDevices([
      { id: "", name: "System default", is_default: true, is_selected: false },
      { id: "Shure MV7", name: "Shure MV7", is_default: false, is_selected: true },
      { id: "MacBook Pro Microphone", name: "MacBook Pro Microphone", is_default: false, is_selected: false },
    ]);
    renderOutputDevices([
      { id: "", name: "System default", is_default: true, is_selected: false },
      { id: "BlackHole 2ch", name: "BlackHole 2ch", is_default: false, is_selected: true },
      { id: "MacBook Pro Speakers", name: "MacBook Pro Speakers", is_default: false, is_selected: false },
    ]);
    els.orchestratorEnabled.checked = true;
    updateRouteStatus();
    return;
  }

  const [inputDevices, outputDevices, settings] = await Promise.all([
    call("list_input_devices"),
    call("list_output_devices"),
    call("get_settings"),
  ]);
  if (inputDevices) {
    renderInputDevices(inputDevices);
  }
  if (outputDevices) {
    renderOutputDevices(outputDevices);
  }
  if (settings) {
    els.orchestratorEnabled.checked = Boolean(settings.audio?.orchestrator_enabled);
  }
  updateRouteStatus();
}

function renderInputDevices(devices) {
  renderDeviceOptions(els.inputDevice, devices);
}

function renderOutputDevices(devices) {
  renderDeviceOptions(els.outputDevice, devices);
}

function renderDeviceOptions(select, devices) {
  select.innerHTML = "";

  for (const device of devices) {
    const option = document.createElement("option");
    option.value = device.id;
    option.textContent = device.is_default && device.id ? `${device.name} (system default)` : device.name;
    option.selected = device.is_selected;
    select.append(option);
  }
}

function selectedOptionText(select) {
  return select.selectedOptions[0]?.textContent?.replace(" (system default)", "") ?? "system default";
}

function updateRouteStatus() {
  const mic = selectedOptionText(els.inputDevice);
  const output = selectedOptionText(els.outputDevice);
  const enabled = els.orchestratorEnabled.checked;

  if (!enabled) {
    els.audioRouteStatus.textContent = "Mic mix off. Soundboard goes to the selected Discord input device, but your physical mic is not included.";
    return;
  }

  els.audioRouteStatus.textContent = `Mic mix on. Set Discord input to ${output}; SoulBind sends ${mic} plus soundboard there.`;
}

function currentMode() {
  return els.modeInputs.find((input) => input.checked)?.value ?? "layer";
}

function setMode(mode) {
  const selected = els.modeInputs.find((input) => input.value === mode) ?? els.modeInputs[0];
  selected.checked = true;
}

function resetForm() {
  state.selectedId = null;
  els.id.value = "";
  els.name.value = "";
  els.filePath.value = "";
  els.shortcut.value = "";
  setMode("layer");
  els.volume.value = "1";
  els.title.textContent = "New Binding";
  els.delete.disabled = true;
}

function selectBinding(binding) {
  state.selectedId = binding.id;
  els.id.value = binding.id;
  els.name.value = binding.name;
  els.filePath.value = binding.file_path;
  els.shortcut.value = binding.shortcut;
  setMode(binding.mode ?? "layer");
  els.volume.value = String(binding.volume);
  els.title.textContent = "Edit Binding";
  els.delete.disabled = false;
  render();
}

function filteredBindings() {
  const query = state.query.trim().toLowerCase();
  if (!query) return state.bindings;

  return state.bindings.filter((binding) => {
    return [binding.name, binding.shortcut, binding.file_path, binding.status]
      .join(" ")
      .toLowerCase()
      .includes(query);
  });
}

function render() {
  const bindings = filteredBindings();
  els.list.innerHTML = "";
  els.enableAll.checked = state.bindings.length > 0 && state.bindings.every((binding) => binding.enabled);

  if (bindings.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state";
    empty.textContent = state.bindings.length === 0 ? "No sound bindings yet." : "No matching bindings.";
    els.list.append(empty);
    return;
  }

  for (const binding of bindings) {
    const row = document.createElement("article");
    row.className = `binding-row${binding.id === state.selectedId ? " active" : ""}`;

    const statusClass = binding.status === "registered" || binding.status === "disabled" ? "chip" : "chip error";
    row.innerHTML = `
      <div class="binding-main">
        <h3 class="binding-name"></h3>
        <div class="binding-meta">
          <span class="chip shortcut"></span>
          <span class="${statusClass} status"></span>
          <span class="chip volume"></span>
          <span class="chip mode"></span>
        </div>
      </div>
      <div class="row-actions">
        <button class="icon-button play" type="button" aria-label="Play sound" title="Play sound" ${binding.enabled ? "" : "disabled"}>
          <span class="icon icon-play" aria-hidden="true"></span>
        </button>
        <button class="icon-button duplicate" type="button" aria-label="Duplicate binding" title="Duplicate binding">
          <span class="icon icon-copy" aria-hidden="true"></span>
        </button>
        <button class="icon-button toggle" type="button" aria-label="${binding.enabled ? "Disable binding" : "Enable binding"}" title="${binding.enabled ? "Disable binding" : "Enable binding"}">
          <span class="icon ${binding.enabled ? "icon-toggle-on" : "icon-toggle-off"}" aria-hidden="true"></span>
        </button>
        <button class="icon-button edit" type="button" aria-label="Edit binding" title="Edit binding">
          <span class="icon icon-edit" aria-hidden="true"></span>
        </button>
      </div>
    `;

    row.querySelector(".binding-name").textContent = binding.name;
    row.querySelector(".shortcut").textContent = binding.shortcut;
    row.querySelector(".status").textContent = binding.status;
    row.querySelector(".volume").textContent = `${Math.round(binding.volume * 100)}%`;
    row.querySelector(".mode").textContent = binding.mode ?? "layer";
    row.querySelector(".play").addEventListener("click", async () => {
      await call("play_binding", { id: binding.id });
    });
    row.querySelector(".duplicate").addEventListener("click", async () => {
      const result = await call("duplicate_binding", { id: binding.id });
      if (result) {
        state.bindings = result;
        render();
      }
    });
    row.querySelector(".toggle").addEventListener("click", async () => {
      const result = await call("set_binding_enabled", { id: binding.id, enabled: !binding.enabled });
      if (result) {
        state.bindings = result;
        render();
      }
    });
    row.querySelector(".edit").addEventListener("click", () => selectBinding(binding));
    row.addEventListener("dblclick", () => selectBinding(binding));
    els.list.append(row);
  }
}

async function refresh() {
  if (demoMode) {
    state.bindings = [
      {
        id: "demo-1",
        name: "Intro sting",
        file_path: "/Users/me/Sounds/intro-sting.wav",
        shortcut: "CmdOrControl+Shift+1",
        volume: 0.85,
        enabled: true,
        mode: "layer",
        status: "registered",
      },
      {
        id: "demo-2",
        name: "Long notification name that still wraps cleanly",
        file_path: "/Users/me/Sounds/notification-long-file-name.mp3",
        shortcut: "CmdOrControl+Alt+N",
        volume: 1.2,
        enabled: false,
        mode: "restart",
        status: "disabled",
      },
    ];
    render();
    return;
  }

  const bindings = await call("list_bindings");
  if (bindings) {
    state.bindings = bindings;
  }
  render();
}

els.search.addEventListener("input", (event) => {
  state.query = event.target.value;
  render();
});

els.add.addEventListener("click", () => {
  resetForm();
  els.name.focus();
});

els.clear.addEventListener("click", resetForm);

els.stopAll.addEventListener("click", async () => {
  await call("stop_all");
});

els.refreshDevices.addEventListener("click", loadAudioDevices);

els.inputDevice.addEventListener("change", async (event) => {
  if (demoMode) {
    updateRouteStatus();
    showToast("Mic route changed in preview.");
    return;
  }

  const selected = event.target.value || null;
  const settings = await call("set_input_device", { deviceName: selected });
  if (settings) {
    await loadAudioDevices();
    showToast("Mic route saved.");
  }
});

els.outputDevice.addEventListener("change", async (event) => {
  if (demoMode) {
    updateRouteStatus();
    showToast("Discord input changed in preview.");
    return;
  }

  const selected = event.target.value || null;
  const settings = await call("set_output_device", { deviceName: selected });
  if (settings) {
    await loadAudioDevices();
    showToast("Discord input saved.");
  }
});

els.orchestratorEnabled.addEventListener("change", async (event) => {
  if (demoMode) {
    updateRouteStatus();
    showToast(event.target.checked ? "Mic mix enabled in preview." : "Mic mix disabled in preview.");
    return;
  }

  const settings = await call("set_orchestrator_enabled", { enabled: event.target.checked });
  if (settings) {
    await loadAudioDevices();
    showToast(event.target.checked ? "Mic mix enabled." : "Mic mix disabled.");
  } else {
    event.target.checked = !event.target.checked;
    updateRouteStatus();
  }
});

els.enableAll.addEventListener("change", async (event) => {
  const result = await call("set_all_enabled", { enabled: event.target.checked });
  if (result) {
    state.bindings = result;
    render();
  }
});

els.browseFile.addEventListener("click", async () => {
  if (!openDialog) {
    showToast("File picker is available when running in Tauri.");
    return;
  }

  const selected = await openDialog({
    multiple: false,
    filters: [
      {
        name: "Audio",
        extensions: ["wav", "mp3", "ogg", "flac"],
      },
    ],
  });

  if (typeof selected === "string") {
    els.filePath.value = selected;
  }
});

els.delete.addEventListener("click", async () => {
  if (!state.selectedId) return;
  const deleted = await call("delete_binding", { id: state.selectedId });
  if (deleted) {
    resetForm();
    await refresh();
  }
});

els.form.addEventListener("submit", async (event) => {
  event.preventDefault();
  const payload = {
    name: els.name.value.trim(),
    file_path: els.filePath.value.trim(),
    shortcut: els.shortcut.value.trim(),
    mode: currentMode(),
    volume: Number(els.volume.value),
  };

  const command = state.selectedId ? "update_binding" : "add_binding";
  const args = state.selectedId ? { id: state.selectedId, input: payload } : { input: payload };
  const result = await call(command, args);

  if (result) {
    state.bindings = result;
    showToast("Binding saved.");
    resetForm();
    render();
  }
});

els.shortcut.addEventListener("keydown", (event) => {
  if (event.key === "Tab") return;
  event.preventDefault();

  if (event.key === "Escape") {
    els.shortcut.blur();
    return;
  }

  const parts = [];
  if (event.metaKey || event.ctrlKey) parts.push("CmdOrControl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");

  const key = event.key.length === 1 ? event.key.toUpperCase() : event.key;
  if (!["Control", "Shift", "Alt", "Meta"].includes(key)) {
    parts.push(key === " " ? "Space" : key);
  }

  if (parts.length >= 2) {
    els.shortcut.value = Array.from(new Set(parts)).join("+");
  }
});

resetForm();
refresh();
loadAudioDevices();

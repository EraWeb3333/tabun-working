import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

const STORAGE_KEY = "tabun-working-presets";
const DEFAULT_PRESETS = [
  "たぶん作業中",
  "集中しています",
  "返事は遅め",
  "休憩中",
  "ゲームじゃないです",
];

const app = document.querySelector<HTMLDivElement>("#app");

if (!app) {
  throw new Error("App root not found");
}

app.innerHTML = `
  <main class="shell">
    <section class="deck">
      <header class="hero">
        <div class="mark" aria-hidden="true"><span></span></div>
        <div>
          <p class="label">DISCORD STATUS DECK</p>
          <h1>今日は、なんてことにする？</h1>
        </div>
      </header>

      <section class="active-panel" aria-live="polite">
        <span class="pulse"></span>
        <div>
          <p>現在のステータス</p>
          <strong id="active-status">停止中</strong>
        </div>
      </section>

      <div class="field">
        <label for="preset-select">プリセット</label>
        <div class="select-wrap">
          <select id="preset-select"></select>
          <svg viewBox="0 0 20 20" aria-hidden="true">
            <path d="m5 7 5 6 5-6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
      </div>

      <div class="field">
        <label for="custom-status">新しい表示名</label>
        <div class="input-row">
          <input id="custom-status" maxlength="60" placeholder="例：コーヒーを淹れています" autocomplete="off" />
          <button id="save-preset" class="secondary" type="button">保存</button>
        </div>
      </div>

      <div class="preview">
        <span>次に表示する名前</span>
        <strong id="preview-name">たぶん作業中</strong>
      </div>

      <div class="primary-actions">
        <button id="apply-status" class="primary" type="button">
          <span>この名前で起動</span>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M5 12h13m-5-5 5 5-5 5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        <button id="stop-status" class="danger" type="button">停止</button>
      </div>

      <footer>
        <button id="delete-preset" class="text-button" type="button">選択中を削除</button>
        <button id="minimize" class="text-button" type="button">この画面を最小化</button>
      </footer>

      <p id="message" class="message" role="status"></p>
    </section>
  </main>
`;

const select = document.querySelector<HTMLSelectElement>("#preset-select");
const input = document.querySelector<HTMLInputElement>("#custom-status");
const preview = document.querySelector<HTMLElement>("#preview-name");
const activeStatus = document.querySelector<HTMLElement>("#active-status");
const message = document.querySelector<HTMLElement>("#message");

if (!select || !input || !preview || !activeStatus || !message) {
  throw new Error("Required controls were not found");
}

function readPresets(): string[] {
  try {
    const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "[]");
    if (Array.isArray(saved)) {
      return [...new Set([...DEFAULT_PRESETS, ...saved.filter((item) => typeof item === "string")])];
    }
  } catch {
    localStorage.removeItem(STORAGE_KEY);
  }
  return [...DEFAULT_PRESETS];
}

let presets = readPresets();

function savePresets() {
  const customPresets = presets.filter((preset) => !DEFAULT_PRESETS.includes(preset));
  localStorage.setItem(STORAGE_KEY, JSON.stringify(customPresets));
}

function renderPresets(selected?: string) {
  select.innerHTML = presets
    .map((preset) => `<option value="${escapeHtml(preset)}">${escapeHtml(preset)}</option>`)
    .join("");

  if (selected && presets.includes(selected)) {
    select.value = selected;
  }
  updatePreview();
}

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function updatePreview() {
  preview.textContent = input.value.trim() || select.value;
}

function showMessage(text: string, isError = false) {
  message.textContent = text;
  message.classList.toggle("error", isError);
}

select.addEventListener("change", () => {
  input.value = "";
  updatePreview();
  showMessage("");
});

input.addEventListener("input", updatePreview);

document.querySelector("#save-preset")?.addEventListener("click", () => {
  const name = input.value.trim();
  if (!name) {
    showMessage("保存する名前を入力してください。", true);
    input.focus();
    return;
  }

  if (!presets.includes(name)) {
    presets.push(name);
    savePresets();
  }
  renderPresets(name);
  input.value = "";
  showMessage(`「${name}」を保存しました。`);
});

document.querySelector("#delete-preset")?.addEventListener("click", () => {
  const name = select.value;
  if (DEFAULT_PRESETS.includes(name)) {
    showMessage("最初から入っているプリセットは削除できません。", true);
    return;
  }

  presets = presets.filter((preset) => preset !== name);
  savePresets();
  renderPresets();
  showMessage(`「${name}」を削除しました。`);
});

document.querySelector("#apply-status")?.addEventListener("click", async () => {
  const name = input.value.trim() || select.value;
  if (!name) {
    showMessage("表示名を選ぶか入力してください。", true);
    return;
  }

  showMessage("ステータスを切り替えています…");
  try {
    await invoke<string>("activate_status", { name });
    activeStatus.textContent = name;
    showMessage(`「${name}」で起動しました。`);
  } catch (error) {
    showMessage(String(error), true);
  }
});

document.querySelector("#stop-status")?.addEventListener("click", async () => {
  try {
    await invoke("stop_status");
    activeStatus.textContent = "停止中";
    showMessage("ステータスを停止しました。");
  } catch (error) {
    showMessage(String(error), true);
  }
});

document.querySelector("#minimize")?.addEventListener("click", async () => {
  try {
    await invoke("minimize_launcher");
  } catch (error) {
    showMessage(String(error), true);
  }
});

async function initialize() {
  renderPresets();
  try {
    const active = await invoke<string | null>("get_active_status");
    activeStatus.textContent = active || "停止中";
    if (active) {
      if (presets.includes(active)) {
        select.value = active;
        input.value = "";
      } else {
        input.value = active;
      }
      updatePreview();
    }
  } catch {
    activeStatus.textContent = "確認できません";
  }
}

void initialize();

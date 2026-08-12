import type {
  ProxyMode,
  ProxySettings,
  ProxySettingsInput,
  ProxySource,
  ProxyTestResult,
  ProxyType
} from "~/types/proxy-settings";

export type ProxyOperationStatus = {
  kind: "success" | "error"
  stage?: "update" | "readback"
  message: string
} | null;

interface ProxyFieldErrors {
  pacUrl: string
  host: string
  port: string
  username: string
  password: string
  bypass: string
  targetUrl: string
}

const DEFAULT_BYPASS = ["localhost", "127.0.0.0/8", "::1"];

function createDefaultSettings(): ProxySettings {
  return {
    mode: "direct",
    preferredMode: "system",
    pacUrl: "",
    proxyType: "http",
    host: "",
    port: null,
    username: "",
    bypass: [...DEFAULT_BYPASS],
    hasPassword: false,
    warning: undefined
  };
}

function createEmptyErrors(): ProxyFieldErrors {
  return {
    pacUrl: "",
    host: "",
    port: "",
    username: "",
    password: "",
    bypass: "",
    targetUrl: ""
  };
}

function normalizeSettings(value: ProxySettings): ProxySettings {
  const mode: ProxyMode = ["direct", "system", "pac", "manual"].includes(value?.mode) ? value.mode : "direct";
  const proxyType: ProxyType = value?.proxyType === "socks5" ? "socks5" : "http";
  const rawPort = value?.port;
  const port = Number.isInteger(rawPort) && Number(rawPort) >= 1 && Number(rawPort) <= 65535 ? Number(rawPort) : null;
  const hasLegacyManualConfig = typeof value?.host === "string" && Boolean(value.host.trim()) && port != null;
  const preferredMode: ProxySource = ["system", "pac", "manual"].includes(value?.preferredMode)
    ? value.preferredMode
    : mode === "pac"
      ? "pac"
      : mode === "manual" || hasLegacyManualConfig
        ? "manual"
        : "system";
  const bypass = Array.isArray(value?.bypass)
    ? value.bypass.map((entry) => String(entry).trim()).filter(Boolean)
    : [...DEFAULT_BYPASS];

  return {
    mode,
    preferredMode,
    pacUrl: typeof value?.pacUrl === "string" ? value.pacUrl : "",
    proxyType,
    host: typeof value?.host === "string" ? value.host : "",
    port,
    username: typeof value?.username === "string" ? value.username : "",
    bypass,
    hasPassword: value?.hasPassword === true,
    warning: typeof value?.warning === "string" && value.warning ? value.warning : undefined
  };
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) return String(error.message);
  return String(error || "Unknown error");
}

export function parseBypassEntries(value: string): string[] {
  const seen = new Set<string>();

  return value
    .split(/[\n,]/)
    .map((entry) => entry.trim())
    .filter((entry) => {
      const key = entry.toLowerCase();
      if (!entry || seen.has(key)) return false;
      seen.add(key);
      return true;
    });
}

export function validateJumpServerOrigin(value: string): boolean {
  try {
    const url = new URL(value.trim());
    return (
      ["http:", "https:"].includes(url.protocol)
      && Boolean(url.hostname)
      && !url.username
      && !url.password
      && (url.pathname === "" || url.pathname === "/")
      && !url.search
      && !url.hash
    );
  } catch {
    return false;
  }
}

export function validatePacUrl(value: string): boolean {
  const trimmed = value.trim();
  try {
    const url = new URL(trimmed);
    const authority = trimmed.match(/^[^:]+:\/\/([^/?#]*)/)?.[1];
    return (
      ["http:", "https:"].includes(url.protocol)
      && Boolean(url.hostname)
      && Boolean(authority)
      && !authority?.includes("@")
      && !url.username
      && !url.password
    );
  } catch {
    return false;
  }
}

export function useProxySettings() {
  const settings = reactive<ProxySettings>(createDefaultSettings());
  const password = shallowRef("");
  const clearPassword = shallowRef(false);
  const bypassText = shallowRef(DEFAULT_BYPASS.join("\n"));
  const targetUrl = shallowRef("");
  const errors = reactive<ProxyFieldErrors>(createEmptyErrors());

  const isLoading = shallowRef(true);
  const isSaving = shallowRef(false);
  const isTesting = shallowRef(false);
  const loadError = shallowRef("");
  const saveStatus = shallowRef<ProxyOperationStatus>(null);
  const testResult = shallowRef<ProxyTestResult | null>(null);

  function applySettings(value: ProxySettings) {
    const normalized = normalizeSettings(value);
    Object.assign(settings, normalized);
    bypassText.value = normalized.bypass.join("\n");
    password.value = "";
    clearPassword.value = false;
  }

  function clearErrors() {
    Object.assign(errors, createEmptyErrors());
  }

  function buildInput(): ProxySettingsInput {
    const rawPort = settings.port == null ? "" : String(settings.port).trim();
    const canChangePassword = settings.mode === "manual";
    const input: ProxySettingsInput = {
      mode: settings.mode,
      preferredMode: settings.preferredMode,
      pacUrl: settings.pacUrl.trim(),
      proxyType: settings.proxyType,
      host: settings.host.trim(),
      port: rawPort ? Number(rawPort) : null,
      username: settings.username.trim(),
      bypass: parseBypassEntries(bypassText.value),
      clearPassword: canChangePassword && clearPassword.value
    };

    if (canChangePassword && clearPassword.value) input.password = null;
    else if (canChangePassword && password.value) input.password = password.value;

    return input;
  }

  async function load() {
    isLoading.value = true;
    loadError.value = "";
    try {
      const value = await useTauriCoreInvoke<ProxySettings>("get_proxy_settings", {});
      applySettings(value);
    } catch (error) {
      loadError.value = errorMessage(error);
    } finally {
      isLoading.value = false;
    }
  }

  async function save(): Promise<boolean> {
    isSaving.value = true;
    saveStatus.value = null;
    let updateCompleted = false;
    try {
      await useTauriCoreInvoke<ProxySettings>("update_proxy_settings", { settings: buildInput() });
      updateCompleted = true;
      const readback = await useTauriCoreInvoke<ProxySettings>("get_proxy_settings", {});
      applySettings(readback);
      saveStatus.value = { kind: "success", message: "" };
      return true;
    } catch (error) {
      saveStatus.value = {
        kind: "error",
        stage: updateCompleted ? "readback" : "update",
        message: errorMessage(error)
      };
      return false;
    } finally {
      isSaving.value = false;
    }
  }

  async function test(): Promise<boolean> {
    isTesting.value = true;
    testResult.value = null;
    try {
      testResult.value = await useTauriCoreInvoke<ProxyTestResult>("test_proxy_settings", {
        settings: buildInput(),
        targetUrl: targetUrl.value.trim()
      });
      return testResult.value.success;
    } catch (error) {
      testResult.value = {
        success: false,
        elapsedMs: 0,
        message: errorMessage(error)
      };
      return false;
    } finally {
      isTesting.value = false;
    }
  }

  return {
    settings,
    password,
    clearPassword,
    bypassText,
    targetUrl,
    errors,
    isLoading,
    isSaving,
    isTesting,
    loadError,
    saveStatus,
    testResult,
    load,
    save,
    test,
    clearErrors
  };
}

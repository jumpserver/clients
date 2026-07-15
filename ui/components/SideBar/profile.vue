<script setup lang="ts">
import type { DropdownMenuItem } from "@nuxt/ui";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { LangType, ThemeType, UserData } from "~/types/index";

import { LogicalPosition } from "@tauri-apps/api/dpi";
import { useSettingManager } from "~/composables/useSettingManager";
import { useUserInfoStore } from "~/store/modules/userInfo";
import RecentSites from "./recentSites.vue";

interface VersionAlertPayload {
  type: string
  version?: string
}

interface VersionMessageResponse {
  status: number
  data: string
  success: boolean
}

const props = defineProps<{ collapse: boolean }>();

const recentSiteLimit = 5;

const toast = useToast();
const appConfig = useAppConfig();
const localePath = useLocalePath();
const userInfoStore = useUserInfoStore();

const { t, locales, locale } = useI18n();
const { loggedIn, currentSite, userMap, currentUser } = storeToRefs(userInfoStore);
const { isMacOS } = usePlatform();

const {
  setLang,
  theme,
  themeMode,
  primaryColorLight,
  primaryColorDark,
  recentSites,
  setRecentSites,
  hydrationPromise
} = useSettingManager();
const { manualSetTheme, enableFollowSystem, followSystem, userTheme } = useThemeAdapter();
const { applyPrimaryColor } = useColor();

const inputSite = ref("");
const errorMessage = ref("");
const loginBtn = ref(false);
const openModal = ref(false);
const hasValidationError = ref(false);
const recentSitesDismissed = ref(false);
const unlistenErrorPageRef = ref<UnlistenFn | null>(null);
const unlistenLoginFailedRef = ref<UnlistenFn | null>(null);
const inputRef = ref<ComponentPublicInstance | null>(null);

let loginBtnUnlockTimer: ReturnType<typeof setTimeout> | null = null;

useEventBus().on("login", openLoginPage);

const normalizedInputSite = computed(() => normalizeSite(inputSite.value));

const normalizedRecentSites = computed(() => {
  const raw = Array.isArray(recentSites.value) ? recentSites.value : [];
  const normalized: string[] = [];
  const seen = new Set<string>();

  for (const site of raw) {
    const value = normalizeSite(site);
    if (!value || seen.has(value)) continue;
    seen.add(value);
    normalized.push(value);
    if (normalized.length >= recentSiteLimit) break;
  }

  return normalized;
});

const filteredRecentSites = computed(() => {
  const query = normalizeSite(inputSite.value).toLowerCase();
  const list = normalizedRecentSites.value;
  if (!query) return list;

  return list.filter((site) => site.toLowerCase().includes(query));
});

const showRecentSites = computed(
  () => openModal.value && !recentSitesDismissed.value && filteredRecentSites.value.length > 0
);

const clearLoginBtnUnlockTimer = () => {
  if (loginBtnUnlockTimer) {
    clearTimeout(loginBtnUnlockTimer);
    loginBtnUnlockTimer = null;
  }
};

const enableLoginBtnAfter = (ms: number) => {
  clearLoginBtnUnlockTimer();

  loginBtnUnlockTimer = setTimeout(() => {
    loginBtn.value = false;
    loginBtnUnlockTimer = null;
  }, ms);
};

const selectedLanguage = computed<LangType>({
  get: () => (locale.value as LangType) || "zh",
  set: (code: LangType) => {
    if (!code) return;
    setLang(code);
  }
});

const languageItems = computed(() => {
  const arr = (locales.value as any[]) || [];
  return arr.map((l: any) => ({
    id: l.code || l,
    label: l.name || l
  }));
});

const languageChildren = computed<DropdownMenuItem[][]>(() => [
  languageItems.value.map((item) => ({
    label: item.label,
    type: "checkbox",
    checked: selectedLanguage.value === (item.id as LangType),
    onUpdateChecked: (checked: boolean) => {
      if (!checked) return;
      handleLanguageChange(item.id as LangType);
    }
  }))
]);

const appearanceOptions = computed(() => [
  { id: "withSystem", label: t("Common.WithSystem") },
  { id: "light", label: t("Common.Light") },
  { id: "dark", label: t("Common.Dark") }
]);

const selectedAppearance = computed<ThemeType>({
  get: () => {
    const mode = (themeMode.value || "") as ThemeType;
    if (mode === "withSystem" || mode === "dark" || mode === "light") return mode;

    if (followSystem.value) return "withSystem";

    const saved = (theme.value || "") as ThemeType;
    if (saved === "dark" || saved === "light") return saved;

    const current = (userTheme.value || "") as ThemeType;
    if (current === "dark" || current === "light") return current;

    return "light";
  },
  set: (id: ThemeType) => {
    if (id === "withSystem") {
      void enableFollowSystem().then(() => {
        nextTick().then(() => applyCurrentThemeColor(true));
      });
      return;
    }

    manualSetTheme(id as any);
    nextTick().then(() => applyCurrentThemeColor(true));
  }
});

const appearanceChildren = computed<DropdownMenuItem[][]>(() => [
  appearanceOptions.value.map((opt) => ({
    label: opt.label,
    type: "checkbox",
    checked: selectedAppearance.value === (opt.id as ThemeType),
    onUpdateChecked: (checked: boolean) => {
      if (!checked) return;
      if (selectedAppearance.value !== (opt.id as ThemeType)) {
        selectedAppearance.value = opt.id as ThemeType;
      }
    }
  }))
]);

const toolChildren = computed<DropdownMenuItem[][]>(() => [
  [
    {
      label: t("Menu.Player"),
      icon: "lucide:clapperboard",
      onClick: () => openToolWindow("videoplayer", t("Menu.Player"), "/videoplayer", 1480, 920, 1320, 860)
    },
    {
      label: t("Menu.Transcode"),
      icon: "lucide:repeat-2",
      onClick: () => openToolWindow("transcode", t("Menu.Transcode"), "/transcode", 1180, 760, 960, 640)
    }
  ]
]);

const profileDisplayName = computed(() => {
  if (loggedIn.value) return currentUser.value?.name || "";
  return t("Common.UnSigned");
});

const profileMenuItems = computed<DropdownMenuItem[][]>(() => {
  const mainItems: DropdownMenuItem[] = [
    {
      label: t("Login.AddAccount"),
      icon: "i-lucide-user-round-plus",
      onClick: openLoginPage
    },
    {
      label: t("Common.Appearance"),
      icon: "solar:palette-linear",
      children: appearanceChildren.value
    },
    {
      label: t("Menu.Tool"),
      icon: "lucide:wrench",
      children: toolChildren.value
    },
    {
      label: t("Common.Language"),
      icon: "solar:global-outline",
      children: languageChildren.value
    }
  ];

  if (loggedIn.value) {
    mainItems.splice(1, 0, {
      label: t("Login.SwitchSite"),
      icon: "i-lucide-arrow-down-up",
      children: switchAccountChildren()
    });

    return [
      mainItems,
      [
        {
          label: t("Login.Logout"),
          icon: "solar:login-outline",
          color: "error",
          ui: {
            itemLabel:
              "!text-error group-data-highlighted:!text-error group-data-[state=open]:!text-error group-data-[state=checked]:!text-error",
            itemLeadingIcon:
              "group-data-[state=checked]:text-error group-data-highlighted:!text-error group-data-[state=open]:!text-error"
          },
          onClick: clearAuthInfo
        }
      ]
    ];
  }

  return [mainItems];
});

watch(
  () => userTheme.value,
  () => {
    applyCurrentThemeColor();
  }
);

function applyCurrentThemeColor(broadcast = false) {
  const modeNow = (userTheme.value as string) || (selectedAppearance.value as string);
  const hexNow = modeNow === "dark" ? primaryColorDark.value : primaryColorLight.value;

  if (hexNow) {
    applyPrimaryColor(hexNow);
    if (broadcast) {
      useTauriEventEmit("primary-color-changed", { hex: hexNow, mode: modeNow });
    }
  }
}

function handleLanguageChange(code: LangType) {
  if (!code || code === selectedLanguage.value) return;

  selectedLanguage.value = code;
}

async function openToolWindow(
  label: string,
  title: string,
  path: string,
  width: number,
  height: number,
  minWidth: number,
  minHeight: number
) {
  const url = localePath({ path });

  try {
    const existing = await useTauriWebviewWindowWebviewWindow.getByLabel(label);

    if (existing) {
      if (await existing.isMinimized()) {
        await existing.unminimize();
      }

      if (!(await existing.isVisible())) {
        await existing.show();
      }

      await existing.setFocus();
      return;
    }

    const useNativeWindowFrame = isMacOS.value;

    // eslint-disable-next-line no-new
    new useTauriWebviewWindowWebviewWindow(label, {
      title,
      url,
      width,
      height,
      minWidth,
      minHeight,
      hiddenTitle: true,
      titleBarStyle: "overlay",
      // Tauri's native handler must be disabled so UFileUpload receives HTML5 drag/drop events.
      dragDropEnabled: false,
      trafficLightPosition: new LogicalPosition(10, 22),
      decorations: useNativeWindowFrame,
      shadow: useNativeWindowFrame
    });
  } catch {
    await navigateTo(url);
  }
}

/**
 * @description 标准化站点输入：去除首尾空格 + 去除末尾斜杠
 * @param value 站点输入
 * @returns 标准化后的站点
 */
function normalizeSite(value: string): string {
  const s = (value || "").trim();
  if (!s) return "";
  return s.replace(/\/+$/, "");
}

const ensureRecentSitesReady = async () => {
  if (hydrationPromise.value) {
    await hydrationPromise.value;
  }
};

const saveRecentSite = async (site: string) => {
  try {
    const normalized = normalizeSite(site);
    if (!normalized) return;

    await ensureRecentSitesReady();
    const next = [normalized, ...normalizedRecentSites.value.filter((item) => item !== normalized)].slice(
      0,
      recentSiteLimit
    );
    setRecentSites(next);
  } catch (err) {
    console.error("save recent sites failed", err);
  }
};

const removeRecentSite = async (site: string) => {
  try {
    const normalized = normalizeSite(site);
    if (!normalized) return;

    await ensureRecentSitesReady();
    const next = normalizedRecentSites.value.filter((item) => item !== normalized);
    setRecentSites(next);
  } catch (err) {
    console.error("remove recent site failed", err);
  }
};

const clearRecentSites = async () => {
  try {
    await ensureRecentSitesReady();
    setRecentSites([]);
  } catch (err) {
    console.error("clear recent sites failed", err);
  }
};

const selectRecentSite = (site: string) => {
  inputSite.value = site;
  clearValidationError();
  recentSitesDismissed.value = true;
  nextTick(() => {
    inputRef.value?.$el?.querySelector("input")?.focus();
  });
};

const handleClearInput = () => {
  inputSite.value = "";
  clearValidationError();
  recentSitesDismissed.value = false;
};

function normalizeVersionMessage(response: VersionMessageResponse) {
  if (response.status === 404) {
    return { status: "incompatible" as const, versions: [] as string[] };
  }

  if (!response.data) {
    return { status: "list" as const, versions: [] as string[] };
  }

  try {
    const parsed = JSON.parse(response.data);
    const versions = Array.isArray(parsed)
      ? parsed.map((item) => (item == null ? "" : String(item))).filter((v) => v.length > 0)
      : [];
    return { status: "list" as const, versions };
  } catch {
    return { status: "list" as const, versions: [] as string[] };
  }
}

function normalizeMajorMinor(version: string) {
  const cleaned = (version || "").trim();
  if (!cleaned) return "";

  const parts = cleaned.split(".");
  const major = (parts[0] || "").replace(/\D/g, "");
  if (!major) return "";

  const minor = (parts[1] || "").replace(/\D/g, "");
  return minor ? `${major}.${minor}` : major;
}

function normalizeMajorMinorList(versions: string[]) {
  const normalized: string[] = [];
  const seen = new Set<string>();

  for (const version of versions) {
    const value = normalizeMajorMinor(version);
    if (!value || seen.has(value)) continue;
    seen.add(value);
    normalized.push(value);
  }

  return normalized;
}

const emitVersionAlertAndCloseModal = (payload: VersionAlertPayload) => {
  openModal.value = false;
  loginBtn.value = false;
  useEventBus().emit("versionAlert", payload);
};

const checkVersionBeforeOAuth = async (site: string) => {
  await useTauriCoreInvoke("set_api_session", {
    sessionKey: site,
    origin: site,
    bearerToken: "",
    orgId: ""
  });

  const [versionResponse, appVersion] = await Promise.all([
    useTauriCoreInvoke<VersionMessageResponse>("get_version_message", {}).catch(() => {
      return null;
    }),
    useTauriAppGetVersion().catch(() => "")
  ]);

  if (!versionResponse || versionResponse.status === 0) {
    console.warn("Skip version precheck before OAuth because version endpoint is unavailable", {
      site,
      versionResponse
    });
    return true;
  }

  const { status: versionStatus, versions } = normalizeVersionMessage(versionResponse);

  if (versionStatus === "incompatible") {
    emitVersionAlertAndCloseModal({ type: "incompatible" });
    return false;
  }

  const normalizedAppVersion = normalizeMajorMinor(appVersion);
  const normalizedVersions = normalizeMajorMinorList(versions);

  if (normalizedAppVersion && normalizedVersions.length > 0) {
    if (!normalizedVersions.includes(normalizedAppVersion)) {
      emitVersionAlertAndCloseModal({ type: "noMatch", version: versions[versions.length - 1] });
      return false;
    }
  } else if (appVersion && versions.length > 0 && !versions.includes(appVersion)) {
    emitVersionAlertAndCloseModal({ type: "noMatch", version: versions[versions.length - 1] });
    return false;
  }

  return true;
};

/**
 * @description 打开登录页面
 */
function openLoginPage() {
  openModal.value = true;
  recentSitesDismissed.value = false;
  hasValidationError.value = false;
  errorMessage.value = "";
  nextTick(() => {
    inputRef.value?.$el.querySelector("input")?.focus();
  });
}

/**
 * @description 清除验证错误
 */
function clearValidationError() {
  if (hasValidationError.value) {
    hasValidationError.value = false;
    errorMessage.value = "";
  }
}

/**
 * @description 切换账户子菜单
 * @returns 切换账户子菜单
 */
function switchAccountChildren() {
  const items: DropdownMenuItem[] = (Object.values(userMap.value) as UserData[]).map((u: UserData) => {
    let host = u.site;

    try {
      host = new URL(u.site).host;
    } catch (e) {
      console.log("e", e);
    }

    const label = `${host}`;
    const isCurrent = u.site === currentSite.value;

    return {
      label,
      type: "checkbox",
      checked: isCurrent,
      onUpdateChecked: (checked: boolean) => {
        if (!checked || isCurrent) return;
        handleSwitchAccount(u.site);
      }
    } as DropdownMenuItem;
  });

  return [items];
}

/**
 * @description 切换账户
 * @param site 站点
 */
function handleSwitchAccount(site: string) {
  if (site === currentSite.value) return;

  userInfoStore.setCurrentSite(site);

  nextTick(() => {
    useEventBus().emit("refresh", undefined);
  });
}

/**
 * @description 清除认证信息
 */
function clearAuthInfo() {
  userInfoStore.deleteUserData(currentSite.value);
}

/**
 * @description 过滤输入中的控制字符
 */
// eslint-disable-next-line no-control-regex
const sanitizeInput = (value: string) => value.replace(/[\u0000-\u001F\u007F-\u009F]/g, "");

/**
 * @description 输入事件处理：移除控制字符，并保持光标位置
 * 移除控制字符后，用“移除前的长度差”修正光标，保证左右键仍可正常移动
 */
const handleInputSanitize = (event: Event) => {
  const target = event.target as HTMLInputElement | null;

  if (!target) return;

  const raw = target.value;
  const caret = target.selectionStart ?? raw.length;

  const sanitized = sanitizeInput(raw);

  if (sanitized !== raw) {
    const beforeCaretRaw = raw.slice(0, caret);
    const beforeCaretSanitized = sanitizeInput(beforeCaretRaw);
    const removedBeforeCaret = beforeCaretRaw.length - beforeCaretSanitized.length;

    target.value = sanitized;

    const newCaret = Math.max(0, caret - removedBeforeCaret);
    target.setSelectionRange(newCaret, newCaret);
  }

  inputSite.value = sanitized;
  recentSitesDismissed.value = false;
  clearValidationError();
};

/**
 * @description 处理剪贴板输入
 * @param value 剪贴板输入
 */
const handleClipboard = (value: string) => {
  inputSite.value = normalizeSite(value);
  recentSitesDismissed.value = false;
};

/**
 * @description 处理确认输入
 */
const handleConfirm = async () => {
  if (loginBtn.value) return;

  errorMessage.value = "";
  hasValidationError.value = false;

  const normalizedSite = normalizedInputSite.value;
  const urlRegExp = appConfig.componentsConfig.urlRegExp;

  if (!normalizedSite) {
    hasValidationError.value = true;
    errorMessage.value = t("Login.EmptyUrlError");

    nextTick(() => {
      inputRef.value?.$el?.querySelector("input")?.focus();
    });

    return;
  }

  const users = Object.values(userMap.value) as UserData[];

  if (users.some((user) => normalizeSite(user.site) === normalizedSite)) {
    hasValidationError.value = true;
    errorMessage.value = t("Login.AlreadyLoggedInError");

    return;
  }

  if (!urlRegExp.test(normalizedSite)) {
    hasValidationError.value = true;
    errorMessage.value = t("Login.InvalidUrlError");

    nextTick(() => {
      inputRef.value?.$el?.querySelector("input")?.focus();
    });

    return;
  }

  try {
    clearLoginBtnUnlockTimer();
    loginBtn.value = true;
    const ok = await checkVersionBeforeOAuth(normalizedSite);
    if (!ok) return;

    await useTauriCoreInvoke("auth_login", {
      site: normalizedSite
    });
    void saveRecentSite(normalizedSite);
  } catch (e: any) {
    const raw = (e?.message || e || "").toString();
    const looksLikeSiteIssue = [
      "Failed to fetch OAuth config",
      "OAuth config endpoint returned",
      "Failed to parse OAuth config JSON",
      "Failed to read response body"
    ].some((needle) => raw.includes(needle));

    hasValidationError.value = true;
    errorMessage.value = looksLikeSiteIssue ? t("Login.InvalidSiteError") : raw || t("Login.LoginFailed");

    if (!looksLikeSiteIssue) {
      toast.add({
        title: t("Login.LoginFailed"),
        description: raw || t("Login.LoginFailed"),
        color: "error",
        icon: "line-md:close-circle",
        progress: true,
        duration: 4000
      });
      enableLoginBtnAfter(2000);
    }

    nextTick(() => {
      inputRef.value?.$el?.querySelector("input")?.focus();
    });
  }
};

onMounted(async () => {
  applyCurrentThemeColor();

  const unlisten = await useTauriEventListen("auth_url", (event) => {
    const url = (event?.payload || "").toString();
    if (!url) return;

    clearLoginBtnUnlockTimer();
    loginBtn.value = false;
    openModal.value = false;
    navigateTo({ path: localePath({ path: "/auth/browser" }), query: { auth_url: url } });
    if (url && typeof url === "string") {
      useTauriOpenerOpenUrl(url);
    }
    unlisten?.();
  });

  unlistenErrorPageRef.value = await useTauriEventListen("error-page", (event) => {
    const payload = (event?.payload || {}) as any;
    const status = (payload?.status || "").toString();
    const reason = (payload?.reason || "").toString();
    const message = (payload?.message || "").toString();

    if (status !== "failure") return;

    let description = message || t("Login.LoginFailedErrorPage");

    if (reason === "invalid-site") {
      description = t("Login.InvalidSiteError");
    }

    toast.add({
      title: t("Login.LoginFailed"),
      description,
      color: "error",
      icon: "line-md:close-circle",
      progress: true,
      duration: 4000
    });
    enableLoginBtnAfter(2000);

    if (reason === "invalid-site") {
      hasValidationError.value = true;
      errorMessage.value = t("Login.InvalidSiteError");

      nextTick(() => {
        inputRef.value?.$el?.querySelector("input")?.focus();
      });

      return;
    }

    nextTick(() => {
      userInfoStore.setUserLoggedIn(false);
    });
  });

  unlistenLoginFailedRef.value = await useTauriEventListen("login-failed-detected", (event) => {
    const payload = (event?.payload || {}) as any;
    const reason = (payload?.reason || "").toString();
    const message = (payload?.message || "").toString();

    let description = message || t("Login.LoginFailedDescription");

    if (reason === "invalid-site") {
      description = t("Login.InvalidSiteError");
    }

    toast.add({
      title: t("Login.LoginFailed"),
      description,
      color: "error",
      icon: "line-md:close-circle",
      progress: true,
      duration: 4000
    });

    enableLoginBtnAfter(2000);

    if (reason === "invalid-site") {
      hasValidationError.value = true;
      errorMessage.value = t("Login.InvalidSiteError");

      nextTick(() => {
        inputRef.value?.$el?.querySelector("input")?.focus();
      });

      return;
    }

    userInfoStore.setUserLoggedIn(false);
  });
});

onBeforeUnmount(() => {
  if (unlistenErrorPageRef.value) unlistenErrorPageRef.value();
  if (unlistenLoginFailedRef.value) unlistenLoginFailedRef.value();
  clearLoginBtnUnlockTimer();
});
</script>

<template>
  <UDropdownMenu
    :items="profileMenuItems"
    size="sm"
    side="top"
    align="start"
    :ui="{ content: 'w-56 p-1' }"
  >
    <div
      class="flex items-center py-2 px-2 w-full min-w-0 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 transition-colors mb-1"
      :style="{
        justifyContent: collapse ? 'center' : ''
      }"
    >
      <UUser
        size="sm"
        :avatar="{
          src: '/user_avatar.png'
        }"
        :ui="props.collapse ? { root: 'justify-center gap-0' } : undefined"
      >
        <template #name>
          <UTooltip v-if="!props.collapse" arrow :text="profileDisplayName">
            <span class="block md:max-w-[150px] truncate leading-tight text-sm font-medium cursor-pointer">
              {{ profileDisplayName }}
            </span>
          </UTooltip>
        </template>
      </UUser>
    </div>
  </UDropdownMenu>

  <Modal
    v-model:open="openModal"
    :title="t('Login.Title')"
    :disabled="loginBtn"
    @update:open="openModal = $event"
    @confirm="handleConfirm"
    @clipboard="handleClipboard"
  >
    <div class="space-y-1">
      <UInput
        ref="inputRef"
        v-model="inputSite"
        :color="hasValidationError ? 'error' : 'primary'"
        :ui="{ base: 'peer' }"
        placeholder=" "
        autocapitalize="none"
        autocorrect="off"
        @input="handleInputSanitize"
      >
        <label
          class="pointer-events-none absolute left-0 -top-2.5 text-xs font-medium px-1.5 transition-all peer-focus:-top-2.5 peer-focus:text-xs peer-focus:font-medium peer-placeholder-shown:text-sm peer-placeholder-shown:top-1.5 peer-placeholder-shown:font-normal"
        >
          <span class="inline-flex bg-default px-1">
            {{ t("Login.Description") }}
          </span>
        </label>

        <template v-if="normalizedInputSite?.length" #trailing>
          <UButton
            color="neutral"
            variant="link"
            size="sm"
            icon="i-lucide-circle-x"
            aria-label="Clear input"
            @click="handleClearInput"
          />
        </template>
      </UInput>

      <RecentSites
        :visible="showRecentSites"
        :sites="filteredRecentSites"
        @select="selectRecentSite"
        @remove="removeRecentSite"
        @clear="clearRecentSites"
      />

      <div v-if="hasValidationError" class="text-red-500 text-xs px-1">
        {{ errorMessage }}
      </div>
    </div>
  </Modal>
</template>

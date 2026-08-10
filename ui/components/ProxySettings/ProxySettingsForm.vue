<script setup lang="ts">
import type { ProxyType } from "~/types/proxy-settings";

import { parseBypassEntries, useProxySettings, validateJumpServerOrigin } from "~/composables/useProxySettings";

const { t } = useI18n();
const { recentSites, hydrationPromise } = useSettingManager();
const {
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
} = useProxySettings();

const proxyTypeItems = computed(() => [
  { label: "HTTP", value: "http" as ProxyType },
  { label: "SOCKS5", value: "socks5" as ProxyType }
]);

const proxyEnabled = computed({
  get: () => settings.mode === "manual",
  set: (enabled: boolean) => {
    settings.mode = enabled ? "manual" : "direct";
  }
});
const proxyEnableDescription = computed(() =>
  proxyEnabled.value ? t("Proxy.EnabledDescription") : t("Proxy.DisabledDescription")
);
const isBusy = computed(() => isLoading.value || isSaving.value || isTesting.value);
const passwordHelp = computed(() => {
  if (clearPassword.value) return t("Proxy.PasswordWillClear");
  if (settings.hasPassword && !password.value) return t("Proxy.PasswordKeepHint");
  return t("Proxy.PasswordOptionalHint");
});
const formError = shallowRef("");
const testDetailMessage = computed(() => {
  const result = testResult.value;
  if (!result?.message || result.message === `HTTP ${result.status}`) return "";
  return result.message;
});

function validateBypass() {
  const entries = parseBypassEntries(bypassText.value);
  const invalid = entries.some(
    (entry) =>
      /\s/.test(entry) || entry.includes("://") || entry.includes("@") || entry.includes("?") || entry.includes("#")
  );
  if (bypassText.value.length > 32768) errors.bypass = t("Proxy.Errors.BypassTextTooLong");
  else if (entries.length > 128) errors.bypass = t("Proxy.Errors.TooManyBypassEntries");
  else if (entries.some((entry) => entry.length > 255)) errors.bypass = t("Proxy.Errors.BypassEntryTooLong");
  else errors.bypass = invalid ? t("Proxy.Errors.InvalidBypass") : "";
}

function validateManualSettings() {
  validateBypass();
  if (settings.mode !== "manual") return;

  const host = settings.host.trim();
  if (!host) errors.host = t("Proxy.Errors.HostRequired");
  else if (host.length > 255) errors.host = t("Proxy.Errors.HostTooLong");
  else if (/[\s/?#@]|:\/\//.test(host)) errors.host = t("Proxy.Errors.InvalidHost");

  const port = Number(settings.port);
  if (!Number.isInteger(port) || port < 1 || port > 65535) errors.port = t("Proxy.Errors.InvalidPort");

  const hasUsername = Boolean(settings.username.trim());
  if (settings.username.length > 256) errors.username = t("Proxy.Errors.UsernameTooLong");
  if (password.value.length > 1024) errors.password = t("Proxy.Errors.PasswordTooLong");

  const hasEffectivePassword = !clearPassword.value && Boolean(password.value || settings.hasPassword);
  if (!errors.password && hasUsername && !hasEffectivePassword) errors.password = t("Proxy.Errors.PasswordRequired");
  if (!errors.username && !hasUsername && hasEffectivePassword) errors.username = t("Proxy.Errors.UsernameRequired");
}

function validateTarget() {
  const value = targetUrl.value.trim();
  if (!value) errors.targetUrl = t("Proxy.Errors.TargetRequired");
  else if (value.length > 2048) errors.targetUrl = t("Proxy.Errors.TargetTooLong");
  else if (!validateJumpServerOrigin(value)) errors.targetUrl = t("Proxy.Errors.InvalidTargetOrigin");
}

function validateForSave() {
  clearErrors();
  validateManualSettings();
  return !errors.host && !errors.port && !errors.username && !errors.password && !errors.bypass;
}

function validateForTest() {
  clearErrors();
  validateManualSettings();
  validateTarget();
  return validateForSaveFields() && !errors.targetUrl;
}

function validateForSaveFields() {
  return !errors.host && !errors.port && !errors.username && !errors.password && !errors.bypass;
}

async function handleSave() {
  formError.value = "";
  if (!validateForSave()) {
    formError.value = errors.bypass || t("Proxy.Errors.FixFields");
    return;
  }
  await save();
}

async function handleTest() {
  formError.value = "";
  if (!validateForTest()) {
    formError.value = errors.bypass || t("Proxy.Errors.FixFields");
    return;
  }
  await test();
}

watch(
  [
    () => settings.mode,
    () => settings.proxyType,
    () => settings.host,
    () => settings.port,
    () => settings.username,
    password,
    clearPassword,
    bypassText
  ],
  () => {
    formError.value = "";
    saveStatus.value = null;
    testResult.value = null;
  },
  { flush: "sync" }
);

watch(
  () => settings.mode,
  (mode) => {
    clearErrors();
    if (mode !== "manual") {
      password.value = "";
      clearPassword.value = false;
    }
  },
  { flush: "sync" }
);

watch(
  targetUrl,
  () => {
    testResult.value = null;
  },
  { flush: "sync" }
);

watch(clearPassword, (shouldClear) => {
  if (shouldClear) password.value = "";
  errors.password = "";
  errors.username = "";
});

onMounted(async () => {
  await Promise.all([load(), hydrationPromise.value ?? Promise.resolve()]);

  const firstRecentSite = recentSites.value[0]?.trim();
  if (firstRecentSite) {
    targetUrl.value = validateJumpServerOrigin(firstRecentSite) ? new URL(firstRecentSite).origin : firstRecentSite;
  }
});
</script>

<template>
  <div class="mx-auto flex w-full max-w-3xl flex-col gap-5 p-4 sm:p-5" :aria-busy="isLoading">
    <header class="space-y-1">
      <h1 class="text-base font-semibold text-highlighted">
        {{ t("Proxy.Title") }}
      </h1>
      <p class="text-sm leading-5 text-muted">
        {{ t("Proxy.ScopeIncluded") }}
      </p>
      <p class="text-xs leading-5 text-dimmed">
        {{ t("Proxy.ScopeExcluded") }}
      </p>
    </header>

    <div v-if="loadError" class="space-y-3">
      <UAlert
        color="error"
        variant="subtle"
        icon="i-lucide-circle-alert"
        :title="t('Proxy.LoadFailed')"
        :description="loadError"
        role="alert"
      />
      <UButton
        type="button"
        color="neutral"
        variant="outline"
        icon="i-lucide-refresh-cw"
        :label="t('Proxy.Retry')"
        :loading="isLoading"
        @click="load"
      />
    </div>

    <UAlert
      v-else-if="settings.warning"
      color="warning"
      variant="subtle"
      icon="i-lucide-triangle-alert"
      :title="t('Proxy.ConfigRecovered')"
      :description="settings.warning"
      role="status"
    />

    <div v-if="isLoading" class="flex items-center gap-2 py-4 text-sm text-muted" role="status">
      <UIcon name="i-lucide-loader-circle" class="size-4 animate-spin" aria-hidden="true" />
      <span>{{ t("Proxy.Loading") }}</span>
    </div>

    <form v-else-if="!loadError" class="flex flex-col gap-5" novalidate @submit.prevent="handleSave">
      <section aria-labelledby="proxy-enabled-label">
        <USwitch
          id="proxy-enabled"
          v-model="proxyEnabled"
          name="proxy-enabled"
          :label="t('Proxy.EnableProxy')"
          :disabled="isBusy"
          aria-describedby="proxy-enabled-description"
          class="w-full"
          :ui="{
            root: 'w-full',
            wrapper: 'min-w-0',
            label: 'text-sm font-medium',
            description: 'leading-5'
          }"
        >
          <template #label>
            <span id="proxy-enabled-label">{{ t("Proxy.EnableProxy") }}</span>
          </template>
          <template #description>
            <span id="proxy-enabled-description">{{ proxyEnableDescription }}</span>
          </template>
        </USwitch>
      </section>

      <template v-if="settings.mode === 'manual'">
        <USeparator />

        <section class="space-y-4" aria-labelledby="manual-proxy-heading">
          <h2 id="manual-proxy-heading" class="text-sm font-medium text-highlighted">
            {{ t("Proxy.ManualSettings") }}
          </h2>

          <URadioGroup
            v-model="settings.proxyType"
            :items="proxyTypeItems"
            value-key="value"
            :legend="t('Proxy.ProxyType')"
            variant="table"
            indicator="hidden"
            orientation="horizontal"
            :disabled="isBusy"
            class="max-w-64"
            :ui="{
              legend: 'mb-1.5 text-xs',
              fieldset: 'w-full',
              item: 'flex-1 justify-center px-3 py-2 has-data-[state=checked]:bg-primary/10',
              label: 'text-center'
            }"
          />

          <div class="grid gap-4 sm:grid-cols-[minmax(0,1fr)_9rem]">
            <UFormField name="proxy-host" :label="t('Proxy.Host')" :error="errors.host || false" required>
              <UInput
                id="proxy-host"
                v-model="settings.host"
                autocomplete="off"
                spellcheck="false"
                maxlength="255"
                required
                :placeholder="t('Proxy.HostPlaceholder')"
                :disabled="isBusy"
                :aria-invalid="Boolean(errors.host)"
                @input="errors.host = ''"
              />
            </UFormField>

            <UFormField name="proxy-port" :label="t('Proxy.Port')" :error="errors.port || false" required>
              <UInput
                id="proxy-port"
                v-model.number="settings.port"
                type="number"
                inputmode="numeric"
                min="1"
                max="65535"
                required
                placeholder="8080"
                :disabled="isBusy"
                :aria-invalid="Boolean(errors.port)"
                @input="errors.port = ''"
              />
            </UFormField>
          </div>

          <div class="grid gap-4 sm:grid-cols-2">
            <UFormField name="proxy-username" :label="t('Proxy.Username')" :error="errors.username || false">
              <UInput
                id="proxy-username"
                v-model="settings.username"
                autocomplete="username"
                maxlength="256"
                :placeholder="t('Proxy.Optional')"
                :disabled="isBusy"
                :aria-invalid="Boolean(errors.username)"
                @input="errors.username = ''"
              />
            </UFormField>

            <UFormField
              name="proxy-password"
              :label="t('Proxy.Password')"
              :help="errors.password ? undefined : passwordHelp"
              :error="errors.password || false"
            >
              <template #label>
                <span class="inline-flex items-center gap-1.5">
                  {{ t("Proxy.Password") }}
                  <span
                    v-if="settings.hasPassword"
                    class="inline-flex items-center gap-1 text-xs font-normal text-success"
                  >
                    <UIcon name="i-lucide-circle-check" class="size-3.5" aria-hidden="true" />
                    {{ t("Proxy.PasswordSaved") }}
                  </span>
                </span>
              </template>
              <UInput
                id="proxy-password"
                v-model="password"
                type="password"
                autocomplete="new-password"
                maxlength="1024"
                :placeholder="settings.hasPassword ? t('Proxy.PasswordKeepPlaceholder') : t('Proxy.Optional')"
                :disabled="isBusy || clearPassword"
                :aria-invalid="Boolean(errors.password)"
                @input="errors.password = ''"
              />
            </UFormField>
          </div>

          <UCheckbox
            v-if="settings.hasPassword"
            v-model="clearPassword"
            :label="t('Proxy.ClearPassword')"
            :description="t('Proxy.ClearPasswordHint')"
            :disabled="isBusy"
          />

          <UFormField
            name="proxy-bypass"
            :label="t('Proxy.Bypass')"
            :description="t('Proxy.BypassDescription')"
            :error="errors.bypass || false"
          >
            <UTextarea
              id="proxy-bypass"
              v-model="bypassText"
              :rows="3"
              autocomplete="off"
              spellcheck="false"
              maxlength="32768"
              :placeholder="t('Proxy.BypassPlaceholder')"
              :disabled="isBusy"
              :aria-invalid="Boolean(errors.bypass)"
              @input="errors.bypass = ''"
            />
          </UFormField>
        </section>
      </template>

      <USeparator />

      <section class="space-y-3" aria-labelledby="proxy-test-heading">
        <div class="space-y-1">
          <h2 id="proxy-test-heading" class="text-sm font-medium text-highlighted">
            {{ t("Proxy.TestTitle") }}
          </h2>
          <p class="text-xs leading-5 text-muted">
            {{ t("Proxy.TestDescription") }}
          </p>
        </div>

        <div class="flex flex-col items-start gap-3 sm:flex-row sm:items-start">
          <UFormField
            name="proxy-test-target"
            :label="t('Proxy.TargetUrl')"
            :error="errors.targetUrl || false"
            class="flex-1"
            required
          >
            <UInput
              id="proxy-test-target"
              v-model="targetUrl"
              type="url"
              inputmode="url"
              autocomplete="url"
              spellcheck="false"
              maxlength="2048"
              required
              placeholder="https://jumpserver.example.com"
              :disabled="isBusy"
              :aria-invalid="Boolean(errors.targetUrl)"
              @input="errors.targetUrl = ''"
            />
          </UFormField>
          <UButton
            type="button"
            color="neutral"
            variant="outline"
            icon="i-lucide-plug-zap"
            :label="t('Proxy.TestConnection')"
            :loading="isTesting"
            :disabled="isBusy"
            class="sm:mt-6"
            @click="handleTest"
          />
        </div>

        <div aria-live="polite" aria-atomic="true">
          <div
            v-if="testResult"
            class="flex items-start gap-2 rounded-sm border px-3 py-2 text-sm"
            :class="
              testResult.success
                ? 'border-success/30 bg-success/5 text-success'
                : 'border-error/30 bg-error/5 text-error'
            "
          >
            <UIcon
              :name="testResult.success ? 'i-lucide-circle-check' : 'i-lucide-circle-x'"
              class="mt-0.5 size-4 shrink-0"
              aria-hidden="true"
            />
            <p class="flex min-w-0 flex-wrap gap-x-1 break-words">
              <span class="font-medium">
                {{ testResult.success ? t("Proxy.TestSucceeded") : t("Proxy.TestFailed") }}
              </span>
              <span v-if="testResult.status">HTTP {{ testResult.status }}</span>
              <span>{{ testResult.elapsedMs }} ms</span>
              <span v-if="testDetailMessage">{{ testDetailMessage }}</span>
            </p>
          </div>
        </div>
      </section>

      <div class="flex items-center justify-between gap-3 border-t border-default pt-4">
        <div class="min-h-5 min-w-0 text-sm">
          <p v-if="formError" class="break-words text-error" role="alert">
            {{ formError }}
          </p>
          <p
            v-else-if="saveStatus?.kind === 'success'"
            class="inline-flex items-center gap-1.5 text-success"
            role="status"
          >
            <UIcon name="i-lucide-circle-check" class="size-4" aria-hidden="true" />
            {{ t("Proxy.SaveSucceeded") }}
          </p>
          <p v-else-if="saveStatus?.kind === 'error'" class="break-words text-error" role="alert">
            {{
              saveStatus.stage === "readback"
                ? t("Proxy.ReadbackFailedWithReason", { reason: saveStatus.message })
                : t("Proxy.SaveFailedWithReason", { reason: saveStatus.message })
            }}
          </p>
        </div>
        <UButton
          type="submit"
          icon="i-lucide-save"
          :label="t('Proxy.Save')"
          :loading="isSaving"
          :disabled="isBusy"
          class="shrink-0"
        />
      </div>
    </form>
  </div>
</template>

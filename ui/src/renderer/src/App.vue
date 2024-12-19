<template>
  <n-config-provider
    :locale="defaultLang === 'zh' ? zhCN : enUS"
    :theme="defaultTheme === 'dark' ? darkTheme : lightTheme"
    :class="defaultTheme === 'dark' ? 'theme-dark' : 'theme-light'"
    :themeOverrides="defaultTheme === 'dark' ? darkThemeOverrides : lightThemeOverrides"
  >
    <n-modal-provider>
      <n-message-provider>
        <div class="custom-header ele_drag bg-primary border-b-primary border-b">
          <div class="logo">
            <img :src="iconImage" alt="" />
            <span class="title text-primary">JumpServer Client</span>
          </div>
        </div>
        <LoginModal :show-modal="showModal" @close-mask="handleCloseMask" />
        <router-view />
      </n-message-provider>
    </n-modal-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { getProfile } from './api/modals/user';
import { useUserStore } from './store/module/userStore';
import { darkThemeOverrides, lightThemeOverrides } from './overrides';
import { computed, watch, onBeforeUnmount, onMounted, ref } from 'vue';
import { darkTheme, enUS, zhCN, lightTheme, createDiscreteApi } from 'naive-ui';

import { useElectronConfig } from './hooks/useElectronConfig';
import { getIconImage, getAvatarImage } from './utils/common';

import type { ConfigProviderProps } from 'naive-ui';

import mittBus from './eventBus';
import LoginModal from './components/LoginModal/index.vue';

const { t, locale } = useI18n();
const { getDefaultSetting, setDefaultSetting } = useElectronConfig();

const router = useRouter();
const userStore = useUserStore();

const showModal = ref(false);
const defaultLang = ref('');
const defaultTheme = ref('');
const iconImage = ref<string | null>(null);
const avatarImage = ref<string | null>(null);

const configProviderPropsRef = computed<ConfigProviderProps>(() => ({
  theme: defaultTheme.value === 'light' ? lightTheme : darkTheme
}));

watch(
  () => defaultLang.value,
  nv => {
    if (nv) {
      locale.value = nv;
    }
  }
);

const { notification } = createDiscreteApi(['notification'], {
  configProviderProps: configProviderPropsRef
});

/**
 * @description 切换语言
 */
const handleLangChange = async (lang: string) => {
  switch (lang) {
    case 'zh': {
      defaultLang.value = 'en';
      break;
    }
    case 'en': {
      defaultLang.value = 'zh';
      break;
    }
  }

  setDefaultSetting({ language: defaultLang.value });
};

/**
 * @description 切换主题
 * @param theme
 */
const handleThemeChange = async (theme: string) => {
  switch (theme) {
    case 'light': {
      defaultTheme.value = 'dark';
      break;
    }
    case 'dark': {
      defaultTheme.value = 'light';
      break;
    }
  }

  setDefaultSetting({ theme: defaultTheme.value });
};

/**
 * @description 添加账号
 */
const handleAddAccount = () => {
  showModal.value = true;
};

/**
 * @description 移除账号
 */
const handleRemoveAccount = () => {
  userStore.removeCurrentUser();

  const userInfo = userStore.userInfo;

  if (userInfo && userInfo.length > 0) {
    userStore.setCurrentUser({
      ...userInfo[0]
    });

    mittBus.emit('search');
  } else {
    showModal.value = true;
  }
};

/**
 * @description 关闭遮罩
 */
const handleCloseMask = () => {
  showModal.value = !showModal.value;
};

onMounted(async () => {
  try {
    iconImage.value = await getIconImage();
    avatarImage.value = await getAvatarImage();

    const { theme, language } = await getDefaultSetting();

    defaultTheme.value = theme;
    defaultLang.value = language;

    // 如果有 token 才去验证
    if (userStore.token) {
      try {
        await getProfile();
      } catch (e: any) {
        const status = e.response?.status;

        if (status === 401 || status === 403) {
          userStore.setToken('');
          showModal.value = true;
        }
      }
    }
  } catch (e) {
    notification.create({
      type: 'info',
      content: t('Message.GetDefaultSettingFailed'),
      duration: 2000
    });
  }

  // 检查是否需要显示登录框
  if (!userStore.token || (userStore.userInfo && userStore.userInfo.length <= 0)) showModal.value = true;

  window.electron.ipcRenderer.on('set-token', async (_e, token: string) => {
    if (token) {
      showModal.value = false;
      userStore.setToken(token);

      try {
        const res = await getProfile();

        userStore.setUserInfo({
          token,
          username: res?.username,
          display_name: res?.system_roles.map((item: any) => item.display_name),
          avatar_url: avatarImage.value,
          currentSite: userStore.currentSite
        });

        userStore.setCurrentUser({
          token,
          username: res?.username,
          display_name: res?.system_roles.map((item: any) => item.display_name),
          avatar_url: avatarImage.value,
          currentSite: userStore.currentSite
        });

        if (res) {
          notification.create({
            type: 'success',
            content: t('Message.AuthenticatedSuccess'),
            duration: 2000
          });

          mittBus.emit('search');
          await router.push({ name: 'Linux' });
        }
      } catch (e) {
        showModal.value = false;
      }
    }
  });

  mittBus.on('addAccount', handleAddAccount);
  mittBus.on('changeLang', handleLangChange);
  mittBus.on('changeTheme', handleThemeChange);
  mittBus.on('removeAccount', handleRemoveAccount);
});

onBeforeUnmount(() => {
  mittBus.off('addAccount', handleAddAccount);
  mittBus.off('changeLang', handleLangChange);
  mittBus.off('changeTheme', handleThemeChange);
  mittBus.off('removeAccount', handleRemoveAccount);
});
</script>

<style scoped lang="scss">
.n-config-provider {
  height: 100%;
  width: 100%;
}
</style>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { darkThemeOverrides, lightThemeOverrides } from './overrides';
import type { ConfigProviderProps } from 'naive-ui';
import { createDiscreteApi, darkTheme, enUS, lightTheme, zhCN } from 'naive-ui';

import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { getProfile } from '@renderer/api/modals/user';
import { useUserStore } from '@renderer/store/module/userStore';
import { getSystemSetting } from '@renderer/api/modals/setting';
import { useSettingStore } from '@renderer/store/module/settingStore';

import { Conf } from 'electron-conf/renderer';

import mittBus from '@renderer/eventBus';
import LoginModal from '@renderer/components/LoginModal/index.vue';

const conf = new Conf();
const router = useRouter();
const { t, locale } = useI18n();

const iconImage = ref('');
const defaultLang = ref('');
const defaultTheme = ref('');
const showModal = ref(false);

const userStore = useUserStore();
const settingStore = useSettingStore();

let avatarImage: string;

conf.get('defaultSetting').then(res => {
  if (res) {
    // @ts-ignore
    defaultTheme.value = res?.theme;
    // @ts-ignore
    defaultLang.value = res?.language;
  }
});

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
  const currentSettings = (await conf.get('defaultSetting')) as Record<string, any>;

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

  await conf.set('defaultSetting', {
    ...currentSettings,
    language: defaultLang.value
  });
};

/**
 * @description 切换主题
 * @param theme
 */
const handleThemeChange = async (theme: string) => {
  const currentSettings = (await conf.get('defaultSetting')) as Record<string, any>;

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
  window.electron.ipcRenderer.send('update-titlebar-overlay', defaultTheme.value);
  await conf.set('defaultSetting', {
    ...currentSettings,
    theme: defaultTheme.value
  });
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
 * @description 获取 Logo
 */
const getIconImage = async () => {
  const res = await import('@renderer/assets/Logo.svg');

  iconImage.value = res.default;
};

/**
 * @description 获取头像
 */
const getAvatarImage = async () => {
  const res = await import('@renderer/assets/avatar.png');

  avatarImage = res.default;
};

/**
 * @description 关闭遮罩
 */
const handleCloseMask = () => {
  showModal.value = !showModal.value;
};

onMounted(async () => {
  await getIconImage();
  await getAvatarImage();

  try {
    const res = await getProfile();

    if (res) {
      notification.create({
        type: 'success',
        content: t('Message.AuthenticatedSuccess'),
        duration: 2000
      });

      await router.push({ name: 'Linux' });
    }

    const setting = await getSystemSetting();
    if (setting) {
      settingStore.setRdpClientOption(setting.graphics.rdp_client_option);
      settingStore.setKeyboardLayout(setting.graphics.keyboard_layout);
      settingStore.setRdpSmartSize(setting.graphics.rdp_smart_size);
      settingStore.setRdpColorQuality(setting.graphics.rdp_color_quality);
    }
  } catch (e: any) {
    const status = e.response?.status;

    if (status === 401 || status === 403) {
      userStore.setToken('');
      showModal.value = true;
    }
  }

  // @ts-ignore
  if (userStore?.userInfo.length <= 0) showModal.value = true;

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
          avatar_url: avatarImage,
          currentSite: userStore.currentSite
        });

        userStore.setCurrentUser({
          token,
          username: res?.username,
          display_name: res?.system_roles.map((item: any) => item.display_name),
          avatar_url: avatarImage,
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

          const setting = await getSystemSetting();
          if (setting) {
            settingStore.setRdpClientOption(setting.graphics.rdp_client_option);
            settingStore.setKeyboardLayout(setting.graphics.keyboard_layout);
            settingStore.setRdpSmartSize(setting.graphics.rdp_smart_size);
            settingStore.setRdpColorQuality(setting.graphics.rdp_color_quality);
          }
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

<style scoped lang="scss">
.n-config-provider {
  height: 100%;
  width: 100%;
}
</style>

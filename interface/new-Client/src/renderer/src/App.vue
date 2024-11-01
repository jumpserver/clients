<script setup lang="ts">
import { darkThemeOverrides, lightThemeOverrides } from './overrides';
import { darkTheme, lightTheme, zhCN, enUS } from 'naive-ui';

import { useUserStore } from '@renderer/store/module/userStore';
import { onBeforeUnmount, ref, onMounted, nextTick } from 'vue';
import { getProfile } from '@renderer/api/modals/user';
import { useMessage } from 'naive-ui';
import { Conf } from 'electron-conf/renderer';

import mittBus from '@renderer/eventBus';

import LoginModal from '@renderer/components/LoginModal/index.vue';

const conf = new Conf();

const iconImage = ref('');
const defaultLang = ref('');
const defaultTheme = ref('');
const showModal = ref(false);

const message = useMessage();
const userStore = useUserStore();

conf.get('defaultSetting').then(res => {
  if (res) {
    // @ts-ignore
    defaultTheme.value = res?.theme;
    // @ts-ignore
    defaultLang.value = res?.language;

    return;
  }
});

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

  await conf.set('defaultSetting', {
    ...currentSettings,
    theme: defaultTheme.value
  });
};

/**
 * @description 获取 Logo
 */
const getIconImage = async () => {
  const res = await import('@renderer/assets/Logo.svg');

  iconImage.value = res.default;
};

const getValidate = async () => {
  try {
    await getProfile();

    //todo)) 设置 userInfo
  } catch (e: any) {
    const status = e.response.status;

    if (status === 401 || status === 403) {
      userStore.setToken('');
      showModal.value = true;
    }
  }
};

onMounted(async () => {
  await getIconImage();
  mittBus.on('changeTheme', handleThemeChange);

  await getValidate();

  window.electron.ipcRenderer.on('set-token', (_e, token: string) => {
    if (token) {
      showModal.value = false;
      userStore.setToken(token);

      nextTick(async () => {
        try {
          const res = await getProfile();

          userStore.setUserInfo({
            username: res?.username,
            display_name: res?.system_roles.map((item: any) => item.display_name),
            avatar_url: res?.avatar_url
          });

          if (res) {
            message.success('您已登录认证成功!');
          }
        } catch (e) {
          showModal.value = false;
        }
      });
    }
  });
});

onBeforeUnmount(() => {
  mittBus.off('changeTheme', handleThemeChange);
});
</script>

<template>
  <n-config-provider
    :locale="defaultLang === 'zhCN' ? zhCN : enUS"
    :theme="defaultTheme === 'dark' ? darkTheme : lightTheme"
    :class="defaultTheme === 'dark' ? 'theme-dark' : 'theme-light'"
    :themeOverrides="defaultTheme === 'dark' ? darkThemeOverrides : lightThemeOverrides"
  >
    <n-modal-provider>
      <n-message-provider placement="top-right">
        <div class="custom-header ele_drag bg-primary border-b-primary border-b">
          <div class="logo">
            <img :src="iconImage" alt="" />
            <span class="title text-primary">JumpServer Client</span>
          </div>
        </div>
        <LoginModal :show-modal="showModal" />
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

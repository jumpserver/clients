<script setup lang="ts">
import { darkThemeOverrides, lightThemeOverrides } from './overrides';
import { darkTheme, enUS, lightTheme, useMessage, zhCN } from 'naive-ui';

import { useUserStore } from '@renderer/store/module/userStore';
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { getProfile } from '@renderer/api/modals/user';
import { Conf } from 'electron-conf/renderer';

import mittBus from '@renderer/eventBus';

import LoginModal from '@renderer/components/LoginModal/index.vue';

const conf = new Conf();

const iconImage = ref('');
const defaultLang = ref('');
const defaultTheme = ref('');
const showModal = ref(false);

const userStore = useUserStore();

let avatarImage: string;

conf.get('defaultSetting').then(res => {
  if (res) {
    // @ts-ignore
    defaultTheme.value = res?.theme;
    // @ts-ignore
    defaultLang.value = res?.language;
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
 * @description 添加账号
 */
const handleAddAccount = () => {
  showModal.value = true;
};

/**
 * @description 移除账号
 */
const handleRemoveAccount = () => {
  const userInfo = userStore.userInfo;

  userStore.removeCurrentUser();

  if (userInfo && userInfo.length > 0) {
    userStore.setCurrentUser(userInfo[0]);
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
      const message = useMessage();
      message.success('您已登录认证成功!');
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

  window.electron.ipcRenderer.on('set-token', (_e, token: string) => {
    if (token) {
      showModal.value = false;
      userStore.setToken(token);

      nextTick(async () => {
        try {
          const res = await getProfile();

          userStore.setUserInfo({
            token,
            username: res?.username,
            display_name: res?.system_roles.map((item: any) => item.display_name),
            avatar_url: avatarImage
          });

          userStore.setCurrentUser({
            token,
            username: res?.username,
            display_name: res?.system_roles.map((item: any) => item.display_name),
            avatar_url: avatarImage
          });

          if (res) {
            const message = useMessage();
            message.success('您已登录认证成功!');
          }
        } catch (e) {
          showModal.value = false;
        }
      });
    }
  });

  mittBus.on('addAccount', handleAddAccount);
  mittBus.on('removeAccount', handleRemoveAccount);
  mittBus.on('changeTheme', handleThemeChange);
});

onBeforeUnmount(() => {
  mittBus.off('addAccount', handleAddAccount);
  mittBus.off('removeAccount', handleRemoveAccount);
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

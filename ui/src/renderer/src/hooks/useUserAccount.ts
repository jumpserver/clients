import { useI18n } from 'vue-i18n';
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { getProfile } from '@renderer/api/modals/user';
import { useElectronConfig } from './useElectronConfig';
import { getAvatarImage } from '@renderer/utils/common';
import { useUserStore } from '@renderer/store/module/userStore';
import { createDiscreteApi, lightTheme, darkTheme } from 'naive-ui';

import type { ConfigProviderProps } from 'naive-ui';
import type { IUserInfo, IOrginization } from '@renderer/store/interface';

export const useUserAccount = () => {
  const { t } = useI18n();
  const router = useRouter();
  const userStore = useUserStore();

  const defaultTheme = ref('');
  const showLoginModal = ref(false);

  const init = async () => {
    const { getDefaultSetting } = useElectronConfig();

    const { theme } = await getDefaultSetting();

    defaultTheme.value = theme;
  };

  /**
   * @description 设置新账号
   */
  const setNewAccount = () => {
    showLoginModal.value = true;
  };

  /**
   * @description 移除账号
   */
  const removeAccount = () => {
    // 移除之前的用户信息
    userStore.removeCurrentUser();

    if (userStore.userInfo && userStore.userInfo.length === 0) {
      return userStore.reset();
    }

    if (userStore.userInfo && userStore.userInfo.length > 0) {
      const firstUser = userStore.userInfo[0];

      userStore.setToken(firstUser.token);
      userStore.setCurrentUser({ ...firstUser });
      userStore.setCurrentSit(firstUser.currentSite as string);
    }
  };

  /**
   * @description 切换账号
   */
  const switchAccount = (token: string) => {
    if (token === userStore.token) {
      return;
    }

    if (userStore.userInfo) {
      const user = userStore.userInfo.find((item: IUserInfo) => item.token === token);

      if (user) {
        userStore.setToken(user.token);
        userStore.setCurrentUser({ ...user });
        userStore.setCurrentSit(user.currentSite as string);
      }
    }
  };

  /**
   * @description 获取账号信息
   */
  const getAccountInfo = () => {};

  /**
   * @description 处理 token 接收
   */
  const handleTokenReceived = async (token: string) => {
    if (!token) {
      useMessage.error('Token is required');
      return;
    }

    userStore.setToken(token);

    try {
      const res = await getProfile();

      console.log(res.audit_orgs);

      if (res) {
        notification.create({
          type: 'success',
          content: t('Message.AuthenticatedSuccess'),
          duration: 2000
        });

        userStore.setUserInfo({
          token,
          username: res?.username,
          display_name: res?.system_roles.map((item: any) => item.display_name),
          avatar_url: await getAvatarImage(),
          currentSite: userStore.currentSite
        });

        userStore.setCurrentUser({
          token,
          username: res?.username,
          display_name: res?.system_roles.map((item: any) => item.display_name),
          avatar_url: await getAvatarImage(),
          currentSite: userStore.currentSite
        });

        res.audit_orgs.forEach((org: IOrginization) => {
          userStore.setOrginization(org);
        });

        showLoginModal.value = false;

        router.push({ name: 'Linux' });
      }
    } catch (e) {
      showLoginModal.value = false;
    }
  };

  const handleModalOpacity = () => {
    showLoginModal.value = !showLoginModal.value;
  };

  init();

  const configProviderPropsRef = computed<ConfigProviderProps>(() => ({
    theme: defaultTheme.value === 'light' ? lightTheme : darkTheme
  }));

  const { message: useMessage, notification } = createDiscreteApi(['message', 'notification'], {
    configProviderProps: configProviderPropsRef
  });

  return {
    showLoginModal: showLoginModal,
    setNewAccount,
    switchAccount,
    removeAccount,
    getAccountInfo,
    handleModalOpacity,
    handleTokenReceived
  };
};

import { useI18n } from 'vue-i18n';
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useElectronConfig } from './useElectronConfig';
import { getAvatarImage } from '@renderer/utils/common';
import { useUserStore } from '@renderer/store/module/userStore';
import { useSettingStore } from '@renderer/store/module/settingStore';
import { getProfile, getOrginization } from '@renderer/api/modals/user';
import { getSystemSetting } from "@renderer/api/modals/setting";
import { createDiscreteApi, lightTheme, darkTheme } from 'naive-ui';

import type { ConfigProviderProps } from 'naive-ui';
import type { IUserInfo, IOrginization } from '@renderer/store/interface';


export const useUserAccount = () => {
  const { t } = useI18n();
  const router = useRouter();
  const userStore = useUserStore();
  const settingStore = useSettingStore();
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
      showLoginModal.value = true;
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
  const switchAccount = async (token: string) => {
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
      const setting = await getSystemSetting();

      if (setting) {
        settingStore.setRdpClientOption(setting.graphics.rdp_client_option);
        settingStore.setKeyboardLayout(setting.graphics.keyboard_layout);
        settingStore.setRdpSmartSize(setting.graphics.rdp_smart_size);
        settingStore.setRdpColorQuality(setting.graphics.rdp_color_quality);
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
    userStore.resetOrginization();

    try {
      const res = await getProfile();

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

        const setting = await getSystemSetting();

        if (setting) {
          settingStore.setRdpClientOption(setting.graphics.rdp_client_option);
          settingStore.setKeyboardLayout(setting.graphics.keyboard_layout);
          settingStore.setRdpSmartSize(setting.graphics.rdp_smart_size);
          settingStore.setRdpColorQuality(setting.graphics.rdp_color_quality);
        }

        showLoginModal.value = false;

        router.push({ name: 'Linux' });
      }
    } catch (e) {
      showLoginModal.value = false;
    }

    try {
      const orgRes = await getOrginization();

      if (orgRes) {
        userStore.setCurrentOrginization(orgRes?.id);
      }
    } catch(e) {
      useMessage.error(t('Message.GetOrginizationFailed'));
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

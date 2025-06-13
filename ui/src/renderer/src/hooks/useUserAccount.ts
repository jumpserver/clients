import { useI18n } from 'vue-i18n';
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useDebounceFn } from '@vueuse/core';
import { useElectronConfig } from './useElectronConfig';
import { getAvatarImage } from '@renderer/utils/common';
import { useUserStore } from '@renderer/store/module/userStore';
import { getSystemSetting } from '@renderer/api/modals/setting';
import { createDiscreteApi, lightTheme, darkTheme } from 'naive-ui';
import { useSettingStore } from '@renderer/store/module/settingStore';
import { getProfile, getOrganization } from '@renderer/api/modals/user';

import type { ConfigProviderProps } from 'naive-ui';
import type { IUserInfo, IOrganization } from '@renderer/store/interface';

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

      userStore.setSession(firstUser.session);
      userStore.setCurrentUser({ ...firstUser });
      userStore.setCurrentSit(firstUser.currentSite as string);
    }
  };

  /**
   * @description 切换账号
   */
  const switchAccount = async (session: string) => {
    if (session === userStore.session) {
      return;
    }

    if (userStore.userInfo) {
      const user = userStore.userInfo.find((item: IUserInfo) => item.session === session);

      if (user) {
        userStore.setSession(user.session);
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
   * @description 处理 session 接收
   */
  const _handleTokenReceived = async (session: string) => {
    if (!session) {
      useMessage.error('Token is required');
      return;
    }

    userStore.setSession(session);
    userStore.resetOrganization();

    try {
      const res = await getProfile();

      const orgRes = await getOrganization();

      if (res) {
        notification.create({
          type: 'success',
          content: t('Message.AuthenticatedSuccess'),
          duration: 2000
        });

        userStore.setUserInfo({
          session,
          username: res?.username,
          display_name: res?.system_roles.map((item: any) => item.display_name),
          avatar_url: await getAvatarImage(),
          currentSite: userStore.currentSite
        });

        userStore.setCurrentUser({
          session,
          username: res?.username,
          display_name: res?.system_roles.map((item: any) => item.display_name),
          avatar_url: await getAvatarImage(),
          currentSite: userStore.currentSite
        });

        const setting = await getSystemSetting();

        // 普通用户
        if (res.system_roles[0]?.id === '00000000-0000-0000-0000-000000000003') {
          userStore.setCurrentOrganization(orgRes.workbench_orgs[0]?.id);
          orgRes.workbench_orgs.forEach((org: IOrganization) => {
            userStore.setOrganization(org);
          });
        }

        if (res.system_roles[0]?.id === '00000000-0000-0000-0000-000000000002') {
          userStore.setCurrentOrganization(orgRes.audit_orgs[0]?.id);
          orgRes.audit_orgs.forEach((org: IOrganization) => {
            userStore.setOrganization(org);
          });
        }

        if (res.system_roles[0]?.id === '00000000-0000-0000-0000-000000000001') {
          userStore.setCurrentOrganization(orgRes.console_orgs[0]?.id);
          orgRes.console_orgs.forEach((org: IOrganization) => {
            userStore.setOrganization(org);
          });
        }

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

    // try {
    //   const currentRes = await getCurrent();

    //   if (currentRes) {
    //     userStore.setCurrentOrganization(currentRes?.id);
    //   }
    // } catch (e) {
    //   useMessage.error(t('Message.GetOrganizationFailed'));
    // }
  };

  const _handleCsrfTokenReceived = async (csrfToken: string) => {
    userStore.setCsrfToken(csrfToken);
  };

  const handleTokenReceived = useDebounceFn(_handleTokenReceived, 2000);
  const handleCsrfTokenReceived = useDebounceFn(_handleCsrfTokenReceived, 2000);

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

  // 监听主进程的 cookie 设置通知
  const setupCookiesForSite = () => {
    if (userStore.currentSite) {
      window.electron.ipcRenderer.send('get-current-site', userStore.currentSite);
    }
  };

  return {
    showLoginModal: showLoginModal,
    setNewAccount,
    switchAccount,
    removeAccount,
    getAccountInfo,
    handleModalOpacity,
    handleTokenReceived,
    handleCsrfTokenReceived,
    setupCookiesForSite
  };
};

import { defineStore } from 'pinia';
import { IUserInfo } from '@renderer/store/interface';
import { piniaPersistConfig } from '@renderer/store/helper';

import type { IUser, IOrganization } from '@renderer/store/interface';

export const useUserStore = defineStore('client-user', {
  state: (): Partial<IUser> => ({
    loading: false,

    sort: 'name',
    session: '',
    csrfToken: '',
    currentSite: '',
    currentOrganization: '',

    userInfo: [],
    organization: [],
    currentUser: {}
  }),
  actions: {
    setSession(session: string) {
      this.session = session;
    },
    setCsrfToken(csrfToken: string) {
      this.csrfToken = csrfToken;
    },
    setCurrentSit(site: string) {
      this.currentSite = site;
    },
    setUserInfo(userInfo: IUserInfo) {
      const existingUserIndex = this.userInfo!.findIndex(
        (item: IUserInfo) => item.session === userInfo.session
      );

      if (existingUserIndex !== -1) {
        this.userInfo![existingUserIndex] = { ...this.userInfo![existingUserIndex], ...userInfo };
      } else {
        this.userInfo!.push(userInfo);
      }
    },
    updateUserInfo(session: string, updates: Partial<IUserInfo>) {
      const userIndex = this.userInfo!.findIndex((item: IUserInfo) => item.session === session);

      if (userIndex !== -1) {
        this.userInfo![userIndex] = { ...this.userInfo![userIndex], ...updates };
      }
    },
    setLoading(status: boolean) {
      this.loading = status;
    },
    setCurrentUser(currentUser: IUserInfo) {
      this.currentUser = currentUser;
    },
    removeCurrentUser() {
      this.userInfo = this.userInfo!.filter(
        (item: IUserInfo) => item.session !== this.currentUser!.session
      );
    },
    setCurrentListSort(type) {
      this.sort = type;
    },
    setOrganization(orgInfo: IOrganization) {
      this.organization?.push({
        id: orgInfo.id,
        is_default: orgInfo.is_default,
        is_root: orgInfo.is_root,
        is_system: orgInfo.is_system,
        name: orgInfo.name
      });
    },
    setCurrentOrganization(orgId: string) {
      this.currentOrganization = orgId;
    },
    reset() {
      this.session = '';
      this.loading = false;
      this.userInfo = [];
      this.currentSite = '';
      this.currentUser = {};
      this.currentOrganization = '';
      this.organization = [];
    },
    resetOrganization() {
      this.currentOrganization = '';
      this.organization = [];
    }
  },
  persist: piniaPersistConfig('client-user')
});

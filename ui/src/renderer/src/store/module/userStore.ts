import { defineStore } from 'pinia';
import { IUserInfo } from '@renderer/store/interface';
import { piniaPersistConfig } from '@renderer/store/helper';

import type { IUser, IOrganization } from '@renderer/store/interface';

export const useUserStore = defineStore({
  id: 'client-user',
  state: (): Partial<IUser> => ({
    loading: false,

    sort: 'name',
    token: '',
    currentSite: '',
    currentOrganization: '',

    userInfo: [],
    organization: [],
    currentUser: {}
  }),
  actions: {
    setToken(token: string) {
      this.token = token;
    },
    setCurrentSit(site: string) {
      this.currentSite = site;
    },
    setUserInfo(userInfo: IUserInfo) {
      if (this.userInfo!.some((item: IUserInfo) => item.token === userInfo.token)) return;

      this.userInfo!.push(userInfo);
    },
    setLoading(status: boolean) {
      this.loading = status;
    },
    setCurrentUser(currentUser: IUserInfo) {
      this.currentUser = currentUser;
    },
    removeCurrentUser() {
      this.userInfo = this.userInfo!.filter(
        (item: IUserInfo) => item.token !== this.currentUser!.token
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
      this.token = '';
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

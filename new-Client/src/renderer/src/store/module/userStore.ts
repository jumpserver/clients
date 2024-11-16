import { defineStore } from 'pinia';
import { piniaPersistConfig } from '@renderer/store/helper';
import type { IUser } from '@renderer/store/interface';
import { IUserInfo } from '@renderer/store/interface';

export const useUserStore = defineStore({
  id: 'client-user',
  state: (): Partial<IUser> => ({
    token: '',
    loading: false,
    userInfo: [],
    currentSite: '',
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
      if (this.userInfo!.some((item: IUserInfo) => item.username === userInfo.username)) return;

      this.userInfo!.push(userInfo);
    },
    setLoading(status: boolean) {
      this.loading = status;
    },
    setCurrentUser(currentUser: IUserInfo) {
      this.currentUser = currentUser;
    }
  },
  persist: piniaPersistConfig('client-user')
});

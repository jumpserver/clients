import { defineStore } from 'pinia';
import { piniaPersistConfig } from '@renderer/store/helper';
import type { IUser } from '@renderer/store/interface';

export const useUserStore = defineStore({
  id: 'client-user',
  state: (): Partial<IUser> => ({
    token: '',
    loading: false
  }),
  actions: {
    setToken(token: string) {
      this.token = token;
    },
    setUserInfo(userInfo: IUser['userInfo']) {
      this.userInfo = userInfo;
    },
    setLoading(status: boolean) {
      this.loading = status;
    }
  },
  persist: piniaPersistConfig('client-user')
});

import { defineStore } from 'pinia';
import { piniaPersistConfig } from '@renderer/store/helper';
import type { IUser } from '@renderer/store/interface';
import { IUserInfo } from '@renderer/store/interface';

//todo)) 由于会有多个账号，所有需要在 userInfo 中去设置 token
export const useUserStore = defineStore({
  id: 'client-user',
  state: (): Partial<IUser> => ({
    token: '',
    loading: false,
    userInfo: [
      {
        username: 'ZhaoJiSen',
        value: 'ZhaoJiSen',
        display_name: ['Admin'],
        avatar_url: ''
      },
      {
        username: 'FengQiang',
        value: 'FengQiang',
        display_name: ['User'],
        avatar_url: ''
      }
    ],
    currentUser: {
      username: 'ZhaoJiSen',
      value: 'ZhaoJiSen',
      display_name: ['Admin'],
      avatar_url: ''
    }
  }),
  actions: {
    setToken(token: string) {
      this.token = token;
    },
    setUserInfo(userInfo: IUserInfo) {
      this.userInfo?.push(userInfo);
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

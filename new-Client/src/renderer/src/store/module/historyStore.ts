import { defineStore } from 'pinia';
import { piniaPersistConfig } from '@renderer/store/helper';
import { ISession } from '@renderer/store/interface';

export const useHistoryStore = defineStore('history', {
  state: () => ({
    history_session: []
  }),
  actions: {
    setHistorySession(s: ISession) {
      if (this.history_session!.some((item: ISession) => item.id === s.id)) return;
      this.history_session!.unshift(s);
      this.history_session!.slice(0, 50);
    },
    getHistorySession(name: string) {
      return this.history_session;
    }
  },
  persist: piniaPersistConfig('history')
});

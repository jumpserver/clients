import { defineStore } from 'pinia';
import type { ISetting } from '@/store/interface';
import { piniaPersistConfig } from '@renderer/store/helper';

export const useSettingStore = defineStore('setting', {
  state: (): ISetting => ({
    charset: 'default',
    rdp_resolution: 'auto',
    is_backspace_as_ctrl_h: 0
  }),
  actions: {
    setCharset(charset: string) {
      this.charset = charset;
    },
    setRdpResolution(rdp_resolution: string) {
      this.rdp_resolution = rdp_resolution;
    },
    setBackspaceAsCtrlH(is_backspace_as_ctrl_h: number) {
      this.is_backspace_as_ctrl_h = is_backspace_as_ctrl_h;
    }
  },
  persist: piniaPersistConfig('setting')
});

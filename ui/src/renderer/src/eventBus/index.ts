import mitt, { Emitter } from 'mitt';
import type { IItemDetail } from '@renderer/components/MainSection/interface';
import type { Ref } from 'vue';

type Event = {
  search: any;
  addAccount: any;
  changeTheme: any;
  changeLayout: any;
  createDrawer: void;
  connectAsset: void;
  removeAccount: void;
  showAssetDetail: { detailMessage: Ref<IItemDetail> };
};

const mittBus: Emitter<Event> = mitt();

export default mittBus;

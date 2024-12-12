import mitt, { Emitter } from 'mitt';

type Event = {
  search: any;
  addAccount: any;
  changeTheme: any;
  changeLayout: any;
  createDrawer: void;
  removeAccount: void;
  showAssetDetail: void;
  connectAsset: void;
};

const mittBus: Emitter<Event> = mitt();

export default mittBus;

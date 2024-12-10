import mitt, { Emitter } from 'mitt';

type Event = {
  search: any;
  addAccount: any;
  changeTheme: any;
  changeLayout: any;
  createDrawer: any;
  removeAccount: any;
};

const mittBus: Emitter<Event> = mitt();

export default mittBus;

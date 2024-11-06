import mitt, { Emitter } from 'mitt';

type Event = {
  changeLayout: any;
  createDrawer: any;
  changeTheme: any;
  search: any;
};

const mittBus: Emitter<Event> = mitt();

export default mittBus;

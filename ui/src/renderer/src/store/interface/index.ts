export interface IUserInfo {
  name?: string;

  username: string;

  display_name: Array<string>;

  avatar_url: string;

  value?: string;

  token: string;

  currentSite?: string;
}

export interface IUser {
  token: string;

  currentSite: string;

  userInfo: IUserInfo[];

  loading: boolean;

  currentUser: Partial<IUserInfo>;

  sort: string;
}

export interface ISetting {
  charset: string;

  rdp_resolution: string;

  is_backspace_as_ctrl_h: boolean;

  keyboard_layout: string;

  rdp_client_option: string[];

  rdp_color_quality: string;

  rdp_smart_size: string;
}

export interface ISession {
  id: string;
  name: string;
  address: string;
  comment: string;
  type: {
    value: string;
    label: string;
  };
  org_name: string;
}

export interface IConnectData {
  asset: string;
  account: string;
  protocol: string;
  input_username: string | undefined;
  input_secret: string | undefined;
  connect_method?: string;
  connect_options?: {
    charset: string;
    resolution: string;
    backspaceAsCtrlH: boolean;
  };
}

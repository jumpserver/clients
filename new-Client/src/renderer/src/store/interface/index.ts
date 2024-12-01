export interface IUserInfo {
  name?: string;

  username: string;

  display_name: Array<string>;

  avatar_url: string;

  value?: string;

  token: string;
}

export interface IUser {
  token: string;

  currentSite: string;

  userInfo: IUserInfo[];

  loading: boolean;

  currentUser: Partial<IUserInfo>;
}

export interface ISetting {
  charset: string;

  rdp_resolution: string;

  is_backspace_as_ctrl_h: number;
}

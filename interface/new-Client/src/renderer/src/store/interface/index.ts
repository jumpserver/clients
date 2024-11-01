export interface IUserInfo {
  username: string;

  display_name: Array<string>;

  avatar_url: string;
}

export interface IUser {
  token: string;

  userInfo: IUserInfo;

  loading: boolean;
}

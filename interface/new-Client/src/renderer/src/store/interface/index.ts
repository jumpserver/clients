export interface IUserInfo {
  name: string;

  username: string;

  display_name: Array<string>;

  avatar_url: string;

  value?: string;
}

export interface IUser {
  token: string;

  userInfo: IUserInfo[];

  loading: boolean;

  currentUser: Partial<IUserInfo>;
}

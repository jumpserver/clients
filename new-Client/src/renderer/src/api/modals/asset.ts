import request from '../index';
import { encryptPassword } from '@renderer/utils/crypto';
import { cleanRDPParams } from '@renderer/utils/common';
import { useSettingStore } from '@renderer/store/module/settingStore';
import { useUserStore } from '@renderer/store/module/userStore';

export const getFavoriteAssets = (params: object) => {
  return request.get('/api/v1/perms/users/self/nodes/favorite/assets/', params);
};

export const getAssets = (params: object) => {
  return request.get('/api/v1/perms/users/self/assets/', params);
};

export const getDatabases = (params: object) => {
  return request.get('/api/v1/perms/users/self/assets/', params);
};

export const getAssetDetail = (id: string) => {
  const url = `/api/v1/perms/users/self/assets/${id}/`;
  return request.get(url);
};

export const createConnectToken = (
  asset_id: string,
  account: { name?: string; username?: string },
  secret: string,
  method: string,
  protocol: string,
  createTicket = false,
  connectOption = {}
) => {
  const params = createTicket ? '?create_ticket=1' : '';
  const url = '/api/v1/authentication/connection-token/' + params;
  // account.username.startsWith('@') ? manualAuthInfo.username :
  const username = account.username;
  const _secret = encryptPassword(secret);
  const data = {
    asset: asset_id,
    account: account.username,
    protocol: protocol,
    input_username: username,
    input_secret: _secret,
    connect_method: method,
    connect_options: connectOption
  };
  return request.post(url, data);
};

export const getLocalClientUrl = token => {
  const userStore = useUserStore();
  const settingStore = useSettingStore();
  const url = new URL(
    `/api/v1/authentication/connection-token/${token.id}/client-url/`,
    userStore.currentSite ?? 'https://jumpserver.local'
  );
  let params = cleanRDPParams(settingStore);
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      url.searchParams.append(k, v);
    }
  }
  return request.get(url.href);
};

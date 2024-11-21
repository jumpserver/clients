import request from '../index';
import { ConnectionToken } from '../model';
import { encryptPassword } from '@renderer/utils/crypto';

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
  acccount: object,
  method: string,
  method: string,
  protocol: string,
  createTicket = false,
  connectOption = {}
) => {
  const params = createTicket ? '?create_ticket=1' : '';
  const url = '/api/v1/authentication/connection-token/' + params;
  // const username = account.username.startsWith('@') ? manualAuthInfo.username : account.username;
  const secret = encryptPassword('');
  const data = {
    asset: asset_id,
    account: account.name,
    protocol: protocol,
    input_username: username,
    input_secret: secret,
    connect_method: method,
    connect_options: connectOption
  };
  return request.post<ConnectionToken>(url, data);
};

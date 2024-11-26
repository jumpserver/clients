import request from '../index';
import { ConnectionToken } from '../model';
import { encryptPassword } from '@renderer/utils/crypto'; // import { cleanRDPParams } from '@renderer/utils/common';
// import { cleanRDPParams } from '@renderer/utils/common';

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
    account: account.name,
    protocol: protocol,
    input_username: username,
    input_secret: _secret,
    connect_method: method,
    connect_options: connectOption
  };
  return request.post<ConnectionToken>(url, data);
};

export const getLocalClientUrl = (token, params: Object = {}) => {
  const url = new URL(
    `/api/v1/authentication/connection-token/${token.id}/client-url/`,
    window.location.origin
  );
  // params = cleanRDPParams(params);
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      url.searchParams.append(k, v);
    }
  }
  return request.get(url.pathname);
};

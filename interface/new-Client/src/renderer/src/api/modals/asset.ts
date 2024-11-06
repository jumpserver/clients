import request from '../index';

export const getFavorites = (params: object) => {
  return request.get('/api/v1/perms/users/self/nodes/favorite/assets/', params);
};

export const getAssets = (params: object) => {
  return request.get('/api/v1/perms/users/self/assets/', params);
};

export const getDatabases = (params: object) => {
  return request.get('/api/v1/perms/users/self/assets/', params);
};

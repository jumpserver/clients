import request from '../index';

export const getFavorite = () => {
  return request.get('/api/v1/assets/favorite-assets/');
};

export const getAsset = (type: string, offset: number, limit: number) => {
  return request.get(
    `/api/v1/perms/users/self/assets/?type=${type}&offset=${offset}&limit=${limit}`
  );
};

import request from '../index';

export const getProfile = () => {
  return request.get('/api/v1/users/profile/');
};

import request from '../index';

export const getSystemSetting = () => {
  return request.get('/api/v1/users/preference/?category=luna');
};

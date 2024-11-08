import type { ResultData, CustomAxiosRequestConfig } from './interface/index';
import type { AxiosResponse, AxiosInstance, AxiosRequestConfig, AxiosError } from 'axios';
import { useUserStore } from '@renderer/store/module/userStore';
import { createDiscreteApi } from 'naive-ui';
import { router } from '@renderer/router';
import axios from 'axios';

const { message } = createDiscreteApi(['message']);

const config = {
  timeout: 5000,
  withCredentials: true
};

class RequestHttp {
  public service: AxiosInstance;

  constructor(config: AxiosRequestConfig) {
    this.service = axios.create(config);

    /**
     * @description 请求拦截器
     */
    this.service.interceptors.request.use(
      (config: CustomAxiosRequestConfig) => {
        const userStore = useUserStore();

        config.loading ??= true;
        config.baseURL = userStore.currentSite;

        userStore.setLoading(config.loading);

        if (config.headers && typeof config.headers.set === 'function') {
          config.headers['Authorization'] = `Bearer ${userStore.token}`;
          config.headers['X-JMS-ORG'] = '00000000-0000-0000-0000-000000000000';
          config.headers['X-TZ'] = Intl.DateTimeFormat().resolvedOptions().timeZone;
        }

        return config;
      },
      (error: any) => {
        message.error(error);

        return Promise.reject(error);
      }
    );

    /**
     * @description 响应拦截器
     */
    this.service.interceptors.response.use(
      (response: AxiosResponse & { config: CustomAxiosRequestConfig }) => {
        const { data, status, config } = response;

        const userStore = useUserStore();

        config.loading && userStore.setLoading(false);

        // 登录失效
        if (status == 401) {
          userStore.setToken('');
          message.error('登录认证已失效');
          return Promise.reject(data);
        }

        return data;
      },
      (error: AxiosError) => {
        if (error.message.indexOf('timeout') !== -1) message.error('请求超时！请您稍后重试');
        if (error.message.indexOf('Network Error') !== -1) message.error('网络错误！请您稍后重试');

        if (!window.navigator.onLine) router.replace({ name: '404' });

        return Promise.reject(error);
      }
    );
  }

  get(url: string, params?: object, _object = {}): Promise<any> {
    return this.service.get(url, { params, ..._object });
  }
  post<T>(url: string, params?: object | string, _object = {}): Promise<ResultData<T>> {
    return this.service.post(url, params, _object);
  }
  put<T>(url: string, params?: object, _object = {}): Promise<ResultData<T>> {
    return this.service.put(url, params, _object);
  }
  delete<T>(url: string, params?: any, _object = {}): Promise<ResultData<T>> {
    return this.service.delete(url, { params, ..._object });
  }
}

export default new RequestHttp(config);

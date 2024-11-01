import type { InternalAxiosRequestConfig } from 'axios';

export interface ResultData<T = any> extends Result {
  data: T;
}

export interface CustomAxiosRequestConfig extends InternalAxiosRequestConfig {
  loading?: boolean;
}

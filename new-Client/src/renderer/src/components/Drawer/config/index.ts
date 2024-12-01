import type { Ref } from 'vue';
import { ref } from 'vue';

export interface ICustomFrom {
  path: string;
  comment: object;
  download_url: string;
  protocol: any;
  arg_format: string;
  is_internal: boolean;
  is_set: boolean;
  is_default: boolean;
}

export const linuxOptions: Ref<Array<ICustomFrom>> = ref([]);

export const windowsOptions: Ref<Array<ICustomFrom>> = ref([]);

export const databaseOptions: Ref<Array<ICustomFrom>> = ref([]);

export const charsetOptions = [
  { label: 'Default', value: 'default' },
  { label: 'UTF-8', value: 'utf8' },
  { label: 'GBK', value: 'gbk' },
  { label: 'GB2312', value: 'gb2312' },
  { label: 'IOS-8859-1', value: 'ios-8859-1' }
];

export const resolutionsOptions = [
  { label: 'Auto', value: 'auto' },
  { label: '1024x768', value: '1024x768' },
  { label: '1366x768', value: '1366x768' },
  { label: '1600x900', value: '1600x900' },
  { label: '1920x1080', value: '1920x1080' }
];

export const boolOptions = [
  { label: 'Yes', value: 1 },
  { label: 'No', value: 0 }
];

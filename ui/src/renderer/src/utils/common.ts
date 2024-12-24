import { LocalStorageService } from '@renderer/utils/localstorage';

export function getCookie(name: string): string {
  let cookieValue = '';
  if (document.cookie && document.cookie !== '') {
    const cookies = document.cookie.split(';');
    for (let i = 0; i < cookies.length; i++) {
      const cookie = cookies[i].trim();
      // Does this cookie string begin with the name we want?
      if (cookie.substring(0, name.length + 1) === name + '=') {
        cookieValue = decodeURIComponent(cookie.substring(name.length + 1));
        break;
      }
    }
  }
  return cookieValue;
}

export function getCsrfTokenFromCookie(): string {
  let prefix = getCookie('SESSION_COOKIE_NAME_PREFIX');
  if (!prefix || [`""`, `''`].indexOf(prefix) > -1) {
    prefix = '';
  }
  const name = `${prefix}csrftoken`;
  return getCookie(name);
}

export function cleanRDPParams(params): Object {
  const cleanedParams = {};

  const { rdp_resolution, rdp_client_option, rdp_smart_size, rdp_color_quality } = params;

  if (rdp_resolution && rdp_resolution.indexOf('x') > -1) {
    const [width, height] = rdp_resolution.split('x');
    cleanedParams['width'] = width;
    cleanedParams['height'] = height;
  }
  if (rdp_client_option.includes('full_screen')) {
    cleanedParams['full_screen'] = '1';
  }
  if (rdp_client_option.includes('multi_screen')) {
    cleanedParams['multi_mon'] = '1';
  }
  if (rdp_client_option.includes('drives_redirect')) {
    cleanedParams['drives_redirect'] = '1';
  }

  cleanedParams['rdp_smart_size'] = rdp_smart_size;
  cleanedParams['rdp_color_quality'] = rdp_color_quality;
  return cleanedParams;
}

export function getConnectOption(params): Object {
  const connectOption = {};
  const { charset, is_backspace_as_ctrl_h, rdp_resolution, keyboard_layout } = params;

  if (rdp_resolution && rdp_resolution.indexOf('x') > -1) {
    connectOption['resolution'] = rdp_resolution;
  }

  connectOption['charset'] = charset;
  connectOption['is_backspace_as_ctrl_h'] = is_backspace_as_ctrl_h;
  connectOption['keyboard_layout'] = keyboard_layout;

  return connectOption;
}

export function loadOriManualAuthInfo() {
  const manualAuthInfoKey = 'ManualAuthInfo';
  const authInfos = LocalStorageService.get(manualAuthInfoKey);
  if (!authInfos) {
    return;
  }
  if (authInfos && typeof authInfos === 'object') {
    for (const [key, auths] of Object.entries(authInfos)) {
      const newKey = `JMS_MA_${key}`;
      LocalStorageService.set(newKey, auths);
    }
  }
  LocalStorageService.delete(manualAuthInfoKey);
}

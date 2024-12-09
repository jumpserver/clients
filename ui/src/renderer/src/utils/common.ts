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

export function setPreConnectData(asset, connectData) {
  const { account, protocol, connectMethod, manualAuthInfo, connectOption } = connectData;
  const key = `JMS_PRE_${asset.id}`;

  const saveData = {
    account: { alias: account.alias, username: account.username, has_secret: account.has_secret },
    connectMethod: { value: connectMethod.value },
    protocol: { name: protocol.name },
    downloadRDP: connectData.downloadRDP,
    autoLogin: connectData.autoLogin,
    connectOption
  };
  setAccountLocalAuth(asset, account, manualAuthInfo);
  LocalStorageService.set(key, saveData);
}

export function getPreConnectData(asset: Asset) {
  const key = `JMS_PRE_${asset.id}`;
  const connectData = LocalStorageService.get(key) as ConnectData;
  if (!connectData) {
    return null;
  }
  connectData.manualAuthInfo = new AuthInfo();
  if (connectData.account.has_secret) {
    return connectData;
  }
  if (connectData.account) {
    const auths = getAccountLocalAuth(asset.id);
    const matched = auths.find(item => item.alias === connectData.account.alias);
    if (matched) {
      connectData.manualAuthInfo = matched;
    }
  }
  return connectData;
}

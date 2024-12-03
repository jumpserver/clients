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

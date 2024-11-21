export class Action {
  label: string = '';
  value: string = '';
}

export class Account {
  alias: string = '';
  name: string = '';
  username: string = '';
  has_secret: boolean = false;
  secret: string = '';
  actions: Array<Action> = [];
}

class Choice {
  label: string = '';
  value: string = '';
}

class SpecInfo {
  db_name?: string = '';
}

export class Asset {
  id: string = '';
  name: string = '';
  address: string = '';
  comment: string = '';
  type: Choice = new Choice();
  category: Choice = new Choice();
  permed_protocols: Array<Protocol> = [];
  permed_accounts: Array<Account> = [];
  spec_info: SpecInfo = new SpecInfo();
}

export class GlobalSetting {
  WINDOWS_SKIP_ALL_MANUAL_PASSWORD: boolean = false;
  SECURITY_MAX_IDLE_TIME: number = 0;
  XPACK_LICENSE_IS_VALID: boolean = false;
  SECURITY_COMMAND_EXECUTION: boolean = false;
  SECURITY_LUNA_REMEMBER_AUTH: boolean = false;
  SECURITY_WATERMARK_ENABLED: boolean = false;
  HELP_DOCUMENT_URL: string = '';
  HELP_SUPPORT_URL: string = '';
  TERMINAL_RAZOR_ENABLED: boolean = false;
  TERMINAL_MAGNUS_ENABLED: boolean = false;
  TERMINAL_KOKO_SSH_ENABLED: boolean = false;
  INTERFACE: any = null;
  TERMINAL_GRAPHICAL_RESOLUTION: string = '';
  CONNECTION_TOKEN_REUSABLE: boolean = false;
  CHAT_AI_ENABLED: boolean = false;
}

export class Setting {
  commandExecution: boolean = true;
  isSkipAllManualPassword: string = '0';
  sqlClient = '1';

  basic = {
    is_async_asset_tree: false,
    connect_default_open_method: 'new'
  };
  graphics = {
    rdp_resolution: 'Auto',
    keyboard_layout: 'en-us-qwerty',
    rdp_client_option: [],
    applet_connection_method: 'web',
    rdp_smart_size: '0',
    rdp_color_quality: '32'
  };
  command_line = {
    character_terminal_font_size: 14,
    is_backspace_as_ctrl_h: false,
    is_right_click_quickly_paste: false
  };
}

export class AuthInfo {
  alias: string = '';
  username: string = '';
  secret: string = '';
  rememberAuth: boolean = false;
}

export class ConnectData {
  asset: Asset = new Asset();
  account: Account = new Account();
  protocol: Protocol = new Protocol();
  manualAuthInfo: AuthInfo = new AuthInfo();
  connectOption: Object = {};
  autoLogin: boolean = false;
}

class FromTicketInfo {
  check_ticket_api: {
    method: string;
    url: string;
  } = {
    method: '',
    url: ''
  };
  close_ticket_api: {
    method: string;
    url: string;
  } = {
    method: '',
    url: ''
  };
  ticket_detail_page_url: string = '';
  assignees: Array<string> = [];
}

export class ConnectionToken {
  id: string = '';
  value: string = '';
  protocol: string = '';
  asset: string = '';
  user?: string = '';
  account: string = '';
  expire_time: number = 0;
  is_active: boolean = false;
  date_expired: Date = new Date();
  is_reusable: boolean = false;
  from_ticket: {
    id: string;
  } = {
    id: ''
  };
  from_ticket_info: FromTicketInfo = new FromTicketInfo();
}

export class Protocol {
  name: string = '';
  port: number = 0;
  public: boolean = false;
  setting: any = null;
}

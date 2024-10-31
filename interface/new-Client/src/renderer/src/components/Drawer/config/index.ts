import type { SelectOption } from 'naive-ui';
import { ref } from 'vue';
import type { Ref } from 'vue';

export interface ICustomFrom {
  enable: boolean;
  path: string;
  textarea: string;
  downLoadUrl: string;
}

export interface ICustomSelectOption extends SelectOption {
  form: ICustomFrom;
}

export const linuxOptions: Ref<Array<ICustomSelectOption>> = ref([
  {
    label: 'Terminal',
    value: 'terminal',
    form: {
      enable: true,
      path: 'Terminal',
      textarea: 'Terminal是MacOS操作系统上的虚拟终端应用软件，位于“实用工具”文件夹内。',
      downLoadUrl: '系统自带'
    }
  },
  {
    label: 'iTerm2',
    value: 'iTerm2',
    form: {
      enable: false,
      path: 'iTerm2',
      textarea: 'iTerm2是MacOS操作系统上的虚拟终端应用软件。',
      downLoadUrl: 'https://iterm2.com/downloads.html'
    }
  },
  {
    label: 'SecureCRT',
    value: 'secureCRT',
    form: {
      enable: false,
      path: '/Applications/SecureCRT.app/Contents/MacOS/SecureCRT',
      textarea: 'SecureCRT是VanDyke Software所开发销售的一个SSH、Telnet客户端和虚拟终端软件。',
      downLoadUrl: 'https://www.vandyke.com/cgi-bin/releases.php?product=securecrt'
    }
  }
]);

export const windowsOptions: Ref<Array<ICustomSelectOption>> = ref([
  {
    label: 'Terminal',
    value: 'terminal',
    form: {
      enable: true,
      path: 'Terminal',
      textarea: 'Terminal是MacOS操作系统上的虚拟终端应用软件，位于“实用工具”文件夹内。',
      downLoadUrl: '系统自带'
    }
  },
  {
    label: 'Microsoft Remote Desktop',
    value: 'microsoftRemoteDesktop',
    form: {
      enable: false,
      path: '/Applications/Microsoft Remote Desktop.app',
      textarea:
        'Microsoft Remote Desktop是一款强大的微软远程连接工具，可以从几乎任何地方连接到远程PC和您的工作资源。',
      downLoadUrl: '系统自带'
    }
  }
]);

export const databaseOptions: Ref<Array<ICustomSelectOption>> = ref([
  {
    label: 'Terminal',
    value: 'terminal',
    form: {
      enable: true,
      path: 'Terminal',
      textarea: 'Terminal是MacOS操作系统上的虚拟终端应用软件，位于“实用工具”文件夹内。',
      downLoadUrl: '系统自带'
    }
  },
  {
    label: 'DBeaver Community',
    value: 'dBeaver',
    form: {
      enable: false,
      path: '/Applications/DBeaver.app/Contents/MacOS/dbeaver',
      textarea:
        'DBeaver Community是一个通用的数据库管理工具和SQL客户端，支持MySQL、PostgreSQL、Oracle以及其他兼容JDBC的数据库。',
      downLoadUrl: 'https://dbeaver.io/download/'
    }
  },
  {
    label: 'Another Redis Desktop Manager',
    value: 'anotherRedisDesktopManager',
    form: {
      enable: false,
      path: '/Applications/Another Redis Desktop Manager.app/Contents/MacOS/Another Redis Desktop Manager',
      textarea: '更快、更好、更稳定的Redis桌面(GUI)管理客户端。',
      downLoadUrl: 'https://github.com/qishibo/AnotherRedisDesktopManager'
    }
  }
]);

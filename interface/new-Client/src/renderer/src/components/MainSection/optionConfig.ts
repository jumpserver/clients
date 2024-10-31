export const useAccount = () => {
  const setAccount = () => {};
  const getAccount = () => {};

  return {
    getAccount,
    setAccount
  };
};

export const defaultSetting = [
  {
    type: 'linux',
    children: [
      {
        label: 'Terminal',
        key: 'terminal',
        tooltip: '',
        icon: ''
      },
      {
        label: 'iTerm2',
        key: 'iTerm2',
        tooltip: '',
        icon: ''
      },
      {
        label: 'SecureCRT',
        key: 'secureCRT',
        tooltip: '',
        icon: ''
      }
    ]
  },
  {
    type: 'windows'
  },
  {
    type: 'database'
  }
];

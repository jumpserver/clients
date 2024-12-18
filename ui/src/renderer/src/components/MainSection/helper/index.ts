import { h, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  createDiscreteApi,
  NText,
  NIcon,
  NButton,
  NDescriptions,
  NDescriptionsItem,
  NForm,
  NFormItem,
  NInput
} from 'naive-ui';
import type { Component } from 'vue';
import type { DropdownOption, ConfigProviderProps } from 'naive-ui';
import type { IItemDetail } from '@renderer/components/MainSection/interface';
import type { Ref } from 'vue';
import mittBus from '@renderer/eventBus';
import { computed } from 'vue';
import { lightTheme, darkTheme } from 'naive-ui';
import { Conf } from 'electron-conf/renderer';

export const renderCustomHeader = (
  component: Component,
  text: string,
  type: string = '',
  detailMessage?: Ref<IItemDetail>,
  callback?: () => void
) => {
  return () => {
    const { t } = useI18n();
    const conf = new Conf();
    const defaultTheme = ref('');

    conf.get('defaultSetting').then(res => {
      if (res) {
        // @ts-ignore
        defaultTheme.value = res?.theme;
      }
    });

    const configProviderPropsRef = computed<ConfigProviderProps>(() => ({
      theme: defaultTheme.value === 'light' ? lightTheme : darkTheme
    }));

    const { modal } = createDiscreteApi(['modal'], {
      configProviderProps: configProviderPropsRef
    });

    const handleDetailClick = () => {
      if (!detailMessage?.value) return;

      console.log(detailMessage.value);

      modal.create({
        title: t('Common.AssetDetails'),
        preset: 'card',
        bordered: false,
        segmented: true,
        style: {
          width: '40rem',
          borderRadius: '10px'
        },
        content: () =>
          h(
            NDescriptions,
            {
              column: 1,
              labelPlacement: 'left',
              bordered: true
            },
            {
              default: () => [
                // Platform Info
                h(
                  NDescriptionsItem,
                  {
                    label: t('Common.PlatformInfo')
                  },
                  {
                    default: () => [
                      `${t('Common.PlatformID')}: ${detailMessage.value.platform.id}`,
                      h('br'),
                      `${t('Common.PlatformName')}: ${detailMessage.value.platform.name}`
                    ]
                  }
                ),

                // Connectivity
                h(
                  NDescriptionsItem,
                  {
                    label: t('Common.Connectivity')
                  },
                  {
                    default: () => [` ${detailMessage.value.connectivity.label}`]
                  }
                ),

                // Category
                h(
                  NDescriptionsItem,
                  {
                    label: t('Common.Category')
                  },
                  {
                    default: () => detailMessage.value.category.label
                  }
                ),

                // Nodes
                h(
                  NDescriptionsItem,
                  {
                    label: t('Common.Nodes')
                  },
                  {
                    default: () =>
                      detailMessage.value.nodes.map(node =>
                        h('div', { key: node.id }, `${node.name} (ID: ${node.id})`)
                      )
                  }
                ),

                // Protocols
                h(
                  NDescriptionsItem,
                  {
                    label: t('Common.PermedProtocols')
                  },
                  {
                    default: () =>
                      detailMessage.value.permed_protocols.map(protocol =>
                        h('div', { key: protocol.name }, [
                          `${protocol.name} (Port: ${protocol.port}) `
                        ])
                      )
                  }
                ),

                // Accounts
                h(
                  NDescriptionsItem,
                  {
                    label: t('Common.PermedAccounts')
                  },
                  {
                    default: () =>
                      detailMessage.value.permed_accounts.map(account =>
                        h(
                          'div',
                          { key: account.id },
                          `${account.alias} (${account.username || t('Common.NoUsername')})`
                        )
                      )
                  }
                )
              ]
            }
          )
      });

      callback?.();
    };

    return h(
      'div',
      {
        style: {
          display: 'flex',
          alignItems: 'center',
          padding: '8px 12px'
        }
      },
      [
        h(
          NButton,
          {
            text: true,
            size: 'small',
            type: 'primary',
            style: {
              width: '100%',
              justifyContent: 'flex-start'
            },
            onClick: () => {
              switch (type) {
                case 'detail-message':
                  handleDetailClick();
                  break;
                case 'fast-connection':
                  mittBus.emit('connectAsset');
                  break;
              }
            }
          },
          {
            default: () => [
              h(NIcon, {
                size: '20',
                component: component,
                style: { marginRight: '0.5rem' }
              }),
              h(NText, { depth: 1 }, { default: () => text })
            ]
          }
        )
      ]
    );
  };
};

export const moveElementToEnd = (arr: DropdownOption[], searchKey: string, changeText: string) => {
  const index = arr.findIndex(item => item.label === searchKey);
  if (index !== -1) {
    const elementToMove = arr[index];
    arr.splice(index, 1);
    arr.push({
      key: elementToMove.key,
      label: changeText
    });
  }
  return arr;
};

export const useAccountModal = (type: string, t: any) => {
  const conf = new Conf();
  const defaultTheme = ref('');
  const inputPassword = ref('');
  const inputUsername = ref('');

  conf.get('defaultSetting').then(res => {
    if (res) {
      // @ts-ignore
      defaultTheme.value = res?.theme;
    }
  });

  const configProviderPropsRef = computed<ConfigProviderProps>(() => ({
    theme: defaultTheme.value === 'light' ? lightTheme : darkTheme
  }));

  const { modal } = createDiscreteApi(['modal'], {
    configProviderProps: configProviderPropsRef
  });

  const modalTitle =
    type !== '@INPUT' ? t('Common.InputPassword') : t('Common.InputAccountPassword');

  const m = modal.create({
    title: modalTitle,
    preset: 'card',
    bordered: false,
    segmented: true,
    style: {
      width: '30rem',
      borderRadius: '10px'
    },
    content: () =>
      h(
        NForm,
        {
          labelPlacement: 'top',
          labelWidth: 80,
          size: 'small'
        },
        {
          default: () => [
            // 用户名输入框（仅在 @INPUT 类型时显示）
            h(
              NFormItem,
              {
                label: t('Common.Username'),
                path: 'username',
                style: { display: type !== '@INPUT' ? 'none' : '' }
              },
              {
                default: () =>
                  h(NInput, {
                    value: inputUsername.value,
                    clearable: true,
                    placeholder: t('Common.UsernamePlaceholder'),
                    onUpdateValue: value => {
                      inputUsername.value = value;
                    }
                  })
              }
            ),
            // 密码输入框
            h(
              NFormItem,
              {
                label: t('Common.Password'),
                path: 'password'
              },
              {
                default: () =>
                  h(NInput, {
                    value: inputPassword.value,
                    type: 'password',
                    clearable: true,
                    showPasswordOn: 'click',
                    placeholder: t('Common.InputPassword'),
                    onUpdateValue: value => {
                      inputPassword.value = value;
                    }
                  })
              }
            ),
            // 确认按钮
            h(
              NFormItem,
              {
                style: {
                  display: 'flex',
                  justifyContent: 'flex-end'
                }
              },
              {
                default: () =>
                  h(
                    NButton,
                    {
                      type: 'primary',
                      size: 'medium',
                      onClick: () => m.destroy()
                    },
                    {
                      default: () => t('Common.Confirm')
                    }
                  )
              }
            )
          ]
        }
      )
  });

  return {
    inputPassword,
    inputUsername
  };
};

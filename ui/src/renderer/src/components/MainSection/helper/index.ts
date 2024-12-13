import { computed, h, ref } from 'vue';
import {
  NText,
  NIcon,
  NButton,
  NForm,
  NFormItem,
  NInput,
  ConfigProviderProps,
  lightTheme,
  darkTheme,
  createDiscreteApi
} from 'naive-ui';
import mittBus from '@renderer/eventBus';

import type { Ref } from 'vue';
import type { Component } from 'vue';
import type { DropdownOption } from 'naive-ui';
import type { IItemDetail } from '@renderer/components/MainSection/interface';

import { Conf } from 'electron-conf/renderer';

export const renderCustomHeader = (
  component: Component,
  text: string,
  type: string = '',
  detailMessage?: Ref<IItemDetail>,
  callback?: () => void
) => {
  return () => {
    return h(
      'div',
      {
        style: 'display: flex; align-items: center; padding: 8px 12px;'
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
                case 'detail-message': {
                  if (detailMessage) {
                    mittBus.emit('showAssetDetail', { detailMessage });

                    callback ? callback() : '';
                  }

                  break;
                }
                case 'fast-connection': {
                  mittBus.emit('connectAsset');

                  break;
                }
              }
            }
          },
          {
            default: () => [
              h(NIcon, {
                size: '20',
                component: component,
                style: {
                  marginRight: '0.5rem'
                }
              }),
              h(
                NText,
                {
                  depth: 1
                },
                { default: () => text }
              )
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
    // 保存要移动的元素
    const elementToMove = arr[index];

    // 从数组中移除该元素
    arr.splice(index, 1);

    // 将元素添加到数组末尾
    arr.push({
      key: elementToMove.key,
      label: changeText
    });
  }

  return arr;
};

export const useAccountModal = (type: string) => {
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

  const modalTitle = type === '手动输入' ? '手动输入账号密码' : '输入账号密码';

  const handleConfirm = () => {
    console.log(1);
  };

  modal.create({
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
            h(
              NFormItem,
              {
                label: '用户名',
                path: 'username'
              },
              {
                default: () =>
                  h(NInput, {
                    value: inputUsername.value,
                    clearable: true,
                    placeholder: '用户名',
                    onUpdateValue: value => {
                      inputUsername.value = value;
                    }
                  })
              }
            ),
            h(
              NFormItem,
              {
                label: '密码',
                path: 'password'
              },
              {
                default: () =>
                  h(NInput, {
                    value: inputPassword.value,
                    type: 'password',
                    clearable: true,
                    showPasswordOn: 'click',
                    placeholder: '请输入密码',
                    onUpdateValue: value => {
                      inputPassword.value = value;
                    }
                  })
              }
            ),
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
                      onClick: () => handleConfirm()
                    },
                    {
                      default: () => '确认'
                    }
                  )
              }
            )
          ]
        }
      )
  });

  return {
    inputPassword: inputPassword.value,
    inputUsername: inputUsername.value
  };
};

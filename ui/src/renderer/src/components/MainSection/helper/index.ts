import { h } from 'vue';
import { NText, NIcon, NButton } from 'naive-ui';
import mittBus from '@renderer/eventBus';

import type { Ref } from 'vue';
import type { Component } from 'vue';
import type { DropdownOption } from 'naive-ui';
import type { IItemDetail } from '@renderer/components/MainSection/interface';

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
                color: '#fff',
                component: component,
                style: {
                  marginRight: '0.5rem'
                }
              }),
              h(
                NText,
                {
                  depth: 1,
                  strong: true
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

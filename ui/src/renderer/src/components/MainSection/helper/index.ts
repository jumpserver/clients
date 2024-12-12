import { NText, NIcon, NButton, type DropdownOption } from 'naive-ui';
import { h } from 'vue';
import type { Component } from 'vue';
import mittBus from '@renderer/eventBus';

const handleActionButton = (type: string) => {
  switch (type) {
    case 'detail-message': {
      mittBus.emit('showAssetDetail');

      break;
    }
    case 'fast-connection': {
      mittBus.emit('connectAsset');
      break;
    }
  }
};

export const renderCustomHeader = (component: Component, text: string, type: string = '') => {
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
              handleActionButton(type);
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

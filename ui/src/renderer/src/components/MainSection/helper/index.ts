import { NText, NIcon, type DropdownOption } from 'naive-ui';
import { h } from 'vue';
import type { Component } from 'vue';

export const renderCustomHeader = (component: Component, text: string) => {
  return () => {
    return h(
      'div',
      {
        style: 'display: flex; align-items: center; padding: 8px 12px;'
      },
      [
        h(NIcon, {
          size: '20',
          color: '#fff',
          component: component
        }),
        h(
          'div',
          {
            style: {
              marginLeft: '0.5rem'
            }
          },
          [h('div', null, [h(NText, { depth: 1, strong: true }, { default: () => text })])]
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

    // todo)) 后续优化样式
    // const postprocessingArray = [
    //   {
    //     key: 'footer-divider',
    //     type: 'footer'
    //   },
    //   {
    //     key: elementToMove.key,
    //     type: 'render',
    //     render: () => {
    //       h(
    //         NButton,
    //         {},
    //         {
    //           default: () => changeText
    //         }
    //       );
    //     }
    //   }
    // ];

    // 将元素添加到数组末尾
    arr.push({
      key: elementToMove.key,
      label: changeText
    });
  }

  return arr;
};

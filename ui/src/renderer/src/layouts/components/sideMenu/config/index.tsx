import { renderIcon } from '@renderer/layouts/components/HeaderSection/helper';
import { useI18n } from 'vue-i18n';
import { h } from 'vue';

import { BrandWindows, Database, Terminal2, Star, History } from '@vicons/tabler';
import { Trash2, CircleCheck } from 'lucide-vue-next';
import { NFlex, NAvatar, NText } from 'naive-ui';
import { Devices } from '@vicons/carbon';
import { RouterLink } from 'vue-router';

import type { MenuOption, SelectOption } from 'naive-ui';

export const menuOptions = () => {
  const { t } = useI18n();

  return [
    {
      label: () =>
        h(
          RouterLink,
          {
            to: {
              name: 'Linux'
            }
          },
          { default: () => 'Linux' }
        ),
      key: 'linux-page',
      icon: renderIcon(Terminal2)
    },
    {
      label: () =>
        h(
          RouterLink,
          {
            to: {
              name: 'Windows'
            }
          },
          { default: () => 'Windows' }
        ),
      key: 'windows-page',
      icon: renderIcon(BrandWindows)
    },
    {
      label: () =>
        h(
          RouterLink,
          {
            to: {
              name: 'Database'
            }
          },
          { default: () => t('Router.Database') }
        ),
      key: 'database-page',
      icon: renderIcon(Database)
    },
    {
      label: () =>
        h(
          RouterLink,
          {
            to: {
              name: 'Device'
            }
          },
          { default: () => t('Router.Device') }
        ),
      key: 'device-page',
      icon: renderIcon(Devices)
    },
    {
      label: () =>
        h(
          RouterLink,
          {
            to: {
              name: 'Favorite'
            }
          },
          { default: () => t('Router.Favorite') }
        ),
      key: 'favorite-page',
      icon: renderIcon(Star)
    },
    {
      label: () =>
        h(
          RouterLink,
          {
            to: {
              name: 'History'
            }
          },
          { default: () => t('Router.History') }
        ),
      key: 'history-page',
      icon: renderIcon(History)
    }
  ] as MenuOption[];
};

/**
 * @description 获取账号选项渲染 此处必须由于 render-label 必须返回一个 VNode 因此不能写 TSX
 * @param option
 * @returns
 */
export const getAccountOptionsRender = (option: SelectOption) => {
  return h(
    NFlex,
    {
      class: 'w-full',
      align: 'center',
      justify: 'start'
    },
    {
      default: () => [
        h(NAvatar, {
          size: 20,
          src: option.avatar_url as string,
          class: 'mr-2'
        }),
        h(
          NText,
          {
            depth: 1,
            class: 'font-medium text-sm'
          },
          { default: () => option.display_name }
        ),
        h(
          NFlex,
          {
            class: 'ml-auto'
          },
          {
            default: () => [
              h(Trash2, {
                size: 16,
                class: 'mr-2'
              }),
              h(CircleCheck, {
                size: 16
              })
            ]
          }
        )
      ]
    }
  );
};

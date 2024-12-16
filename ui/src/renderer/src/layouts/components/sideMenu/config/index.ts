import { renderIcon } from '@renderer/layouts/components/HeaderSection/helper';
import { useI18n } from 'vue-i18n';
import { h } from 'vue';

import { BrandWindows, Database, Terminal2, Star, History } from '@vicons/tabler';
import { RouterLink } from 'vue-router';

import type { MenuOption } from 'naive-ui';

export const menuOptions: MenuOption[] = () => {
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
  ];
};

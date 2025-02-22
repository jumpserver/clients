import mittBus from '@renderer/eventBus';

import { useI18n } from 'vue-i18n';
import { NPopselect, NIcon, NFlex, useMessage } from 'naive-ui';
import { ref, VNodeChild, watch, onMounted, defineComponent } from 'vue';

import { MdSettings } from '@vicons/ionicons4';
import { DownloadOutlined } from '@vicons/antd';
import { CicsSystemGroup } from '@vicons/carbon';
import { Layout2, Moon, Sun } from '@vicons/tabler';
import { GridViewRound, SortOutlined } from '@vicons/material';
import { ListUl, SortAlphaUp, SortAlphaUpAlt } from '@vicons/fa';
import { RefreshSharp, LanguageOutline } from '@vicons/ionicons5';
import { CalendarRtl48Filled, CalendarLtr48Filled } from '@vicons/fluent';

import { createLabel } from './index';
import { layoutOption, sortOption } from '../config';
import { useUserStore } from '@renderer/store/module/userStore';
import { useElectronConfig } from '@renderer/hooks/useElectronConfig';

import type { SelectOption } from 'naive-ui';

export const RightIconZone = defineComponent({
  name: 'RightIconZone',
  setup() {
    const { t, locale } = useI18n();
    const { getDefaultSetting } = useElectronConfig();
    const message = useMessage();
    const userStore = useUserStore();

    const currentTheme = ref('');
    const popSortSelectValue = ref('');
    const popLayoutSelectValue = ref('');

    const initSettings = async () => {
      try {
        const { layout, theme } = await getDefaultSetting();
        currentTheme.value = theme;
        popLayoutSelectValue.value = layout;
      } catch (error) {
        console.error('Failed to initialize settings:', error);
      }
    };

    onMounted(() => {
      initSettings();
    });

    watch(
      () => userStore.sort,
      type => {
        switch (type) {
          case 'name': {
            popSortSelectValue.value = 'name';
            break;
          }
          case '-name': {
            popSortSelectValue.value = '-name';
            break;
          }
          case '-date_updated': {
            popSortSelectValue.value = '-date_updated';
            break;
          }
          case 'date_updated': {
            popSortSelectValue.value = 'date_updated';
            break;
          }
        }
      },
      { immediate: true }
    );

    const handleUpdateValue = (value: string) => {
      popLayoutSelectValue.value = value;

      mittBus.emit('changeLayout', value);
    };

    const handleListSort = (value: string) => {
      userStore.setCurrentListSort(value);
    };

    const handleChangeTheme = async () => {
      const { theme } = await getDefaultSetting();

      mittBus.emit('changeTheme', theme as string);

      switch (theme) {
        case 'light': {
          currentTheme.value = 'dark';
          break;
        }
        case 'dark': {
          currentTheme.value = 'light';
          break;
        }
      }
    };

    const handleChangeLanguage = async () => {
      const { language } = await getDefaultSetting();

      mittBus.emit('changeLang', language as string);

      switch (language) {
        case 'zh': {
          locale.value = 'en';
          break;
        }
        case 'en': {
          locale.value = 'zh';
          break;
        }
      }
    };

    const handleRefresh = () => {
      const hasUser = userStore.currentUser && Reflect.ownKeys(userStore.currentUser).length > 0;

      if (!hasUser) {
        return message.error(t('Message.PleaseAuthFirst'));
      }

      return mittBus.emit('search', 'reset');
    };

    const handleGlobalSetting = () => {
      mittBus.emit('createDrawer');
    };

    const renderLabel = (option: SelectOption): VNodeChild => {
      switch (option.value) {
        case 'grid':
          return createLabel(GridViewRound, 'Grid');
        case 'list':
          return createLabel(ListUl, 'List');
        case 'name':
          return createLabel(SortAlphaUp, 'A-z');
        case '-name':
          return createLabel(SortAlphaUpAlt, 'Z-a');
        case '-date_updated':
          return createLabel(CalendarRtl48Filled, `${t('Common.NewToOld')}`);
        case 'date_updated':
          return createLabel(CalendarLtr48Filled, `${t('Common.OldToNew')}`);
        case 'import':
          return createLabel(DownloadOutlined, 'Import');
        case 'new-group':
          return createLabel(CicsSystemGroup, 'New Group');
      }
    };

    return () => (
      <NFlex class="gap-2 !flex-nowrap">
        <NPopselect
          size="small"
          trigger="click"
          class="rounded-xl"
          style={{ width: '8rem' }}
          options={layoutOption}
          renderLabel={renderLabel}
          value={popLayoutSelectValue.value}
          onUpdateValue={handleUpdateValue}
        >
          <NIcon size="20" component={Layout2} class="icon-hover" />
        </NPopselect>

        <NPopselect
          size="small"
          trigger="click"
          class="rounded-xl py-1 w-60"
          options={sortOption}
          renderLabel={renderLabel}
          value={popSortSelectValue.value}
          onUpdateValue={handleListSort}
        >
          <NIcon size="20" component={SortOutlined} class="icon-hover" />
        </NPopselect>

        {/* {currentTheme.value === 'light' ? (
          <NIcon size="20" component={Moon} class="icon-hover" onClick={handleChangeTheme} />
        ) : (
          <NIcon size="20" component={Sun} class="icon-hover" onClick={handleChangeTheme} />
        )} */}

        <NIcon size="20" component={RefreshSharp} class="icon-hover" onClick={handleRefresh} />

        {/* <NIcon
          size="20"
          component={LanguageOutline}
          class="icon-hover"
          onClick={handleChangeLanguage}
        /> */}

        <NIcon size="20" component={MdSettings} class="icon-hover" onClick={handleGlobalSetting} />
      </NFlex>
    );
  }
});

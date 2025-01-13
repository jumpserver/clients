import { ref, watch } from 'vue';
import { useUserStore } from '@renderer/store/module/userStore';
import { getAssets, getFavoriteAssets } from '@renderer/api/modals/asset';
import { useLoadingBar, useMessage } from 'naive-ui';
import { useI18n } from 'vue-i18n';
import type { IListItem } from '@renderer/components/MainSection/interface';

export function useAssetList(type: string) {
  const { t } = useI18n();
  const message = useMessage();
  const userStore = useUserStore();
  const loadingBar = useLoadingBar();

  const hasMore = ref(true);
  const loadingStatus = ref(true);
  const listData = ref<IListItem[]>([]);
  let typeObject = {};
  switch (type) {
    case 'linux':
      typeObject = { type: 'linux' };
      break;
    case 'windows':
      typeObject = { type: 'windows' };
      break;
    case 'databases':
      typeObject = { category: 'database' };
      break;
  }
  const params = ref({
    ...typeObject,
    offset: 0,
    limit: 20,
    search: '',
    order: userStore.sort
  });

  const handleScroll = async () => {
    if (!hasMore.value || loadingStatus.value) return;

    params.value.offset += 20;
    params.value.order = userStore.sort;

    try {
      await getAssetsFromServer();
    } catch (e) {
      message.error(`${t('Message.ListErrorOccurred')}`, { closable: true });
    }
  };

  const getAssetsFromServer = async (searchInput?: string) => {
    loadingBar.start();
    if (searchInput !== undefined) {
      params.value.offset = 0;
      params.value.search = searchInput;
      params.value.order = userStore.sort;
      hasMore.value = true;
    }

    if (searchInput === 'reset') {
      params.value.offset = 0;
      params.value.search = '';
      params.value.order = userStore.sort;
      listData.value = [];
      hasMore.value = true;
    }

    loadingStatus.value = true;

    try {
      const func = type === 'favorite' ? getFavoriteAssets : getAssets;
      const res = await func(params.value);

      if (res) {
        const { results, count: total } = res;

        if (params.value.offset === 0) {
          listData.value = results;
        } else {
          listData.value = [...listData.value, ...results];
        }

        hasMore.value = listData.value.length < total;
        loadingStatus.value = false;
      }
    } catch (e) {
      loadingStatus.value = false;
      hasMore.value = false;
      message.error(`${t('Message.FailedRetrieveAssetDataList')}`, { closable: true });
    } finally {
      loadingBar.finish();
    }
  };

  // 监听排序变化
  watch(
    () => userStore.sort,
    () => {
      params.value = {
        ...params.value,
        offset: 0,
        order: userStore.sort
      };
      listData.value = [];
      hasMore.value = true;
      getAssetsFromServer('reset');
    }
  );

  // 监听用户信息变化
  watch(
    () => userStore.userInfo,
    userInfo => {
      if (userInfo && userInfo.length === 0) {
        listData.value = [];
        userStore.setToken('');
      } else {
        getAssetsFromServer('reset');
      }
    },
    { immediate: true }
  );

  return {
    hasMore,
    loadingStatus,
    listData,
    handleScroll,
    getAssetsFromServer
  };
}

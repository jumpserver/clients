<template>
  <n-spin :show="loadingStatus" class="w-full h-[80%]">
    <MainSection
      :list-data="listData"
      :class="active ? 'show-drawer' : ''"
      @loadMore="handleScroll"
    />
  </n-spin>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useUserStore } from '@renderer/store/module/userStore';
import { getAssets } from '@renderer/api/modals/asset';
import { useMessage } from 'naive-ui';
import { useI18n } from 'vue-i18n';

import mittBus from '@renderer/eventBus';
import MainSection from '@renderer/components/MainSection/index.vue';
import type { IListItem } from '@renderer/components/MainSection/interface';

defineProps<{
  active: boolean;
}>();

const { t } = useI18n();
const message = useMessage();
const userStore = useUserStore();

const hasMore = ref(true);
const loadingStatus = ref(true);
const params = ref({
  type: 'windows',
  offset: 0,
  limit: 20,
  search: '',
  order: userStore.sort
});
const listData = ref<IListItem[]>([]);

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
  if (searchInput !== undefined) {
    params.value.offset = 0;
    params.value.search = searchInput;
    params.value.order = userStore.sort;
    listData.value = [];
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
    const res = await getAssets(params.value);

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
  }
};

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

watch(
  () => userStore.userInfo,
  userInfo => {
    if (userInfo && userInfo.length === 0) {
      listData.value = [];
      userStore.setToken('');
    } else {
      getAssetsFromServer();
    }
  },
  { immediate: true }
);

onMounted(() => {
  mittBus.on('search', getAssetsFromServer);
});

onBeforeUnmount(() => {
  mittBus.off('search', getAssetsFromServer);
});
</script>

<style scoped lang="scss">
:deep(.n-dropdown-option) {
  height: 40px;
}

.show-drawer {
  width: calc(100% - 340px);
}
</style>

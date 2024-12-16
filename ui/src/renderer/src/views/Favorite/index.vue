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
import { getFavoriteAssets } from '@renderer/api/modals/asset';
import { useMessage } from 'naive-ui';
import { useI18n } from 'vue-i18n';

import mittBus from '@renderer/eventBus';
import MainSection from '@renderer/components/MainSection/index.vue';

defineProps<{
  active: boolean;
}>();

const { t } = useI18n();
const message = useMessage();
const userStore = useUserStore();

const listData = ref([]);
const hasMore = ref(true);
const loadingStatus = ref(true);
const params = ref({
  offset: 0,
  limit: 20,
  search: '',
  order: userStore.sort
});

watch(
  () => userStore.sort,
  () => {
    params.value.order = userStore.sort;
    getAssetsFromServer();
  }
);

const handleScroll = async () => {
  if (!hasMore.value || loadingStatus.value) return;

  params.value.offset += 20;

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
    listData.value = [];
    hasMore.value = true;
  }

  loadingStatus.value = true;

  try {
    const res = await getFavoriteAssets(params.value);

    if (res) {
      const { results, count: total } = res;

      listData.value = params.value.offset === 0 ? results : [...listData.value, ...results];

      // 检查是否还有更多数据
      hasMore.value = listData.value.length < total;

      await nextTick(() => {
        loadingStatus.value = false;
      });
    }
  } catch (e) {
    hasMore.value = false;
    loadingStatus.value = false;
    message.error(`${t('Message.FailedRetrieveAssetDataList')}`, { closable: true });
  }
};

const handleRemoveAccount = () => {
  const userInfo = userStore.userInfo;

  if (userInfo && userInfo.length === 0) {
    listData.value = [];
    userStore.setToken('');
  }
};

onMounted(async () => {
  await getAssetsFromServer();

  mittBus.on('search', getAssetsFromServer);
  mittBus.on('removeAccount', handleRemoveAccount);
});

onBeforeUnmount(() => {
  mittBus.off('search', getAssetsFromServer);
  mittBus.off('removeAccount', handleRemoveAccount);
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

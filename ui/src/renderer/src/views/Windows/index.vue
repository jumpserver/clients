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
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { getAssets } from '@renderer/api/modals/asset';
import { useMessage } from 'naive-ui';

import mittBus from '@renderer/eventBus';
import MainSection from '@renderer/components/MainSection/index.vue';
import { useUserStore } from '@renderer/store/module/userStore';

defineProps<{
  active: boolean;
}>();

const message = useMessage();
const userStore = useUserStore();
const params = {
  type: 'windows',
  offset: 0,
  limit: 20,
  search: ''
};

const listData = ref([]);
const loadingStatus = ref(true);
const hasMore = ref(true);

const handleScroll = async () => {
  if (!hasMore.value || loadingStatus.value) return;
  params.offset += 20;
  await getAssetsFromServer();
};

const getAssetsFromServer = async (searchInput?: string) => {
  if (searchInput !== undefined) {
    params.offset = 0;
    params.search = searchInput;
    listData.value = [];
    hasMore.value = true;
  }

  loadingStatus.value = true;

  try {
    const res = await getAssets(params);
    if (res) {
      const { results, total } = res;
      listData.value = params.offset === 0 ? results : [...listData.value, ...results];
      // 检查是否还有更多数据
      hasMore.value = listData.value.length < total;
      await nextTick(() => {
        loadingStatus.value = false;
      });
    }
  } catch (e) {
    loadingStatus.value = false;
    hasMore.value = false;
    message.error('获取资产数据列表失败', { closable: true });
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

<template>
  <n-flex vertical justify="space-between" class="py-[15px]" style="height: calc(100% - 30px)">
    <n-flex>
      <n-menu
        :options="menuOptions"
        default-value="linux-page"
        class="w-full flex flex-col items-center"
      />
      <n-divider class="!my-[10px]" />
    </n-flex>

    <n-flex align="center" justify="center" class="!flex-nowrap">
      <n-avatar
        v-if="currentUser?.avatar_url"
        round
        size="medium"
        class="cursor-pointer"
        :src="currentUser?.avatar_url"
      />

      <div v-if="userOptions.length > 0" class="flex flex-col w-[60%]">
        <div class="flex w-full">
          <n-text depth="1" strong class="!inline-flex !items-center justify-between w-full">
            {{ currentUser?.username }}

            <n-popselect
              trigger="click"
              placement="right-end"
              class="w-[300px] rounded-[10px]"
              :options="userOptions"
              :render-label="renderLabel"
              v-model:value="currentUser!.username"
              @update:value="handleAccountChange"
            >
              <n-popover>
                <template #trigger>
                  <n-icon
                    size="14"
                    :component="ArrowsHorizontal"
                    class="ml-[10px] cursor-pointer icon-hover"
                  />
                </template>
                切换账号
              </n-popover>
              <template #action>
                <n-button text class="w-[50%]" @click="handleAddAccount"> 新增账号</n-button>
                <n-button text class="w-[50%]" @click="handleRemoveAccount"> 移除账号</n-button>
              </template>
            </n-popselect>
          </n-text>
        </div>
        <div style="font-size: 12px">
          <n-popover>
            <template #trigger>
              <!-- 默认只展示第一个 -->
              <n-text depth="2">
                {{ currentUser?.display_name?.[0] ?? '' }}
              </n-text>
            </template>
            {{ currentUser?.display_name }}
          </n-popover>
        </div>
      </div>
    </n-flex>
  </n-flex>
</template>

<script setup lang="ts">
import { ArrowsHorizontal } from '@vicons/carbon';
import type { SelectOption, SelectRenderLabel } from 'naive-ui';
import { NAvatar, NText } from 'naive-ui';
import { menuOptions } from './config';

import { useUserStore } from '@renderer/store/module/userStore';
import { computed, h, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import type { IUserInfo } from '@renderer/store/interface';
import mittBus from '@renderer/eventBus';

const router = useRouter();
const userStore = useUserStore();

const { currentUser: storeCurrentUser } = storeToRefs(userStore);

const currentUser = computed(() => storeCurrentUser?.value);

const userOptions = computed(() => {
  return (
    // @ts-ignore
    userStore?.userInfo.map((item: IUserInfo) => {
      return {
        label: item.username,
        value: item.username,
        avatar_url: item.avatar_url,
        display_name: item.display_name
      };
    }) || []
  );
});

const handleAccountChange = (value: string, _option: SelectOption) => {
  if (userStore.userInfo) {
    const user = userStore.userInfo.find((item: IUserInfo) => item.username === value);

    user ? userStore.setToken(user.token) : '';

    user ? userStore.setCurrentUser({ ...user }) : '';
  }
};

const renderLabel: SelectRenderLabel = option => {
  return h(
    'div',
    {
      style: {
        display: 'flex',
        alignItems: 'center'
      }
    },
    [
      h(NAvatar, {
        //@ ts-ignore
        src: option.avatar_url as string,
        round: true,
        size: 'medium',
        style: {
          cursor: 'pointer'
        }
      }),
      h(
        'div',
        {
          style: {
            marginLeft: '12px',
            padding: '4px 0'
          }
        },
        [
          h('div', null, [option.label as string]),
          h(
            NText,
            { depth: 3, tag: 'div' },
            {
              default: () => option.display_name
            }
          )
        ]
      )
    ]
  );
};

const handleAddAccount = () => {
  mittBus.emit('addAccount');
};

const handleRemoveAccount = () => {
  mittBus.emit('removeAccount');
};

onMounted(() => {
  router.push({ name: 'Linux' });
});
</script>

<style scoped lang="scss">
@use './index.scss';
</style>

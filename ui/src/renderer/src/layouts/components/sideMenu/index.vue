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
      <template v-if="userOptions.length === 0">
        <n-button text strong class="flex w-full h-8"> 未登录 </n-button>
      </template>
      <template v-else>
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
                v-model:value="currentUser!.token"
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
                  <n-button text class="w-1/2" @click="handleAddAccount"> 新增账号 </n-button>
                  <n-button text class="w-1/2" @click="handleRemoveAccount"> 移除账号 </n-button>
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

              <template #default>
                <span v-for="item of currentUser?.display_name" :key="item">
                  {{ item }}
                </span>
              </template>
            </n-popover>
          </div>
        </div>
      </template>
    </n-flex>
  </n-flex>
</template>

<script setup lang="ts">
import { NAvatar, NText, NPopover, NTag } from 'naive-ui';
import { ArrowsHorizontal } from '@vicons/carbon';
import { menuOptions } from './config';
import { computed } from 'vue';

import mittBus from '@renderer/eventBus';

import { useUserStore } from '@renderer/store/module/userStore';
import { h, nextTick, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';

import type { SelectOption, SelectRenderLabel } from 'naive-ui';
import type { IUserInfo } from '@renderer/store/interface';

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
        value: item.token,
        avatar_url: item.avatar_url,
        display_name: item.display_name
      };
    }) || []
  );
});

/**
 * @description 切换账号的逻辑
 * @param value
 * @param _option
 */
const handleAccountChange = (value: string, _option: SelectOption) => {
  if (userStore.userInfo) {
    const user = userStore.userInfo.find((item: IUserInfo) => item.token === value);

    if (user) {
      userStore.setToken(user.token);
      userStore.setCurrentUser({ ...user });
      userStore.setCurrentSit(user.currentSite as string);

      nextTick(() => {
        mittBus.emit('search');
      });
    }
  }
};

/**
 * @description popSelect 的自定义渲染
 * @param option
 */
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
            NTag,
            {
              bordered: false,
              size: 'small',
              type: 'info'
            },
            {
              default: () => `来源：${option.currentSite}`
            }
          )
        ]
      )
    ]
  );
};

/**
 * @description 添加账号
 */
const handleAddAccount = () => {
  mittBus.emit('addAccount');
};

/**
 * @description 移除账号
 */
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

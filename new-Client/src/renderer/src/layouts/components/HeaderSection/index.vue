<template>
  <n-flex
    justify="center"
    align="center"
    vertical
    class="px-[20px] pt-[20px] pb-[5px] border-b border-primary bg-primary"
    :class="active ? 'show-drawer' : ''"
  >
    <n-input-group>
      <n-input
        clearable
        class="rounded-[10px]"
        size="medium"
        placeholder="Find a asset"
        v-model:value="searchInput"
        @keypress.native.enter="onKeyEnter"
      />
      <n-button type="primary" secondary round class="rounded-[10px]" @click="handleSearch">
        搜索
      </n-button>
    </n-input-group>

    <n-flex justify="flex-end" align="center" class="w-full h-[40px]">
      <!-- New Host 部分，暂不实现 -->
      <n-flex class="h-full" align="center">
        <!--<n-button-group size="small">-->
        <!--  <n-button class="rounded-[5px] h-[25px]" @click="handleNewHost">-->
        <!--    <template #icon>-->
        <!--      <n-icon size="16" :component="Server" />-->
        <!--    </template>-->
        <!--    <n-text depth="1" strong class="text-[12px] text-[inherit]"> NEW HOST </n-text>-->
        <!--  </n-button>-->
        <!--  <n-popselect-->
        <!--    trigger="click"-->
        <!--    placement="bottom-start"-->
        <!--    style="width: 240px"-->
        <!--    :options="createOption"-->
        <!--    :render-label="renderLabel"-->
        <!--    class="bg-[#292B3C] rounded-[10px] py-[5px]"-->
        <!--          >-->
        <!--    <n-button-->
        <!--      class="rounded-[5px] h-[25px]"-->
        <!--      style="padding: 0 3px"-->
        <!--      @click="showCreateOption"-->
        <!--            >-->
        <!--      <template #icon>-->
        <!--        <n-icon size="16" :component="arrowDown" />-->
        <!--        &lt;!&ndash;              <n-icon size="16" :component="arrowUp" />&ndash;&gt;-->
        <!--      </template>-->
        <!--    </n-button>-->
        <!--  </n-popselect>-->
        <!--</n-button-group>-->
      </n-flex>

      <!-- 右侧 icon  -->
      <n-flex class="h-full" align="center">
        <!--        <n-popselect-->
        <!--          v-model:value="popLayoutSelectValue"-->
        <!--          :options="tagOption"-->
        <!--          trigger="click"-->
        <!--          placement="bottom-end"-->
        <!--          class="rounded-[10px] !px-[5px] !py-[10px] w-[250px]"-->
        <!--        >-->
        <!--          <n-icon-->
        <!--            :component="Tag"-->
        <!--            size="20"-->
        <!--            class="cursor-pointer hover:text-[#4C917D] duration-300 transition-all ease-in-out"-->
        <!--          />-->
        <!--        </n-popselect>-->

        <!-- Layout  -->
        <n-popselect
          trigger="click"
          style="width: 150px"
          :options="layoutOption"
          :render-label="renderLabel"
          class="rounded-[10px] py-[5px]"
          v-model:value="popLayoutSelectValue"
          @update:value="handleUpdateLayoutValue"
        >
          <n-icon size="20" :component="Layout2" class="icon-hover" />
        </n-popselect>

        <!-- Sort  -->
        <n-popselect
          trigger="click"
          style="width: 200px"
          :options="sortOption"
          :render-label="renderLabel"
          class="rounded-[10px] py-[5px]"
          v-model:value="popSortSelectValue"
        >
          <n-icon size="20" :component="SortOutlined" class="icon-hover" />
        </n-popselect>

        <n-icon
          v-if="currentTheme === 'light'"
          :component="Moon"
          size="20"
          class="icon-hover"
          @click="handleChangeTheme"
        />
        <n-icon v-else :component="Sun" size="20" class="icon-hover" @click="handleChangeTheme" />

        <!-- Setting -->
        <n-icon :component="MdSettings" size="20" class="icon-hover" @click="handleGlobalSetting" />
      </n-flex>
    </n-flex>
  </n-flex>
</template>

<script setup lang="ts">
import { VNodeChild } from 'vue';
import type { SelectOption } from 'naive-ui';

import { createLabel } from './helper';
import { ref } from 'vue';

import { Conf } from 'electron-conf/renderer';

import { layoutOption, sortOption } from './config/index';

import { CalendarRtl48Filled, CalendarLtr48Filled } from '@vicons/fluent';
import { ListUl, SortAlphaUp, SortAlphaUpAlt } from '@vicons/fa';
import { GridViewRound, SortOutlined } from '@vicons/material';
import { Layout2, Moon, Sun } from '@vicons/tabler';
import { CicsSystemGroup } from '@vicons/carbon';
import { DownloadOutlined } from '@vicons/antd';
import { MdSettings } from '@vicons/ionicons4';
import { NFlex, NIcon } from 'naive-ui';

// import { Tag } from '@vicons/tabler';

import mittBus from '@renderer/eventBus';

defineProps<{
  active: boolean;
}>();

const conf = new Conf();

const currentTheme = ref('');
const popSortSelectValue = ref('');
const popLayoutSelectValue = ref('');
const searchInput = ref('');

conf.get('defaultSetting').then(res => {
  if (res) {
    // @ts-ignore
    popLayoutSelectValue.value = res.layout;
    // @ts-ignore
    currentTheme.value = res.theme;
  }
});

/**
 * @description 自定义 label 渲染
 * @param option
 */
const renderLabel = (option: SelectOption): VNodeChild => {
  switch (option.value) {
    case 'grid':
      return createLabel(GridViewRound, 'Grid');
    case 'list':
      return createLabel(ListUl, 'List');
    case 'A-z':
      return createLabel(SortAlphaUp, 'A-z');
    case 'Z-a':
      return createLabel(SortAlphaUpAlt, 'Z-a');
    case 'new-to-old':
      return createLabel(CalendarRtl48Filled, 'Newest to oldest');
    case 'old-to-new':
      return createLabel(CalendarLtr48Filled, 'Oldest to newest');
    case 'import':
      return createLabel(DownloadOutlined, 'Import');
    case 'new-group':
      return createLabel(CicsSystemGroup, 'New Group');
  }
};

/**
 * @description 更新布局
 *
 * @param value
 * @param _option
 */
const handleUpdateLayoutValue = async (value: string, _option: SelectOption) => {
  popLayoutSelectValue.value = value;

  if (value) {
    mittBus.emit('changeLayout', value as string);
  }
};

/**
 * @description 更新主题
 */
const handleChangeTheme = () => {
  let theme = '';

  conf.get('defaultSetting').then(res => {
    if (res) {
      // @ts-ignore
      theme = res.theme;

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
    }
  });
};

const handleSearch = () => {
  mittBus.emit('search', searchInput.value);
};

// @ts-ignore
const handleNewHost = () => {
  mittBus.emit('createDrawer');
};

const handleGlobalSetting = () => {
  // showModal.value = true;
  mittBus.emit('createDrawer');
};

const onKeyEnter = (event: KeyboardEvent) => {
  if (!event.shiftKey || event.ctrlKey) {
    event.preventDefault();
    mittBus.emit('search', searchInput.value);
  }
};
</script>

<style scoped lang="scss">
@use './index.scss';
</style>

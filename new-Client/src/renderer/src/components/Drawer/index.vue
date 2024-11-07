<template>
  <n-drawer
    v-model:show="props.active"
    placement="right"
    to="#drawer-target"
    :width="350"
    :show-mask="false"
    :trap-focus="false"
    :block-scroll="false"
    :mask-closable="false"
    class="!rounded-[unset] bg-primary border-l-primary border-l"
  >
    <n-drawer-content footer-style="border: unset;" body-content-style="padding: 15px 5px">
      <template #header>
        <n-flex align="center" justify="space-between">
          <n-text depth="1">默认配置</n-text>

          <n-flex>
            <n-icon
              size="20"
              :component="ArrowBarRight"
              class="icon-hover rounded-[5px]"
              @click="closeDrawer"
            />
          </n-flex>
        </n-flex>
      </template>

      <n-scrollbar>
        <template #default>
          <n-flex class="mx-[15px]">
            <n-card
              v-for="item of currentOption"
              :bordered="false"
              :key="item.value"
              size="small"
              header-style="font-size: 13px;"
              class="rounded-[10px] !bg-secondary"
            >
              <n-collapse
                :accordion="true"
                :trigger-areas="['extra']"
                :default-expanded-names="enabledItems"
                :expanded-names="expandedNames"
              >
                <n-collapse-item :name="item.label" :title="item.label">
                  <template #header-extra>
                    <n-switch
                      size="small"
                      v-model:value="item.form.enable"
                      @update:value="handleSwitchValueChange(item)"
                    >
                      <template #checked> 已启用 </template>
                      <template #unchecked> 未启用 </template>
                    </n-switch>
                  </template>
                  <n-form :model="item.form">
                    <n-form-item label="应用路径:" label-style="font-size: 13px">
                      <n-input-group>
                        <n-input v-model:value="item.form.path" size="small" disabled />
                        <n-button type="primary" ghost size="small" disabled>
                          <n-icon :component="Folder28Regular" size="14" />
                        </n-button>
                      </n-input-group>
                    </n-form-item>
                    <n-form-item
                      label="应用说明:"
                      label-style="font-size: 13px"
                      label-placement="left"
                    >
                      <n-popover>
                        <template #trigger>
                          <span class="truncate">
                            {{ item.form.textarea }}
                          </span>
                        </template>
                        {{ item.form.textarea }}
                      </n-popover>
                    </n-form-item>
                    <n-form-item
                      label="下载地址:"
                      label-style="font-size: 13px"
                      feedback-style="min-height: unset"
                      label-placement="left"
                    >
                      <n-popover>
                        <template #trigger>
                          <n-a
                            href="#"
                            class="truncate"
                            @click.prevent="copyToClipboard(item.form.downLoadUrl)"
                          >
                            {{ item.form.downLoadUrl }}
                          </n-a>
                        </template>
                        {{ item.form.downLoadUrl }}
                      </n-popover>
                    </n-form-item>
                  </n-form>
                </n-collapse-item>
              </n-collapse>
            </n-card>

            <n-card
              :bordered="false"
              size="small"
              header-style="font-size: 13px;"
              class="rounded-[10px] !bg-secondary"
            >
              <n-collapse>
                <n-collapse-item title="高级选项">
                  <n-form>
                    <n-form-item label="字符集:" label-style="font-size: 13px">
                      <n-select size="small" />
                    </n-form-item>
                    <n-form-item
                      label="字符终端 Backspace As Ctrl + H"
                      label-style="font-size: 13px"
                    >
                      <n-select size="small" />
                    </n-form-item>
                  </n-form>
                </n-collapse-item>
              </n-collapse>
            </n-card>
          </n-flex>
        </template>
      </n-scrollbar>
    </n-drawer-content>
  </n-drawer>
</template>

<script setup lang="ts">
import mittBus from '@renderer/eventBus';

import { linuxOptions, windowsOptions, databaseOptions } from './config/index';
import { Folder28Regular } from '@vicons/fluent';
import { ArrowBarRight } from '@vicons/tabler';

import { nextTick, ref, watch, computed } from 'vue';
import { useMessage } from 'naive-ui';
import { useRoute } from 'vue-router';

import type { ICustomSelectOption } from './config/index';
import type { Ref } from 'vue';

const currentOption: Ref<ICustomSelectOption[] | null> = ref(null);

const props = withDefaults(defineProps<{ active: boolean }>(), { active: false });

const route = useRoute();
const message = useMessage();

watch(
  () => route.name,
  newValue => {
    switch (newValue) {
      case 'Linux':
        currentOption.value = linuxOptions.value;
        break;
      case 'Windows':
        currentOption.value = windowsOptions.value;
        break;
      case 'Database':
        currentOption.value = databaseOptions.value;
        break;
      default:
        currentOption.value = [];
    }
  },
  {
    immediate: true
  }
);

const enabledItems = computed(() => {
  // 获取所有启用状态的 item 标签作为展开的名称
  return currentOption.value
    ? currentOption.value.filter(item => item.form.enable).map(item => item.label)
    : [];
});

const expandedNames = computed(() => {
  const enabledItem = currentOption.value?.find(item => item.form.enable);

  return enabledItem ? [enabledItem.label] : [];
});

const copyToClipboard = (text: string) => {
  navigator.clipboard
    .writeText(text)
    .then(() => message.success('已复制到剪切板'))
    .catch();
};

/**
 * @description 关闭抽屉
 */
const closeDrawer = () => {
  mittBus.emit('createDrawer');
};

/**
 * @deprecated 当只有一个处于启用状态时不允许关闭当前 Switch，只有点击启用其他 Option 时该状态关闭
 *
 * @param item
 */
const handleSwitchValueChange = (item: ICustomSelectOption) => {
  nextTick(() => {
    const enabledOptions = currentOption.value?.filter(option => option.form.enable);

    if (enabledOptions && enabledOptions.length === 0) {
      message.warning('请至少启用一个选项');
      item.form.enable = true;

      return;
    }

    // 当前的为 true 其他的为 false
    if (enabledOptions && enabledOptions.length > 0) {
      item.form.enable = true;

      currentOption.value?.forEach(option => {
        if (option !== item) {
          option.form.enable = false;
        }
      });
    }
  });
};
</script>

<style lang="scss" scoped>
:deep(.n-form-item-feedback-wrapper) {
  min-height: 15px;
}

:deep(.n-collapse-item-arrow) {
  display: none !important;
}
</style>

import { NIcon, NFlex, NAvatar, NText, NButton, NPopover } from 'naive-ui';
import { RemoveCircleOutline } from '@vicons/ionicons5';
import type { Component } from 'vue';
import type { SelectRenderLabel } from 'naive-ui';

export const renderIcon = (icon: Component) => {
  return <NIcon component={icon} />;
};

export const createAccountRenderLabel = (onRemove: (token: string) => void): SelectRenderLabel => {
  return option => {
    const handleRemove = (e: Event) => {
      e.stopPropagation();

      onRemove(option.value as string);
    };

    return (
      <div class="w-full px-2 py-2 group hover:bg-primary-100 dark:hover:bg-primary-900">
        <NFlex align="center" justify="space-between" class="w-36.5">
          {/* 左侧 */}
          <NFlex align="center" justify="start" class="gap-0">
            <NAvatar src={option.avatar_url as string} round size="medium" class="flex-shrink-0" />

            <NFlex vertical class="!gap-0">
              <NText class="whitespace-nowrap overflow-hidden text-ellipsis">{option.label}</NText>
              <NText depth={3} class="text-xs whitespace-nowrap overflow-hidden text-ellipsis">
                {option.display_name}
              </NText>
            </NFlex>
          </NFlex>

          {/* 右侧 */}
          <NPopover trigger="hover">
            {{
              trigger: () => (
                <NButton
                  text
                  onClick={handleRemove}
                  class="absolute right-8 opacity-0 group-hover:opacity-100 transition-opacity duration-300"
                >
                  <NIcon size={20} class="text-red-500" component={RemoveCircleOutline} />
                </NButton>
              ),
              default: () => '删除'
            }}
          </NPopover>
        </NFlex>
      </div>
    );
  };
};

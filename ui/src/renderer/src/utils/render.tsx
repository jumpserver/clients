import { NIcon, NFlex, NAvatar, NText, NButton } from 'naive-ui';
import { RemoveCircleOutline } from '@vicons/ionicons5';
import type { Component } from 'vue';
import type { SelectRenderLabel } from 'naive-ui';

export const renderIcon = (icon: Component) => {
  return <NIcon component={icon} />;
};

export const accountRenderLabel: SelectRenderLabel = option => {
  return (
    <NFlex align="center" justify="space-between" class="group w-full">
      {/* 左侧 */}
      <NFlex align="center" justify="start" class="flex-1 w-4/5">
        <NAvatar src={option.avatar_url as string} round size="medium" />

        <NFlex class="!flex-col">
          <NText>{option.username}</NText>
          <NText>{option.display_name}</NText>
        </NFlex>
      </NFlex>

      {/* 右侧 */}
      <NButton text class="hidden group-hover:block">
        <NIcon size={24} component={RemoveCircleOutline} />
      </NButton>
    </NFlex>
  );
};

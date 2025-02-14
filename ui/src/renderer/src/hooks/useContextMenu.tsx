import type { Ref } from 'vue';
import type { DropdownOption, ConfigProviderProps } from 'naive-ui';
import type { IListItem, IItemDetail } from '@renderer/components/MainSection/interface';
import type { Permed_protocols, Permed_accounts } from '@renderer/components/MainSection/interface';

import mittBus from '@renderer/eventBus';

import {
  NFlex,
  NText,
  useLoadingBar,
  NDescriptions,
  NDescriptionsItem,
  lightTheme,
  darkTheme,
  createDiscreteApi
} from 'naive-ui';
import { useI18n } from 'vue-i18n';
import { ref, computed } from 'vue';
import { useElectronConfig } from './useElectronConfig';
import { useAssetStore } from '@renderer/store/module/assetStore';
import { createConnectToken, getAssetDetail, getLocalClientUrl } from '@renderer/api/modals/asset';

import { Link, Eye, FileText, UsersRound, UserRoundCheck, Check } from 'lucide-vue-next';

export const useContextMenu = () => {
  const { t } = useI18n();
  const { getDefaultSetting } = useElectronConfig();

  const loadingBar = useLoadingBar();
  const assetStore = useAssetStore();

  const selectedAccount = ref('');
  const selectedProtocol = ref('');

  const defaultTheme = ref('');
  const detailMessage: Ref<Partial<IItemDetail>> = ref({});
  const rightOptions: Ref<DropdownOption[]> = ref([
    {
      label: t('Common.QuickConnect'),
      key: 'connect-direction',
      icon: () => <Link size={16} />
    },
    {
      label: t('Common.AssetDetails'),
      icon: () => <Eye size={16} />,
      key: 'asset-details'
    },
    {
      type: 'divider',
      key: 'd1'
    },
    {
      label: t('Common.ConnectionProtocol'),
      key: 'optional-protocol',
      icon: () => <FileText size={16} />,
      children: []
    },
    {
      label: t('Common.AccountList'),
      key: 'available-account',
      icon: () => <UsersRound size={16} />,
      children: []
    }
  ]);

  getDefaultSetting().then(res => {
    if (res) {
      defaultTheme.value = res.theme;
    }
  });

  const configProviderPropsRef = computed<ConfigProviderProps>(() => ({
    theme: defaultTheme.value === 'light' ? lightTheme : darkTheme
  }));

  const { modal } = createDiscreteApi(['modal'], {
    configProviderProps: configProviderPropsRef
  });

  const preCheck = async (): Promise<boolean> => {
    return Promise.resolve(true);
  };

  const createModal = () => {
    modal.create({
      title: t('Common.AssetDetails'),
      preset: 'card',
      bordered: false,
      segmented: true,
      style: {
        width: '40rem',
        borderRadius: '10px'
      },
      content: () => {
        return (
          <NDescriptions label-placement="left" column={1} bordered>
            <NDescriptionsItem label={t('Common.PlatformInfo')}>
              {t('Common.PlatformID')}: ${detailMessage?.value?.platform.id || ''}
              <br />
              {t('Common.PlatformName')}: ${detailMessage?.value?.platform.name || ''}
            </NDescriptionsItem>
            <NDescriptionsItem label={t('Common.PlatformID')}>
              {detailMessage?.value?.platform.id || ''}
            </NDescriptionsItem>
            <NDescriptionsItem label={t('Common.PlatformName')}>
              {detailMessage.value.category.label}
            </NDescriptionsItem>
            <NDescriptionsItem label={t('Common.Connectivity')}>
              {detailMessage.value.connectivity.label}
            </NDescriptionsItem>
            <NDescriptionsItem label={t('Common.Category')}>
              {detailMessage.value.nodes.map(node => {
                return <div key={node.id}>{`${node.name} (ID: ${node.id})`}</div>;
              })}
            </NDescriptionsItem>
            <NDescriptionsItem label={t('Common.Nodes')}>
              {detailMessage.value.permed_protocols.map(protocol => {
                return <div key={protocol.name}>{`${protocol.name} (Port: ${protocol.port})`}</div>;
              })}
            </NDescriptionsItem>
            <NDescriptionsItem label={t('Common.Protocols')}>
              {detailMessage.value.permed_accounts.map(account => {
                return (
                  <div
                    key={account.id}
                  >{`${account.alias} (${account.username || t('Common.NoUsername')})`}</div>
                );
              })}
            </NDescriptionsItem>
          </NDescriptions>
        );
      }
    });
  };

  const getAssetDetailMessage = async (
    item: IListItem,
    event: MouseEvent
  ): Promise<boolean | string> => {
    loadingBar.start();
    event.stopPropagation();

    try {
      const assetDetail: IItemDetail = await getAssetDetail(item.id);

      detailMessage.value = assetDetail;

      if (assetDetail) {
        // prettier-ignore
        const accountMenuItem: DropdownOption = rightOptions.value.find( option => option.key === 'available-account')!;
        // prettier-ignore
        const protocolMenuItem: DropdownOption = rightOptions.value.find(option => option.key === 'optional-protocol')!;

        selectedProtocol.value = assetStore.getAssetMap(assetDetail.id!)?.protocol?.key as string;
        selectedAccount.value = assetStore.getAssetMap(assetDetail.id!)?.account?.key as string;

        const hoverBackground = defaultTheme.value === 'light' ? '#F3F3F5' : '#FFFFFF17';

        protocolMenuItem!.children = assetDetail.permed_protocols
          .filter((protocol: Permed_protocols) => protocol.public)
          .map(protocol => ({
            type: 'render',
            render: () => {
              return (
                <NFlex
                  justify="space-between"
                  align="center"
                  class={`w-full px-4 py-1 cursor-pointer hover:bg-[${hoverBackground}]`}
                >
                  <NText>{protocol.name}</NText>

                  {selectedProtocol.value === protocol.name && (
                    <Check
                      size={14}
                      color={defaultTheme.value === 'light' ? '#18a058' : '#63e2b7'}
                    />
                  )}
                </NFlex>
              );
            },
            key: protocol.name,
            label: protocol.name
          })) as DropdownOption[];

        accountMenuItem!.children = assetDetail.permed_accounts
          .filter((account: Permed_accounts) => account.alias !== '@ANON')
          .map(account => ({
            type: 'render',
            render: () => {
              return (
                <NFlex
                  justify="space-between"
                  align="center"
                  class={`!flex-nowrap w-full px-4 py-1 cursor-pointer hover:bg-[${hoverBackground}]`}
                >
                  <NText> 
                    {account.name +
                      (account.alias === account.username || account.alias.startsWith('@')
                        ? ''
                        : '(' + account.username + ')')}
                  </NText>

                  {selectedAccount.value === account.id && (
                    <UserRoundCheck
                      size={14}
                      color={defaultTheme.value === 'light' ? '#18a058' : '#63e2b7'}
                    />
                  )}
                </NFlex>
              );
            },
            label: account.name,
            key: account.id
          })) as DropdownOption[];

        // 检测之前是否修改过默认的 account 和 protocol。 如果修改过，则返回 false，反之为 true
        const checked = await preCheck();

        if (checked) {
          assetStore.setAssetMap(detailMessage.value.id!, {
            account: accountMenuItem.children[0] as { accountId: string; accountLabel: string },
            protocol: protocolMenuItem.children[0] as { protocolId: string; protocolLabel: string }
          });
        }
      }

      return Promise.resolve(detailMessage.value.id!);
    } catch (e) {
      return Promise.resolve(false);
    } finally {
      loadingBar.finish();
    }
  };

  const handleOptionSelect = (key: string) => {
    switch (key) {
      case 'asset-details':
        createModal();
        break;
      case 'connect-direction':
        break;
      case 'optional-protocol':
        break;
    }
  };

  const handleThemeChange = (theme: string) => {
    switch (theme) {
      case 'light':
        defaultTheme.value = 'dark';
        break;
      case 'dark':
        defaultTheme.value = 'light';
        break;
    }
  };

  mittBus.on('changeTheme', handleThemeChange);

  return {
    rightOptions,
    handleOptionSelect,
    getAssetDetailMessage
  };
};

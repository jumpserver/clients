import type { Ref } from 'vue';
import type { DropdownOption, ConfigProviderProps } from 'naive-ui';
import type { IListItem, IItemDetail } from '@renderer/components/MainSection/interface';

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
import { ref, reactive, computed } from 'vue';
import { useElectronConfig } from './useElectronConfig';
import { createConnectToken, getAssetDetail, getLocalClientUrl } from '@renderer/api/modals/asset';

import { Link, Eye, FileText, UsersRound, UserRoundCheck, Check } from 'lucide-vue-next';

export const useContextMenu = () => {
  const { t } = useI18n();
  const { getDefaultSetting } = useElectronConfig();

  const loadingBar = useLoadingBar();

  const selectedAccount = reactive({ id: '', label: '' });
  const selectedProtocol = reactive({ key: '', label: '' });

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

  // TODO: 优化代码
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

  const getAssetDetailMessage = async (item: IListItem, event: MouseEvent): Promise<boolean> => {
    loadingBar.start();
    event.stopPropagation();

    try {
      const assetDetail: IItemDetail = await getAssetDetail(item.id);

      detailMessage.value = assetDetail;

      if (assetDetail) {
        // prettier-ignore
        const accountMenuItem = rightOptions.value.find( option => option.key === 'available-account');
        // prettier-ignore
        const protocolMenuItem = rightOptions.value.find(option => option.key === 'optional-protocol');

        const checked = await preCheck();

        if (checked) {
          selectedAccount.id = assetDetail.permed_accounts[0].id;
          selectedAccount.label = assetDetail.permed_accounts[0].name;

          selectedProtocol.key = assetDetail.permed_protocols[0].name;
          selectedProtocol.label = assetDetail.permed_protocols[0].name;
        }

        protocolMenuItem!.children = assetDetail.permed_protocols.map(protocol => ({
          type: 'render',
          render: () => {
            return (
              <NFlex
                justify="space-between"
                align="center"
                class="w-full px-4 py-1 cursor-pointer hover:bg-[#F3F3F5]"
              >
                <NText>{protocol.name}</NText>

                {selectedProtocol.label === protocol.name && (
                  <Check size={14} color={defaultTheme.value === 'light' ? '#18a058' : '#63e2b7'} />
                )}
              </NFlex>
            );
          },
          key: protocol.name,
          label: protocol.name
        }));

        accountMenuItem!.children = assetDetail.permed_accounts.map(account => ({
          type: 'render',
          render: () => {
            return (
              <NFlex
                justify="space-between"
                align="center"
                class="w-full px-4 py-1 cursor-pointer hover:bg-[#FFFFFF17]"
              >
                <NText>{account.name}</NText>

                {selectedAccount.id === account.id && (
                  <UserRoundCheck
                    size={14}
                    color={defaultTheme.value === 'light' ? '#18a058' : '#63e2b7'}
                  />
                )}
              </NFlex>
            );
          },
          key: account.id
        }));
      }

      return Promise.resolve(true);
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

  return {
    rightOptions,
    handleOptionSelect,
    getAssetDetailMessage
  };
};

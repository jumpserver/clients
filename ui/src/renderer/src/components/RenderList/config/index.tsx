import { h } from 'vue';
import { useI18n } from 'vue-i18n';
import { NTag, NButton, NButtonGroup } from 'naive-ui';
import { CircleX, CircleCheck, Cable, EllipsisVertical } from 'lucide-vue-next';

import type { DataTableColumns } from 'naive-ui';
import type { IListItem } from '@renderer/components/MainSection/interface';

export const getColumns = (): DataTableColumns<IListItem> => {
  const { t } = useI18n();

  return [
    {
      title: t('Common.AssetName'),
      key: 'name',
      width: 200
    },
    {
      title: t('Common.Organization'),
      key: 'org_name',
      width: 200,
      render: row => {
        return (
          <NTag bordered={false} round type="info" size="small" class="tracking-widest">
            {row.org_name}
          </NTag>
        );
      }
    },
    {
      title: t('Common.Address'),
      key: 'address'
    },
    {
      title: t('Common.Connectability'),
      key: 'is_active',
      render: row => {
        return row.is_active ? <CircleCheck color="#18a058" /> : <CircleX color="#e88080" />;
      }
    },
    {
      title: 'Action',
      key: 'actions',
      align: 'center',
      render: _row => (
        <NButtonGroup class="items-center">
          <NButton round quaternary>
            <Cable size={16} class="mr-2" />
            连接
          </NButton>

          <NButton round size="small" quaternary>
            <EllipsisVertical size={16} />
          </NButton>
        </NButtonGroup>
      )
    }
  ];
};

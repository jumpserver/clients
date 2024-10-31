import type { SelectOption } from 'naive-ui';
import {
  renderCustomBody,
  renderCustomInput
} from '@renderer/layouts/components/HeaderSection/helper';

export const layoutOption: Array<SelectOption> = [
  {
    value: 'grid',
    label: 'Grid'
  },
  {
    value: 'list',
    label: 'List'
  }
];

export const sortOption: Array<SelectOption> = [
  {
    value: 'A-z',
    label: 'A-z'
  },
  {
    value: 'Z-a',
    label: 'Z-a'
  },
  {
    value: 'new-to-old',
    label: 'Newest to oldest'
  },
  {
    value: 'old-to-new',
    label: 'Oldest to newest'
  }
];

export const createOption: Array<SelectOption> = [
  {
    label: 'New Group',
    value: 'new-group'
  },
  {
    label: 'Import',
    value: 'import'
  }
];

export const tagOption: Array<SelectOption> = [
  {
    key: 'header',
    type: 'render',
    render: () => renderCustomInput([{ id: '1', label: '爱在西元前', isChecked: false }])
  },
  {
    key: 'tags',
    type: 'render',
    render: () => renderCustomBody([{ id: '1', label: '爱在西元前', isChecked: false }])
  },
  {
    key: 'header-divider',
    type: 'divider'
  },
  {
    label: 'Clear selection',
    type: 'button'
  }
];

export const themeOptions: Array<SelectOption> = [
  {
    label: 'Dark',
    value: 'dark'
  },
  {
    label: 'Light',
    value: 'light'
  }
];

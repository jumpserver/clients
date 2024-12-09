import type { Component } from 'vue';
import type { SelectOption } from 'naive-ui';

export interface IOperationZone {
  label: string;

  component: Component;

  width: string;

  size: string;

  options: SelectOption[];
}

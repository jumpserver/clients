import type { GlobalThemeOverrides } from 'naive-ui';

export const darkThemeOverrides: GlobalThemeOverrides = {
  Layout: {},
  Menu: {
    itemTextColorActive: '#fff'
  },
  Dropdown: {
    optionHeightMedium: '40px',
    borderRadius: '10px'
  }
};

export const lightThemeOverrides: GlobalThemeOverrides = {
  Layout: {},
  Menu: {},
  Dropdown: {
    optionHeightMedium: '40px',
    borderRadius: '10px'
  }
};

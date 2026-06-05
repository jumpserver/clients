export default defineAppConfig({
  app: {
    name: "JumpServer Client",
    author: "JumpServer",
    version: "4.0.0",
    repo: "https://github.com/jumpserver/clients"
  },
  componentsConfig: {
    header: {
      // 颜色现在通过 CSS 变量管理，在 main.css 中定义
      // 这里保留用于其他可能的配置
    },
    pages: {
      scrollBarLightThumbColor: "#D0D1D2",
      scrollBarDarkThumbColor: "#4A4A4A",
      scrollBarLightHoverColor: "#B8B9BA",
      scrollBarDarkHoverColor: "#6B6B6B",
      mainCardLightBackgroundColor: "#FAFAFA",
      mainCardDarkBackgroundColor: "#2C2C2C"
    },
    urlRegExp:
      /^(?:https?:\/\/(?:localhost|\d{1,3}(?:\.\d{1,3}){3}|\[[0-9a-fA-F:]+\]|(?:[a-zA-Z0-9-]+\.)+[a-zA-Z]{2,})(?::\d{1,5})?(?:[/?#]\S*)?|\d{1,3}(?:\.\d{1,3}){3}|\[[0-9a-fA-F:]+\])$/
  },
  ui: {
    colors: {
      primary: "primary",
      neutral: "zinc"
    },
    container: {
      base: "mx-0 w-full"
    },
    button: {
      slots: {
        base: "cursor-pointer rounded-app-md! font-semibold transition-colors duration-150"
      },
      variants: {
        ghost: {
          neutral: {
            base: "bg-transparent hover:bg-app-surface-2 dark:hover:bg-app-surface-2"
          }
        },
        outline: {
          neutral: {
            base: "border-app-hairline text-app-ink hover:bg-app-surface-2"
          }
        },
        soft: {
          primary: {
            base: "bg-primary-500/10 text-primary-600 hover:bg-primary-500/15 dark:text-primary-400"
          }
        }
      }
    },
    formField: {
      slots: {
        root: "w-full"
      }
    },
    input: {
      slots: {
        root: "w-full"
      }
    },
    textarea: {
      slots: {
        root: "w-full",
        base: "resize-none"
      }
    },
    accordion: {
      slots: {
        trigger: "cursor-pointer",
        item: "md:py-2"
      }
    },
    dropdownMenu: {
      slots: {
        content: "w-(--reka-dropdown-menu-trigger-width) rounded-app-md! border border-app-hairline bg-app-surface-1 p-1 text-app-ink shadow-lg",
        item: "mx-0.5 rounded-app-sm px-3 py-2 transition-colors duration-150"
      }
    },
    navigationMenu: {
      slots: {
        link: "cursor-pointer rounded-app-sm! transition-colors duration-150"
      },
      variants: {
        disabled: {
          true: {
            link: "cursor-text"
          }
        }
      }
    }
  }
});

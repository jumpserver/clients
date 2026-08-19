import type { AppConfigType } from "~/types";

export const useApplicationConfig = () => {
  const { setAppConfig, appConfig, hydrationPromise } = useSettingManager();

  const isValidAppConfig = (cfg: any): cfg is AppConfigType => {
    return (
      !!cfg
      && Array.isArray(cfg.terminal)
      && Array.isArray(cfg.remotedesktop)
      && Array.isArray(cfg.filetransfer)
      && Array.isArray(cfg.databases)
      && (cfg.terminal.length > 0
        || cfg.remotedesktop.length > 0
        || cfg.filetransfer.length > 0
        || cfg.databases.length > 0)
    );
  };

  const getConfig = async () => {
    try {
      const config = await useTauriCoreInvoke("get_config");

      if (config) {
        await setAppConfig(config as AppConfigType);
      } else {
        console.error("get_config returned empty config");
      }
    } catch (err) {
      console.error("get_config failed", err);
      throw err;
    }
  };

  onMounted(async () => {
    const cur = await useTauriWebviewWindowGetCurrentWebviewWindow();

    // Settings / secondary windows must always refresh from Rust.
    // Cached store data can be null/stale when the window opens before main
    // finishes persisting, which leaves SSH/RDP settings empty.
    if (cur && cur.label !== "main") {
      if (hydrationPromise.value) {
        try {
          await hydrationPromise.value;
        } catch {}
      }

      try {
        await getConfig();
      } catch {
        if (!isValidAppConfig(appConfig.value)) {
          console.error("settings window failed to load application config");
        }
      }
      return;
    }

    await getConfig();
  });

  const selectClient = async (category: keyof AppConfigType, protocol: string, name: string) => {
    const updated = await useTauriCoreInvoke("update_config_selection", {
      category,
      protocol,
      name
    });

    if (updated) {
      await setAppConfig(updated as AppConfigType);
    }
  };

  return {
    appConfig,
    selectClient
  };
};

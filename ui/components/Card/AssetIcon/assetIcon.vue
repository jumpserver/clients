<script setup lang="ts">
interface Props {
  type?: string
  platform?: string
  size?: "sm" | "md" | "lg" | "xl"
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  size: "lg",
  class: "",
  type: "linux"
});

const imageProps = computed(() => {
  const prop: { src?: string, alt?: string } = {};
  const normalizedType = (props.type || "").toLowerCase();
  const normalizedPlatform = (props.platform || "").toLowerCase();
  const iconMap: Record<string, string> = {
    windows: "/icons/windows.png",
    linux: "/icons/linux.png",
    mysql: "/icons/mysql.png",
    mariadb: "/icons/mariadb.png",
    oracle: "/icons/oracle.png",
    postgresql: "/icons/postgre.png",
    sqlserver: "/icons/sqlserver.png",
    redis: "/icons/redis.png",
    mongodb: "/icons/mongodb.png",
    dameng: "/icons/dameng.png",
    clickhouse: "/icons/clickhouse.png",
    windows_ad: "/icons/windows.png",
    website: "/icons/browser.png"
  };
  const textIconMap: Record<string, string> = {
    bsd: "B",
    aix: "A",
    macos: "M",
    unix: "U",
    other: "O"
  };

  const textIcon = textIconMap[normalizedPlatform] || textIconMap[normalizedType] || normalizedType.slice(0, 1).toUpperCase();
  const src = iconMap[normalizedType] || "";
  const alt = textIcon || props.type;

  if (src) {
    prop.src = src;
  } else {
    prop.alt = alt;
  }

  return prop;
});

const sizeClasses = computed(() => {
  const sizeMap = {
    sm: "size-6",
    md: "size-7",
    lg: "size-8",
    xl: "size-10"
  };
  return sizeMap[props.size];
});
</script>

<template>
  <UAvatar
    :size="size"
    v-bind="imageProps"
    :ui="{ root: 'rounded-md font-semibold', image: `${sizeClasses} p-1` }"
    class="shrink-0 bg-neutral-200 dark:bg-neutral-600"
    :class="[props.class]"
  />
</template>

import { ref } from 'vue';

export const getDynamicHeight = () => {
  const dynamicHeight = ref(600);

  const updateHeight = () => {
    const target = document.querySelector('.dynamic-target');
    if (target) {
      const height = target.clientHeight;

      // 动态修改高度
      dynamicHeight.value = height - 220;
    }
  };

  // 初始更新
  setTimeout(updateHeight, 0);

  window.addEventListener('resize', updateHeight, false);

  return dynamicHeight;
};

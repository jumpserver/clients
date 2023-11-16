<template>
  <div>
    <img id="logo" src="@/assets/JumpServer.png" alt=""/>
    <h1>{{ $t('Common.JumpServerClient') }}</h1>
    <span>{{ $t('AboutUs.Author') }}：FIT2CLOUD 飞致云 - Halo</span>
  </div>
</template>

<script>
export default {
  name: "AboutPage"
}
</script>

<style lang="scss" scoped>
div {
  width: 100%;
  height: 100%;
  text-align: center;
  color: #e1e1e1e1;
  #logo {
    margin-top: 80px;
    width: 300px;
  }
}
</style>
